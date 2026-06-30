use std::collections::BTreeSet;

use ruter::core::FunctionDiagnostic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct E0432PreflightAnalysis {
    pub count: usize,
    pub hints: Vec<String>,
    pub summary: String,
}

pub fn analyze_e0432_preflight(
    function_diags: &[FunctionDiagnostic],
) -> Option<E0432PreflightAnalysis> {
    let mut count = 0usize;
    let mut hints = Vec::new();
    let mut seen = BTreeSet::new();

    for diag in function_diags.iter().filter(|diag| diag.code == "E0432") {
        count += 1;

        if let Some(value) = diag.suggested_replacement.as_ref() {
            push_hint(value, &mut seen, &mut hints);
        }
        for value in &diag.children_suggested_replacements {
            push_hint(value, &mut seen, &mut hints);
        }
    }

    if count == 0 {
        return None;
    }

    let summary = if hints.is_empty() {
        format!("E0432 unresolved imports detected: count={count}, replacement_hints=none")
    } else {
        format!(
            "E0432 unresolved imports detected: count={count}, replacement_hints={}",
            hints.len()
        )
    };

    Some(E0432PreflightAnalysis {
        count,
        hints,
        summary,
    })
}

fn push_hint(value: &str, seen: &mut BTreeSet<String>, hints: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }
    if seen.insert(trimmed.to_string()) {
        hints.push(trimmed.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diag(
        code: &str,
        replacement: Option<&str>,
        child_replacements: &[&str],
    ) -> FunctionDiagnostic {
        FunctionDiagnostic {
            code: code.to_string(),
            message: "msg".to_string(),
            primary_span: None,
            label: None,
            suggested_replacement: replacement.map(ToString::to_string),
            children_note_messages: vec![],
            children_help_messages: vec![],
            children_suggested_replacements: child_replacements
                .iter()
                .map(|value| value.to_string())
                .collect(),
        }
    }

    #[test]
    fn analyze_e0432_preflight_collects_count_and_deduped_hints() {
        let diagnostics = vec![
            diag("E0432", Some("crate::a"), &["crate::a", "crate::b"]),
            diag("E0432", None, &["crate::b", "crate::c"]),
            diag("E0308", Some("ignored"), &[]),
        ];

        let analysis = analyze_e0432_preflight(&diagnostics).expect("analysis");
        assert_eq!(analysis.count, 2);
        assert_eq!(
            analysis.hints,
            vec![
                "crate::a".to_string(),
                "crate::b".to_string(),
                "crate::c".to_string()
            ]
        );
        assert!(analysis.summary.contains("count=2"));
    }

    #[test]
    fn analyze_e0432_preflight_returns_none_without_e0432() {
        let diagnostics = vec![diag("E0308", None, &[])];
        assert!(analyze_e0432_preflight(&diagnostics).is_none());
    }
}
