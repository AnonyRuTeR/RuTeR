use crate::core::FixAction;
use crate::patchers::e0433::path_resolver::ResolutionResult;
use crate::patchers::e0433::types::ExtendedSpan;

/// Fix Generator for E0433 Patcher
pub struct FixGenerator;

impl FixGenerator {
    const CRATE_PREFIX: &'static str = "crate::";

    /// Generate FixAction based on the resolution result and extended span
    ///
    /// # Arguments
    /// - `extended`: The extended span containing path information
    /// - `resolution`: The result from path resolution
    /// - `source`: Full source text for context-aware rewrite normalization
    /// # Returns
    /// - `Some(FixAction::Replace)`
    /// # E.g.
    /// - extended: "State::new", resolved_path: "crate::foo::State" -> Replace with "crate::foo::State::new"
    pub fn generate(
        extended: &ExtendedSpan,
        resolution: &ResolutionResult,
        source: &str,
    ) -> Option<FixAction> {
        let resolved_path = match resolution {
            ResolutionResult::Resolved { path, .. } => path.as_str(),
            ResolutionResult::Ambiguous { .. } | ResolutionResult::Unresolved { .. } => {
                return None;
            }
        };

        // Original Path (join with '::')
        let original_path = extended
            .segments
            .iter()
            .map(|seg| seg.name.as_str())
            .collect::<Vec<_>>()
            .join("::");

        // Try to keep suffix like generics or method calls
        let rewritten = Self::normalize_rewrite_with_context(
            extended,
            source,
            Self::rewrite_with_suffix(extended, resolved_path),
        );

        // If the rewritten text is unchanged but the source has an immediate
        // `crate::` before the span and path starts with known extern crate,
        // expand replacement span backward to drop the redundant prefix.
        if rewritten == original_path {
            if Self::has_immediate_crate_prefix_before_span(source, extended.extended_span.byte_start)
                && !rewritten.starts_with(Self::CRATE_PREFIX)
                && extended.extended_span.byte_start >= Self::CRATE_PREFIX.len()
            {
                let mut widened_span = extended.extended_span.clone();
                widened_span.byte_start -= Self::CRATE_PREFIX.len();
                widened_span.col_start = widened_span.col_start.saturating_sub(Self::CRATE_PREFIX.len());
                return Some(FixAction::Replace {
                    span: widened_span,
                    new_content: rewritten,
                });
            }
            return None;
        }

        Some(FixAction::Replace {
            span: extended.extended_span.clone(),
            new_content: rewritten,
        })
    }

    /// Rewrite the resolved path while preserving suffixes
    ///
    /// # E.g.
    /// - extended: "State::new", resolved_path: "crate::foo::State" -> "crate::foo::State::new"
    fn rewrite_with_suffix(extended: &ExtendedSpan, resolved_path: &str) -> String {
        let resolved_tail = resolved_path.rsplit("::").next().unwrap_or(resolved_path);

        // Find the segment matching the resolved tail
        if let Some(idx) = extended
            .segments
            .iter()
            .position(|s| s.name == resolved_tail)
        {
            let suffix = extended.segments[idx + 1..]
                .iter()
                .map(|seg| seg.name.as_str())
                .collect::<Vec<_>>();

            if suffix.is_empty() {
                resolved_path.to_string()
            } else {
                format!("{}::{}", resolved_path, suffix.join("::"))
            }
        } else {
            resolved_path.to_string()
        }
    }

    fn normalize_rewrite_with_context(extended: &ExtendedSpan, source: &str, rewritten: String) -> String {
        if !rewritten.starts_with("crate::") {
            return rewritten;
        }
        if Self::has_immediate_crate_prefix_before_span(source, extended.extended_span.byte_start) {
            return rewritten.trim_start_matches("crate::").to_string();
        }
        rewritten
    }

    fn has_immediate_crate_prefix_before_span(source: &str, byte_start: usize) -> bool {
        let bytes = source.as_bytes();
        let mut cursor = byte_start.min(bytes.len());
        while cursor > 0 && bytes[cursor - 1].is_ascii_whitespace() {
            cursor -= 1;
        }
        if cursor < Self::CRATE_PREFIX.len() {
            return false;
        }
        source
            .get(cursor - Self::CRATE_PREFIX.len()..cursor)
            .map(|slice| slice == Self::CRATE_PREFIX)
            .unwrap_or(false)
    }

}

/// Unit tests for FixGenerator
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::SpanInfo;
    use crate::patchers::e0433::path_resolver::{ResolutionResult, ResolutionSource};
    use crate::patchers::e0433::types::{Confidence, ExtendedSpan, PathSegment, PathStructure};
    use std::path::PathBuf;

    fn mk_span(start: usize, end: usize) -> SpanInfo {
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

    fn mk_extended(names: &[&str], structure: PathStructure) -> ExtendedSpan {
        let mut cursor = 10usize;
        let mut segments = Vec::new();

        for (i, name) in names.iter().enumerate() {
            let start = cursor;
            let end = start + name.len();
            segments.push(PathSegment {
                name: (*name).to_string(),
                byte_range: start..end,
            });
            cursor = end;
            if i + 1 < names.len() {
                cursor += 2; // 模拟 "::"
            }
        }

        ExtendedSpan {
            original_span: mk_span(10, 10 + names[0].len()),
            extended_span: mk_span(10, cursor),
            segments,
            structure,
        }
    }

    fn resolved(path: &str) -> ResolutionResult {
        ResolutionResult::Resolved {
            path: path.to_string(),
            confidence: Confidence::High,
            source: ResolutionSource::CompilerSuggestion,
        }
    }

    #[test]
    fn generate_single_ident_replace() {
        let ext = mk_extended(&["State"], PathStructure::SingleIdent);
        let fix = FixGenerator::generate(
            &ext,
            &resolved("crate::foo::State"),
            "let _ = State;",
        )
        .expect("should fix");

        match fix {
            FixAction::Replace { new_content, .. } => {
                assert_eq!(new_content, "crate::foo::State");
            }
            _ => panic!("expected Replace"),
        }
    }

    #[test]
    fn generate_method_call_keep_suffix() {
        let ext = mk_extended(&["State", "new"], PathStructure::MethodCall);
        let fix = FixGenerator::generate(
            &ext,
            &resolved("crate::foo::State"),
            "let _ = State::new();",
        )
        .expect("should fix");

        match fix {
            FixAction::Replace { new_content, .. } => {
                assert_eq!(new_content, "crate::foo::State::new");
            }
            _ => panic!("expected Replace"),
        }
    }

    #[test]
    fn generate_module_prefix_keep_method_suffix() {
        let ext = mk_extended(&["foo", "State", "new"], PathStructure::MethodCall);
        let fix = FixGenerator::generate(
            &ext,
            &resolved("crate::foo::bar::State"),
            "let _ = foo::State::new();",
        )
        .expect("should fix");

        match fix {
            FixAction::Replace { new_content, .. } => {
                assert_eq!(new_content, "crate::foo::bar::State::new");
            }
            _ => panic!("expected Replace"),
        }
    }

    #[test]
    fn generate_returns_none_on_ambiguous() {
        let ext = mk_extended(&["State"], PathStructure::SingleIdent);
        let res = ResolutionResult::Ambiguous {
            candidates: vec!["a::State".to_string(), "b::State".to_string()],
        };

        let fix = FixGenerator::generate(&ext, &res, "let _ = State;");
        assert!(fix.is_none());
    }

    #[test]
    fn generate_returns_none_on_noop() {
        let ext = mk_extended(&["crate", "foo", "State"], PathStructure::ModuleAccess);
        let fix = FixGenerator::generate(&ext, &resolved("crate::foo::State"), "let _ = crate::foo::State;");
        assert!(fix.is_none());
    }

    #[test]
    fn generate_dedups_leading_crate_prefix_when_span_is_after_crate() {
        let source = "use crate::run_id::wrapper::Duration;";
        let start = source.find("run_id").expect("run_id should exist");
        let ext = ExtendedSpan {
            original_span: mk_span(start, start + "run_id".len()),
            extended_span: mk_span(start, start + "run_id::wrapper::Duration".len()),
            segments: vec![
                PathSegment {
                    name: "run_id".to_string(),
                    byte_range: start..(start + "run_id".len()),
                },
                PathSegment {
                    name: "wrapper".to_string(),
                    byte_range: (start + "run_id::".len())
                        ..(start + "run_id::wrapper".len()),
                },
                PathSegment {
                    name: "Duration".to_string(),
                    byte_range: (start + "run_id::wrapper::".len())
                        ..(start + "run_id::wrapper::Duration".len()),
                },
            ],
            structure: PathStructure::NestedPath,
        };
        let fix = FixGenerator::generate(
            &ext,
            &resolved("crate::wrapper::Duration"),
            source,
        )
        .expect("should fix");

        match fix {
            FixAction::Replace { new_content, .. } => {
                assert_eq!(new_content, "wrapper::Duration");
            }
            _ => panic!("expected Replace"),
        }
    }

    #[test]
    fn generate_drops_leading_crate_prefix_for_known_extern_crate_noop_rewrite() {
        let source = "use crate::core::clone::Clone;";
        let start = source.find("core").expect("core should exist");
        let ext = ExtendedSpan {
            original_span: mk_span(start, start + "core".len()),
            extended_span: mk_span(start, start + "core::clone::Clone".len()),
            segments: vec![
                PathSegment {
                    name: "core".to_string(),
                    byte_range: start..(start + "core".len()),
                },
                PathSegment {
                    name: "clone".to_string(),
                    byte_range: (start + "core::".len())..(start + "core::clone".len()),
                },
                PathSegment {
                    name: "Clone".to_string(),
                    byte_range: (start + "core::clone::".len())
                        ..(start + "core::clone::Clone".len()),
                },
            ],
            structure: PathStructure::NestedPath,
        };
        let fix = FixGenerator::generate(&ext, &resolved("core::clone::Clone"), source)
            .expect("should drop crate prefix");

        match fix {
            FixAction::Replace { span, new_content } => {
                assert_eq!(new_content, "core::clone::Clone");
                assert_eq!(span.byte_start, source.find("crate::").unwrap());
                assert_eq!(span.byte_end, start + "core::clone::Clone".len());
            }
            _ => panic!("expected Replace"),
        }
    }
}
