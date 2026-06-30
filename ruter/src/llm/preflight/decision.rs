use crate::runtime::function::low_value::{LowValueAnalysis, LowValueStatus};
use ruter::patchers::e0308::{E0308Analysis, E0308Classification};
use ruter::patchers::e0599::{E0599Analysis, E0599Classification};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightDecision {
    ContinueToLlm {
        reason: String,
    },
    ContinueToLlmWithHints {
        reason: String,
        hints: Vec<String>,
    },
    SkipLlmCommentOut {
        reason: String,
        risk_flags: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightDecisionResult {
    pub decision: PreflightDecision,
    pub skip_error_code: Option<&'static str>,
}

pub fn decide_preflight(
    low_value: &LowValueAnalysis,
    e0599: Option<&E0599Analysis>,
    e0308: Option<&E0308Analysis>,
) -> PreflightDecisionResult {
    if let Some(analysis) = e0599
        && matches!(
            analysis.classification,
            E0599Classification::MisplacedFreeFunction
        )
    {
        let mut hints = Vec::new();
        if let Some(target) = analysis.target.as_ref() {
            hints.push(format!(
                "The target {} is a free function, not an associated function of {}. Remove {}:: prefix and call {}(...) or crate::{}(...) depending on scope.",
                target.method_name,
                target.normalized_type_name,
                target.normalized_type_name,
                target.method_name,
                target.method_name
            ));
        }
        hints.extend(analysis.recommended_call_forms.iter().map(|call| {
            format!("Prefer call form: {call}")
        }));
        return PreflightDecisionResult {
            decision: PreflightDecision::ContinueToLlmWithHints {
                reason: "E0599 namespace misalignment detected".to_string(),
                hints,
            },
            skip_error_code: None,
        };
    }

    if let Some(analysis) = e0308 {
        if matches!(analysis.classification, E0308Classification::SetupHell) {
            return PreflightDecisionResult {
                decision: PreflightDecision::SkipLlmCommentOut {
                    reason: "E0308 setup-heavy type mismatch (SetupHell)".to_string(),
                    risk_flags: vec!["skip_llm".to_string(), "e0308_setup_hell".to_string()],
                },
                skip_error_code: Some("E0308"),
            };
        }
        if matches!(
            analysis.classification,
            E0308Classification::NominalHallucination
        ) && matches!(low_value.status, LowValueStatus::LowValue)
        {
            return PreflightDecisionResult {
                decision: PreflightDecision::SkipLlmCommentOut {
                    reason: "E0308 nominal hallucination on low-value test".to_string(),
                    risk_flags: vec![
                        "skip_llm".to_string(),
                        "e0308_nominal_low_value".to_string(),
                    ],
                },
                skip_error_code: Some("E0308"),
            };
        }
    }

    if let Some(analysis) = e0599
        && is_e0599_severe(&analysis.classification)
        && matches!(low_value.status, LowValueStatus::LowValue)
    {
        return PreflightDecisionResult {
            decision: PreflightDecision::SkipLlmCommentOut {
                reason: "E0599 severe hallucination on low-value test".to_string(),
                risk_flags: vec!["skip_llm".to_string(), "e0599_severe".to_string()],
            },
            skip_error_code: Some("E0599"),
        };
    }

    if let Some(analysis) = e0308 {
        match analysis.classification {
            E0308Classification::MechanicalMismatch | E0308Classification::WrapperMismatch => {
                return PreflightDecisionResult {
                    decision: PreflightDecision::ContinueToLlmWithHints {
                        reason: "E0308 has actionable type-diff hints".to_string(),
                        hints: analysis.hints.clone(),
                    },
                    skip_error_code: None,
                };
            }
            E0308Classification::NominalHallucination => {
                return PreflightDecisionResult {
                    decision: PreflightDecision::ContinueToLlmWithHints {
                        reason: "E0308 nominal mismatch detected; keep LLM constrained by hints"
                            .to_string(),
                        hints: analysis.hints.clone(),
                    },
                    skip_error_code: None,
                };
            }
            E0308Classification::SetupHell | E0308Classification::Unknown => {}
        }
    }

    PreflightDecisionResult {
        decision: PreflightDecision::ContinueToLlm {
            reason: "preflight did not request skip".to_string(),
        },
        skip_error_code: None,
    }
}

pub fn should_try_budget_e0308_fallback(
    low_value: &LowValueAnalysis,
    e0308: Option<&E0308Analysis>,
) -> bool {
    let Some(analysis) = e0308 else {
        return false;
    };
    matches!(analysis.classification, E0308Classification::SetupHell)
        || (matches!(
            analysis.classification,
            E0308Classification::NominalHallucination
        ) && matches!(low_value.status, LowValueStatus::LowValue))
}

fn is_e0599_severe(kind: &E0599Classification) -> bool {
    matches!(
        kind,
        E0599Classification::SevereNoStruct | E0599Classification::SevereEmptyImpl
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruter::patchers::e0308::{
        ExpectedFoundConfidence, ExpectedFoundPair, ExpectedFoundSource,
    };

    fn low_value(status: LowValueStatus) -> LowValueAnalysis {
        LowValueAnalysis {
            status,
            reason: "x".to_string(),
            markers: vec![],
        }
    }

    fn e0308(kind: E0308Classification) -> E0308Analysis {
        E0308Analysis {
            pair: Some(ExpectedFoundPair {
                expected_type: "Option<String>".to_string(),
                found_type: "String".to_string(),
                source: ExpectedFoundSource::PrimaryLabel,
                confidence: ExpectedFoundConfidence::High,
            }),
            classification: kind,
            hints: vec!["h1".to_string()],
            summary: "s".to_string(),
        }
    }

    fn e0599(kind: E0599Classification) -> E0599Analysis {
        E0599Analysis {
            classification: kind,
            target: None,
            matched_impl_count: 0,
            available_method_signatures: vec![],
            related_free_function_signatures: vec![],
            recommended_call_forms: vec![],
            scope: "none".to_string(),
            summary: "s".to_string(),
        }
    }

    #[test]
    fn preflight_skips_on_e0308_setup_hell() {
        let decision = decide_preflight(
            &low_value(LowValueStatus::HasTestSemantics),
            None,
            Some(&e0308(E0308Classification::SetupHell)),
        );
        assert_eq!(decision.skip_error_code, Some("E0308"));
        assert!(matches!(
            decision.decision,
            PreflightDecision::SkipLlmCommentOut { .. }
        ));
    }

    #[test]
    fn preflight_skips_on_e0308_nominal_plus_low_value() {
        let decision = decide_preflight(
            &low_value(LowValueStatus::LowValue),
            None,
            Some(&e0308(E0308Classification::NominalHallucination)),
        );
        assert_eq!(decision.skip_error_code, Some("E0308"));
        assert!(matches!(
            decision.decision,
            PreflightDecision::SkipLlmCommentOut { .. }
        ));
    }

    #[test]
    fn preflight_skips_on_e0599_severe_plus_low_value() {
        let decision = decide_preflight(
            &low_value(LowValueStatus::LowValue),
            Some(&e0599(E0599Classification::SevereNoStruct)),
            None,
        );
        assert_eq!(decision.skip_error_code, Some("E0599"));
        assert!(matches!(
            decision.decision,
            PreflightDecision::SkipLlmCommentOut { .. }
        ));
    }

    #[test]
    fn preflight_continues_with_hints_for_e0308_wrapper_or_mechanical() {
        let decision = decide_preflight(
            &low_value(LowValueStatus::HasTestSemantics),
            None,
            Some(&e0308(E0308Classification::WrapperMismatch)),
        );
        assert!(matches!(
            decision.decision,
            PreflightDecision::ContinueToLlmWithHints { .. }
        ));
    }

    #[test]
    fn budget_fallback_only_for_e0308_high_risk() {
        assert!(should_try_budget_e0308_fallback(
            &low_value(LowValueStatus::HasTestSemantics),
            Some(&e0308(E0308Classification::SetupHell))
        ));
        assert!(should_try_budget_e0308_fallback(
            &low_value(LowValueStatus::LowValue),
            Some(&e0308(E0308Classification::NominalHallucination))
        ));
        assert!(!should_try_budget_e0308_fallback(
            &low_value(LowValueStatus::HasTestSemantics),
            Some(&e0308(E0308Classification::NominalHallucination))
        ));
        assert!(!should_try_budget_e0308_fallback(
            &low_value(LowValueStatus::HasTestSemantics),
            Some(&e0308(E0308Classification::MechanicalMismatch))
        ));
    }

    #[test]
    fn preflight_continues_with_hints_for_e0599_misplaced_free_function() {
        let mut analysis = e0599(E0599Classification::MisplacedFreeFunction);
        analysis.target = Some(ruter::patchers::e0599::E0599Target {
            method_name: "format_rfc3339".to_string(),
            raw_type_name: "Rfc3339Timestamp".to_string(),
            normalized_type_name: "Rfc3339Timestamp".to_string(),
            raw_kind: "struct".to_string(),
        });
        analysis.recommended_call_forms = vec![
            "format_rfc3339(...)".to_string(),
            "crate::format_rfc3339(...)".to_string(),
        ];
        let decision = decide_preflight(&low_value(LowValueStatus::LowValue), Some(&analysis), None);
        assert!(matches!(
            decision.decision,
            PreflightDecision::ContinueToLlmWithHints { .. }
        ));
        match decision.decision {
            PreflightDecision::ContinueToLlmWithHints { hints, .. } => {
                assert!(hints.iter().any(|line| line.contains("free function")));
                assert!(hints.iter().any(|line| line.contains("crate::format_rfc3339")));
            }
            _ => unreachable!("must be ContinueToLlmWithHints"),
        }
    }
}
