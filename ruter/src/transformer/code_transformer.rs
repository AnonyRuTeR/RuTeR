use crate::core::{FixAction, Result, RuTeRError};

/// Applies text-based replacement actions to source code.
#[derive(Debug, Default, Clone, Copy)]
pub struct CodeTransformer;

#[derive(Debug, Clone)]
struct Replacement {
    start: usize,
    end: usize,
    new_content: String,
}

impl CodeTransformer {
    pub fn new() -> Self {
        Self
    }

    /// Validate and apply replacement actions.
    ///
    /// Note: Only `FixAction::Replace` is supported currently.
    pub fn apply_replacements(&self, source: &str, actions: &[FixAction]) -> Result<String> {
        let mut replacements = Vec::with_capacity(actions.len());

        // Transform FixActions into Replacements
        for action in actions {
            match action {
                FixAction::Replace { span, new_content } => {
                    Self::validate_range(source, span.byte_start, span.byte_end)?;
                    replacements.push(Replacement {
                        start: span.byte_start,
                        end: span.byte_end,
                        new_content: new_content.clone(),
                    });
                }
                other => return Err(RuTeRError::UnsupportedFixAction(format!("{other:?}"))),
            }
        }

        Self::ensure_no_conflict_in_replacements(&replacements)?;

        // Apply replacements from right to left so offsets stay valid.
        replacements.sort_by(|a, b| b.start.cmp(&a.start).then_with(|| b.end.cmp(&a.end)));

        let mut output = source.to_string();
        for replacement in replacements {
            output.replace_range(replacement.start..replacement.end, &replacement.new_content);
        }

        Ok(output)
    }

    /// Ensure no overlapping fix actions
    pub fn ensure_no_conflicts(actions: &[FixAction]) -> Result<()> {
        let mut ranges = Vec::new();
        for action in actions {
            let (start, end) = match action {
                FixAction::Insert { span, .. }
                | FixAction::Replace { span, .. }
                | FixAction::Delete { span } => (span.byte_start, span.byte_end),
            };
            ranges.push((start, end));
        }

        ranges.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        for pair in ranges.windows(2) {
            let prev = pair[0];
            let next = pair[1];
            if next.0 < prev.1 {
                return Err(RuTeRError::ConflictingFixActions(format!(
                    "overlap detected: {}..{} and {}..{}",
                    prev.0, prev.1, next.0, next.1
                )));
            }
        }

        Ok(())
    }

    /// Ensure no overlapping replacements
    fn ensure_no_conflict_in_replacements(replacements: &[Replacement]) -> Result<()> {
        let mut ranges: Vec<(usize, usize)> =
            replacements.iter().map(|r| (r.start, r.end)).collect();
        ranges.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        // Check neighboring ranges for overlap
        for pair in ranges.windows(2) {
            let prev = pair[0];
            let next = pair[1];
            if next.0 < prev.1 {
                return Err(RuTeRError::ConflictingFixActions(format!(
                    "overlap detected: {}..{} and {}..{}",
                    prev.0, prev.1, next.0, next.1
                )));
            }
        }

        Ok(())
    }

    /// Validate that the given byte range is valid within the source string.
    fn validate_range(source: &str, start: usize, end: usize) -> Result<()> {
        if start >= end {
            return Err(RuTeRError::InvalidByteRange(format!(
                "range must satisfy start < end, got {start}..{end}"
            )));
        }

        if end > source.len() {
            return Err(RuTeRError::InvalidByteRange(format!(
                "range end {end} exceeds source length {}",
                source.len()
            )));
        }

        if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
            return Err(RuTeRError::InvalidByteRange(format!(
                "range {start}..{end} is not on UTF-8 char boundaries"
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::SpanInfo;
    use std::path::PathBuf;

    fn span(start: usize, end: usize) -> SpanInfo {
        SpanInfo {
            file_path: PathBuf::from("src/main.rs"),
            byte_start: start,
            byte_end: end,
            line_start: 1,
            line_end: 1,
            col_start: start + 1,
            col_end: end + 1,
            is_primary: true,
            text: vec![],
            label: None,
            suggested_replacement: None,
            suggestion_applicability: None,
            expansion: None,
        }
    }

    #[test]
    fn apply_single_replace() {
        let source = "fn main() { let _ = State::new(); }";
        let start = source.find("State").unwrap();
        let end = start + "State".len();
        let actions = vec![FixAction::Replace {
            span: span(start, end),
            new_content: "crate::foo::State".to_string(),
        }];

        let output = CodeTransformer::new()
            .apply_replacements(source, &actions)
            .unwrap();

        assert_eq!(output, "fn main() { let _ = crate::foo::State::new(); }");
    }

    #[test]
    fn apply_multiple_replacements_descending_offsets() {
        let source = "let a = foo; let b = bar;";
        let foo_start = source.find("foo").unwrap();
        let foo_end = foo_start + 3;
        let bar_start = source.find("bar").unwrap();
        let bar_end = bar_start + 3;

        let actions = vec![
            FixAction::Replace {
                span: span(foo_start, foo_end),
                new_content: "alpha".to_string(),
            },
            FixAction::Replace {
                span: span(bar_start, bar_end),
                new_content: "beta".to_string(),
            },
        ];

        let output = CodeTransformer::new()
            .apply_replacements(source, &actions)
            .unwrap();

        assert_eq!(output, "let a = alpha; let b = beta;");
    }

    #[test]
    fn adjacent_replacements_are_allowed() {
        let source = "abcdef";
        let actions = vec![
            FixAction::Replace {
                span: span(0, 3),
                new_content: "AAA".to_string(),
            },
            FixAction::Replace {
                span: span(3, 6),
                new_content: "BBB".to_string(),
            },
        ];

        let output = CodeTransformer::new()
            .apply_replacements(source, &actions)
            .unwrap();

        assert_eq!(output, "AAABBB");
    }

    #[test]
    fn overlap_replacements_return_error() {
        let source = "abcdef";
        let actions = vec![
            FixAction::Replace {
                span: span(1, 4),
                new_content: "X".to_string(),
            },
            FixAction::Replace {
                span: span(3, 5),
                new_content: "Y".to_string(),
            },
        ];

        let err = CodeTransformer::new()
            .apply_replacements(source, &actions)
            .expect_err("must fail due to overlap");

        assert!(matches!(err, RuTeRError::ConflictingFixActions(_)));
    }

    #[test]
    fn out_of_bounds_range_returns_error() {
        let source = "abc";
        let actions = vec![FixAction::Replace {
            span: span(0, 10),
            new_content: "x".to_string(),
        }];

        let err = CodeTransformer::new()
            .apply_replacements(source, &actions)
            .expect_err("must fail due to invalid range");

        assert!(matches!(err, RuTeRError::InvalidByteRange(_)));
    }

    #[test]
    fn non_replace_action_returns_error() {
        let source = "abc";
        let actions = vec![FixAction::Delete { span: span(0, 1) }];

        let err = CodeTransformer::new()
            .apply_replacements(source, &actions)
            .expect_err("must fail due to unsupported action");

        assert!(matches!(err, RuTeRError::UnsupportedFixAction(_)));
    }
}
