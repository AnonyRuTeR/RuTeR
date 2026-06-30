use crate::patchers::e0433::types::{CandidateFix, CandidateKind, Confidence};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedResolution {
    pub path: String,
    pub confidence: Confidence,
    pub source: ResolutionSource,
    pub score: i32,
}

/// Path resolution result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionResult {
    /// Successfully resolved path with confidence level
    Resolved {
        path: String,
        confidence: Confidence,
        source: ResolutionSource,
    },
    /// Multiple ambiguous paths
    Ambiguous { candidates: Vec<String> },
    /// Unable to resolve path
    Unresolved { reason: String },
}

/// Source of the path resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionSource {
    CompilerMachineApplicable,
    CompilerSuggestion,
    Heuristic,
    StdLibMap,
}

/// Resolver constraint context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolverContext {
    /// Whether current diagnostic is in mutable test code.
    pub is_test_context: bool,
    /// Whether dependency candidates are allowed in test context.
    pub allow_dependency_action_in_tests: bool,
}

impl Default for ResolverContext {
    fn default() -> Self {
        Self {
            is_test_context: false,
            allow_dependency_action_in_tests: false,
        }
    }
}

#[derive(Debug, Clone)]
struct ScoredCandidate {
    path: String,
    confidence: Confidence,
    source: ResolutionSource,
    score: i32,
}

/// Path Resolver for E0433 Patcher
pub struct PathResolver;

impl PathResolver {
    /// Backward-compatible API.
    pub fn resolve(candidates: Vec<CandidateFix>) -> ResolutionResult {
        Self::resolve_with_context(candidates, ResolverContext::default())
    }

    /// Resolve the best candidate from a list by constraints then scoring.
    pub fn resolve_with_context(
        candidates: Vec<CandidateFix>,
        ctx: ResolverContext,
    ) -> ResolutionResult {
        let (ranked, blocked_reasons) = Self::rank_with_context_internal(candidates, ctx);

        if ranked.is_empty() {
            let reason = if blocked_reasons.is_empty() {
                "No high-confidence candidate fixes provided".to_string()
            } else {
                format!(
                    "No high-confidence candidate fixes provided after constraints: {}",
                    blocked_reasons.join("; ")
                )
            };
            return ResolutionResult::Unresolved { reason };
        }

        let best = &ranked[0];

        ResolutionResult::Resolved {
            path: best.path.clone(),
            confidence: best.confidence,
            source: best.source,
        }
    }

    /// Rank executable candidates by score and return high-confidence items only.
    pub fn rank_with_context(
        candidates: Vec<CandidateFix>,
        ctx: ResolverContext,
    ) -> Vec<RankedResolution> {
        let (ranked, _) = Self::rank_with_context_internal(candidates, ctx);
        ranked
    }

    fn rank_with_context_internal(
        candidates: Vec<CandidateFix>,
        ctx: ResolverContext,
    ) -> (Vec<RankedResolution>, Vec<String>) {
        let (filtered, mut blocked_reasons) = Self::apply_constraints(candidates, ctx);
        if filtered.is_empty() {
            return (Vec::new(), blocked_reasons);
        }

        let mut scored = Self::score_candidates(&filtered);
        scored.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.path.len().cmp(&b.path.len()))
                .then_with(|| a.path.cmp(&b.path))
        });

        let mut ranked = Vec::new();
        for item in scored {
            if item.confidence == Confidence::High {
                ranked.push(RankedResolution {
                    path: item.path,
                    confidence: item.confidence,
                    source: item.source,
                    score: item.score,
                });
            } else {
                blocked_reasons.push(format!(
                    "filtered non-high confidence candidate `{}` ({:?})",
                    item.path, item.confidence
                ));
            }
        }

        (ranked, blocked_reasons)
    }

    fn apply_constraints(
        candidates: Vec<CandidateFix>,
        ctx: ResolverContext,
    ) -> (Vec<CandidateFix>, Vec<String>) {
        let mut filtered = Vec::new();
        let mut blocked = Vec::new();

        for candidate in candidates {
            if !candidate.is_executable() {
                blocked.push(
                    candidate
                        .blocked_reason
                        .clone()
                        .unwrap_or_else(|| "candidate is not executable".to_string()),
                );
                continue;
            }

            // M1-3 policy: by default disable dependency action in test context.
            // Note: extractor currently does not emit DependencyAction.
            if ctx.is_test_context
                && !ctx.allow_dependency_action_in_tests
                && candidate.kind == CandidateKind::DependencyAction
            {
                blocked.push("dependency action disabled in test context".to_string());
                continue;
            }

            filtered.push(candidate);
        }

        (filtered, blocked)
    }

    fn score_candidates(candidates: &[CandidateFix]) -> Vec<ScoredCandidate> {
        candidates
            .iter()
            .filter_map(|candidate| {
                let path = candidate.suggested_path.clone()?;
                let mut score = 0i32;

                score += match candidate.kind {
                    CandidateKind::PathRewrite => 100,
                    CandidateKind::UseImport => 80,
                    CandidateKind::DependencyAction => 30,
                    CandidateKind::NonCodeHint => -10_000,
                };

                score += match candidate.confidence {
                    Confidence::High => 30,
                    Confidence::Medium => 20,
                    Confidence::Low => 10,
                };

                if path.starts_with("crate::") {
                    score += 10;
                } else if path.starts_with("std::") {
                    score += 5;
                }

                // Small preference for local and minimal edits.
                score += (80 - (path.len() as i32).min(80)) / 8;

                let source = if candidate.confidence == Confidence::High {
                    ResolutionSource::CompilerMachineApplicable
                } else {
                    ResolutionSource::CompilerSuggestion
                };

                Some(ScoredCandidate {
                    path,
                    confidence: candidate.confidence,
                    source,
                    score,
                })
            })
            .collect()
    }
}

/// Unit tests for PathResolver
#[cfg(test)]
mod tests {
    use super::*;

    fn make_fix(path: &str, confidence: Confidence) -> CandidateFix {
        CandidateFix {
            bare_identifier: "TestIdent".to_string(),
            suggested_path: Some(path.to_string()),
            confidence,
            kind: CandidateKind::PathRewrite,
            blocked_reason: None,
        }
    }

    fn make_fix_kind(path: &str, confidence: Confidence, kind: CandidateKind) -> CandidateFix {
        CandidateFix {
            bare_identifier: "TestIdent".to_string(),
            suggested_path: Some(path.to_string()),
            confidence,
            kind,
            blocked_reason: None,
        }
    }

    #[test]
    fn test_empty_candidates() {
        let result = PathResolver::resolve(vec![]);

        match result {
            ResolutionResult::Unresolved { reason } => {
                assert!(reason.contains("No high-confidence candidate fixes provided"));
            }
            _ => panic!("Expected Unresolved, got {:?}", result),
        }
    }

    #[test]
    fn test_non_executable_candidates_are_ignored() {
        let candidates = vec![CandidateFix {
            bare_identifier: "humantime".to_string(),
            suggested_path: None,
            confidence: Confidence::High,
            kind: CandidateKind::NonCodeHint,
            blocked_reason: Some("command hint".to_string()),
        }];

        let result = PathResolver::resolve(candidates);
        match result {
            ResolutionResult::Unresolved { reason } => {
                assert!(reason.contains("No high-confidence candidate fixes provided"));
                assert!(reason.contains("command hint"));
            }
            _ => panic!("Expected Unresolved, got {:?}", result),
        }
    }

    #[test]
    fn test_single_high_confidence() {
        let candidates = vec![make_fix("std::vec::Vec", Confidence::High)];
        let result = PathResolver::resolve(candidates);

        if let ResolutionResult::Resolved {
            path,
            confidence,
            source,
        } = result
        {
            assert_eq!(path, "std::vec::Vec");
            assert_eq!(confidence, Confidence::High);
            assert_eq!(source, ResolutionSource::CompilerMachineApplicable);
        } else {
            panic!("Unexpected result: {:?}", result);
        }
    }

    #[test]
    fn test_scoring_not_first_hit_even_when_both_high() {
        let candidates = vec![
            make_fix("external::very::long::State", Confidence::High),
            make_fix("crate::State", Confidence::High),
        ];
        let result = PathResolver::resolve(candidates);

        if let ResolutionResult::Resolved { path, .. } = result {
            assert_eq!(path, "crate::State");
        } else {
            panic!("Unexpected result: {:?}", result);
        }
    }

    #[test]
    fn test_context_blocks_dependency_action_in_test_code() {
        let candidates = vec![
            make_fix_kind("serde", Confidence::High, CandidateKind::DependencyAction),
            make_fix("crate::foo::State", Confidence::High),
        ];
        let ctx = ResolverContext {
            is_test_context: true,
            allow_dependency_action_in_tests: false,
        };

        let result = PathResolver::resolve_with_context(candidates, ctx);
        if let ResolutionResult::Resolved { path, .. } = result {
            assert_eq!(path, "crate::foo::State");
        } else {
            panic!("Unexpected result: {:?}", result);
        }
    }

    #[test]
    fn test_context_can_allow_dependency_action() {
        let candidates = vec![make_fix_kind(
            "serde",
            Confidence::High,
            CandidateKind::DependencyAction,
        )];
        let ctx = ResolverContext {
            is_test_context: true,
            allow_dependency_action_in_tests: true,
        };

        let result = PathResolver::resolve_with_context(candidates, ctx);
        if let ResolutionResult::Resolved { path, .. } = result {
            assert_eq!(path, "serde");
        } else {
            panic!("Unexpected result: {:?}", result);
        }
    }

    #[test]
    fn test_heuristic_prefer_crate() {
        let candidates = vec![
            make_fix("external::foo", Confidence::High),
            make_fix("std::foo", Confidence::High),
            make_fix("crate::foo", Confidence::High),
        ];
        let result = PathResolver::resolve(candidates);

        if let ResolutionResult::Resolved { path, source, .. } = result {
            assert_eq!(path, "crate::foo");
            assert_eq!(source, ResolutionSource::CompilerMachineApplicable);
        } else {
            panic!("Unexpected result: {:?}", result);
        }
    }

    #[test]
    fn test_heuristic_prefer_short_path() {
        let candidates = vec![
            make_fix("a::b::c", Confidence::High),
            make_fix("a::b", Confidence::High),
        ];
        let result = PathResolver::resolve(candidates);

        if let ResolutionResult::Resolved { path, .. } = result {
            assert_eq!(path, "a::b");
        } else {
            panic!("Unexpected result: {:?}", result);
        }
    }

    #[test]
    fn test_tie_breaker_alphabetical() {
        let candidates = vec![
            make_fix("ccc::ddd", Confidence::Medium),
            make_fix("aaa::bbb", Confidence::Medium),
        ];
        let result = PathResolver::resolve(candidates);

        match result {
            ResolutionResult::Unresolved { reason } => {
                assert!(reason.contains("No high-confidence candidate fixes provided"));
            }
            _ => panic!("Unexpected result: {:?}", result),
        }
    }

    #[test]
    fn test_rank_with_context_returns_sorted_high_confidence_only() {
        let candidates = vec![
            make_fix("crate::best::State", Confidence::High),
            make_fix("std::collections::HashMap", Confidence::High),
            make_fix("crate::medium::State", Confidence::Medium),
        ];
        let ranked = PathResolver::rank_with_context(candidates, ResolverContext::default());

        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].path, "crate::best::State");
        assert_eq!(ranked[1].path, "std::collections::HashMap");
        assert!(
            ranked
                .iter()
                .all(|item| item.confidence == Confidence::High)
        );
    }

    #[test]
    fn test_resolve_with_only_medium_candidates_returns_unresolved() {
        let candidates = vec![
            make_fix("crate::a::State", Confidence::Medium),
            make_fix("crate::b::State", Confidence::Low),
        ];
        let result = PathResolver::resolve(candidates);
        match result {
            ResolutionResult::Unresolved { reason } => {
                assert!(reason.contains("No high-confidence candidate fixes provided"));
            }
            _ => panic!("Expected unresolved, got {:?}", result),
        }
    }
}
