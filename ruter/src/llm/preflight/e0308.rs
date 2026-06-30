use ruter::core::FunctionDiagnostic;
use ruter::patchers::e0308::{
    E0308Analysis, E0308Classification, E0308DiagnosticInput, ExpectedFoundConfidence,
    analyze_e0308_diagnostic,
};

pub fn analyze_e0308_preflight(function_diags: &[FunctionDiagnostic]) -> Option<E0308Analysis> {
    let mut best: Option<E0308Analysis> = None;

    for diag in function_diags.iter().filter(|diag| diag.code == "E0308") {
        let input = E0308DiagnosticInput {
            message: diag.message.clone(),
            primary_label: diag.label.clone(),
            children_note_messages: diag.children_note_messages.clone(),
            children_help_messages: diag.children_help_messages.clone(),
            children_suggested_replacements: diag.children_suggested_replacements.clone(),
        };
        let analysis = analyze_e0308_diagnostic(&input);
        if should_replace_best(best.as_ref(), &analysis) {
            best = Some(analysis);
        }
    }

    best
}

fn should_replace_best(current: Option<&E0308Analysis>, next: &E0308Analysis) -> bool {
    let Some(current) = current else {
        return true;
    };
    let current_key = rank_key(current);
    let next_key = rank_key(next);
    next_key > current_key
}

fn rank_key(analysis: &E0308Analysis) -> (u8, u8, usize) {
    (
        classification_priority(&analysis.classification),
        analysis
            .pair
            .as_ref()
            .map(|pair| confidence_priority(&pair.confidence))
            .unwrap_or(0),
        analysis.hints.len(),
    )
}

fn classification_priority(kind: &E0308Classification) -> u8 {
    match kind {
        E0308Classification::SetupHell => 5,
        E0308Classification::WrapperMismatch => 4,
        E0308Classification::MechanicalMismatch => 3,
        E0308Classification::NominalHallucination => 2,
        E0308Classification::Unknown => 1,
    }
}

fn confidence_priority(kind: &ExpectedFoundConfidence) -> u8 {
    match kind {
        ExpectedFoundConfidence::High => 3,
        ExpectedFoundConfidence::Medium => 2,
        ExpectedFoundConfidence::Low => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_selects_highest_priority_e0308_signal() {
        let diagnostics = vec![
            FunctionDiagnostic {
                code: "E0308".to_string(),
                message: "mismatched types".to_string(),
                primary_span: None,
                label: Some("expected `String`, found `&str`".to_string()),
                suggested_replacement: None,
                children_note_messages: vec![],
                children_help_messages: vec![],
                children_suggested_replacements: vec![],
            },
            FunctionDiagnostic {
                code: "E0308".to_string(),
                message: "mismatched types".to_string(),
                primary_span: None,
                label: Some("expected `fmt::Formatter<'_>`, found `String`".to_string()),
                suggested_replacement: None,
                children_note_messages: vec![],
                children_help_messages: vec![],
                children_suggested_replacements: vec![],
            },
        ];

        let analysis = analyze_e0308_preflight(&diagnostics).expect("analysis");
        assert_eq!(analysis.classification, E0308Classification::SetupHell);
    }
}
