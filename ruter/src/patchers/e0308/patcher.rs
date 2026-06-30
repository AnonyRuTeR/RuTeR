use std::fs;
use std::path::Path;

use crate::core::{Diagnostic, ErrorCode, FixAction, Result, RuTeRError, Severity, SpanInfo};
use crate::patchers::Patcher;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Attribute, Item};

use super::analyzer::{E0308Classification, E0308DiagnosticInput, analyze_e0308_diagnostic};

#[derive(Debug, Clone)]
struct TestModuleScope {
    line_start: usize,
    byte_start: usize,
    byte_end: usize,
}

/// E0308 patcher with deterministic nominal-drift repair.
///
/// It targets one common drift: expected and found nominal paths share the
/// same tail type name, and the found path was injected into a nearby binding
/// statement inside test module code.
#[derive(Debug, Default)]
pub struct E0308Patcher;

impl E0308Patcher {
    pub fn new() -> Self {
        Self
    }

    fn primary_span(diagnostic: &Diagnostic) -> Option<&SpanInfo> {
        diagnostic
            .span
            .iter()
            .find(|span| span.is_primary)
            .or_else(|| diagnostic.span.first())
    }

    fn load_source(path: &Path) -> Result<String> {
        fs::read_to_string(path).map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                RuTeRError::SourceFileNotFound(path.display().to_string())
            } else {
                RuTeRError::IoError(err)
            }
        })
    }

    fn diagnostic_input(diagnostic: &Diagnostic) -> E0308DiagnosticInput {
        E0308DiagnosticInput {
            message: diagnostic.message.clone(),
            primary_label: Self::primary_span(diagnostic).and_then(|span| span.label.clone()),
            children_note_messages: collect_child_messages_by_level(
                &diagnostic.children,
                Severity::Note,
            ),
            children_help_messages: collect_child_messages_by_level(
                &diagnostic.children,
                Severity::Help,
            ),
            children_suggested_replacements: collect_child_suggested_replacements(
                &diagnostic.children,
            ),
        }
    }

    fn is_path_like(raw: &str) -> bool {
        let trimmed = strip_generic_suffix(raw).trim();
        if !trimmed.contains("::") {
            return false;
        }
        trimmed.split("::").all(is_identifier_like)
    }

    fn is_supported_nominal_pair(expected: &str, found: &str) -> bool {
        if expected == found {
            return false;
        }
        if !Self::is_path_like(expected) || !Self::is_path_like(found) {
            return false;
        }
        last_segment(strip_generic_suffix(expected)) == last_segment(strip_generic_suffix(found))
    }

    /// Build one replace action for nominal drift inside test module scope.
    fn build_nominal_drift_action(
        source: &str,
        primary: &SpanInfo,
        expected_type: &str,
        found_type: &str,
    ) -> Option<FixAction> {
        let scope = enclosing_test_module_scope(source, &primary.file_path, primary.line_start)?;
        let line_offsets = line_start_offsets(source);
        let needle = format!("{found_type}::");
        let replacement = expected_type.to_string();
        if replacement.contains("crate::std::") {
            return None;
        }

        let primary_anchor = primary.byte_start.clamp(scope.byte_start, scope.byte_end);
        if primary_anchor <= scope.byte_start {
            return None;
        }

        let primary_symbol = source
            .get(primary.byte_start..primary.byte_end)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or_default()
            .to_string();

        // 中文说明：优先绑定行匹配（let + 变量名 + found path），若命中不唯一则 fail-closed。
        let mut binding_hits = Vec::new();
        if !primary_symbol.is_empty() {
            for line in scope.line_start..primary.line_start {
                let (line_start, line_end) =
                    line_bounds_for_line(source, &line_offsets, line, source.len())?;
                if line_start >= line_end || line_end > primary_anchor {
                    continue;
                }
                let line_text = source.get(line_start..line_end)?;
                if !line_text.contains("let ") {
                    continue;
                }
                if !line_text.contains(&primary_symbol) || !line_text.contains(&needle) {
                    continue;
                }
                if let Some(offset) = line_text.find(&needle) {
                    binding_hits.push(line_start + offset);
                }
            }
        }

        let patch_start = if binding_hits.len() == 1 {
            binding_hits[0]
        } else if binding_hits.len() > 1 {
            return None;
        } else {
            let before = source.get(scope.byte_start..primary_anchor)?;
            let hits = before
                .match_indices(&needle)
                .map(|(offset, _)| scope.byte_start + offset)
                .collect::<Vec<_>>();
            if hits.len() != 1 {
                return None;
            }
            hits[0]
        };

        let patch_end = patch_start + found_type.len();
        if patch_end > source.len()
            || !source.is_char_boundary(patch_start)
            || !source.is_char_boundary(patch_end)
        {
            return None;
        }
        if !source
            .get(patch_start..patch_end)
            .is_some_and(|text| text == found_type)
        {
            return None;
        }

        let (line_start, col_start) = line_col_for_offset(&line_offsets, patch_start)?;
        let (_, col_end_offset) = line_col_for_offset(&line_offsets, patch_end)?;

        Some(FixAction::Replace {
            span: SpanInfo {
                file_path: primary.file_path.clone(),
                byte_start: patch_start,
                byte_end: patch_end,
                line_start,
                line_end: line_start,
                col_start,
                col_end: col_end_offset,
                is_primary: true,
                text: vec![],
                label: None,
                suggested_replacement: None,
                suggestion_applicability: None,
                expansion: None,
            },
            new_content: replacement,
        })
    }
}

impl Patcher for E0308Patcher {
    fn error_code(&self) -> ErrorCode {
        ErrorCode::E0308
    }

    fn analyze(&self, diagnostic: &Diagnostic) -> Result<Vec<FixAction>> {
        if !self.can_handle(diagnostic) {
            return Ok(Vec::new());
        }

        let Some(primary) = Self::primary_span(diagnostic) else {
            return Ok(Vec::new());
        };
        let source = Self::load_source(&primary.file_path)?;
        if primary.byte_start >= primary.byte_end || primary.byte_end > source.len() {
            return Ok(Vec::new());
        }

        let analysis = analyze_e0308_diagnostic(&Self::diagnostic_input(diagnostic));
        if analysis.classification != E0308Classification::NominalHallucination {
            return Ok(Vec::new());
        }
        let Some(pair) = analysis.pair else {
            return Ok(Vec::new());
        };
        if !Self::is_supported_nominal_pair(&pair.expected_type, &pair.found_type) {
            return Ok(Vec::new());
        }

        let action = Self::build_nominal_drift_action(
            &source,
            primary,
            &pair.expected_type,
            &pair.found_type,
        );
        Ok(action.into_iter().collect())
    }

    fn description(&self) -> &'static str {
        "Patcher for E0308 nominal path drift in test-module scope"
    }
}

fn strip_generic_suffix(raw: &str) -> &str {
    raw.split('<').next().unwrap_or(raw).trim()
}

fn last_segment(raw: &str) -> &str {
    raw.rsplit("::").next().unwrap_or(raw).trim()
}

fn is_identifier_like(segment: &str) -> bool {
    let trimmed = segment.trim();
    if trimmed.is_empty() {
        return false;
    }
    trimmed
        .chars()
        .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn line_start_offsets(source: &str) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (idx, byte) in source.bytes().enumerate() {
        if byte == b'\n' && idx + 1 <= source.len() {
            offsets.push(idx + 1);
        }
    }
    offsets
}

fn line_bounds_for_line(
    source: &str,
    line_offsets: &[usize],
    line: usize,
    source_len: usize,
) -> Option<(usize, usize)> {
    if line == 0 || line > line_offsets.len() {
        return None;
    }
    let start = line_offsets[line - 1];
    let mut end = if line < line_offsets.len() {
        line_offsets[line]
    } else {
        source_len
    };
    while end > start && source.as_bytes().get(end - 1).copied() == Some(b'\n') {
        end -= 1;
    }
    Some((start, end))
}

fn line_col_for_offset(line_offsets: &[usize], offset: usize) -> Option<(usize, usize)> {
    let idx = match line_offsets.binary_search(&offset) {
        Ok(found) => found,
        Err(insert) => insert.checked_sub(1)?,
    };
    let line = idx + 1;
    let col = offset.checked_sub(line_offsets[idx])? + 1;
    Some((line, col))
}

fn collect_child_messages_by_level(children: &[Diagnostic], level: Severity) -> Vec<String> {
    let mut out = Vec::new();
    collect_child_messages_recursive(children, &level, &mut out);
    dedup_keep_order(out)
}

fn collect_child_messages_recursive(
    children: &[Diagnostic],
    level: &Severity,
    out: &mut Vec<String>,
) {
    for child in children {
        if &child.severity == level {
            let line = child.message.trim();
            if !line.is_empty() {
                out.push(line.to_string());
            }
        }
        collect_child_messages_recursive(&child.children, level, out);
    }
}

fn collect_child_suggested_replacements(children: &[Diagnostic]) -> Vec<String> {
    let mut out = Vec::new();
    collect_child_suggested_replacements_recursive(children, &mut out);
    dedup_keep_order(out)
}

fn collect_child_suggested_replacements_recursive(children: &[Diagnostic], out: &mut Vec<String>) {
    for child in children {
        for span in &child.span {
            if let Some(replacement) = span.suggested_replacement.as_ref() {
                let line = replacement.trim();
                if !line.is_empty() {
                    out.push(line.to_string());
                }
            }
        }
        collect_child_suggested_replacements_recursive(&child.children, out);
    }
}

fn dedup_keep_order(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for item in items {
        if seen.insert(item.clone()) {
            out.push(item);
        }
    }
    out
}

fn enclosing_test_module_scope(
    source: &str,
    file_path: &Path,
    target_line: usize,
) -> Option<TestModuleScope> {
    let line_offsets = line_start_offsets(source);
    let full_scope = TestModuleScope {
        line_start: 1,
        byte_start: 0,
        byte_end: source.len(),
    };

    if file_path
        .components()
        .any(|item| item.as_os_str() == "tests")
    {
        return Some(full_scope);
    }

    let syntax = syn::parse_file(source).ok()?;
    let mut candidates = Vec::<(usize, usize, usize)>::new();
    collect_cfg_test_module_ranges(&syntax.items, false, 0, &mut candidates);
    let selected = candidates
        .into_iter()
        .filter(|(_, line_start, line_end)| target_line >= *line_start && target_line <= *line_end)
        .max_by(|lhs, rhs| {
            lhs.0.cmp(&rhs.0).then_with(|| {
                let lhs_span = lhs.2.saturating_sub(lhs.1);
                let rhs_span = rhs.2.saturating_sub(rhs.1);
                rhs_span.cmp(&lhs_span)
            })
        })?;

    let byte_start = line_offsets.get(selected.1.saturating_sub(1)).copied()?;
    let byte_end = if selected.2 < line_offsets.len() {
        line_offsets[selected.2]
    } else {
        source.len()
    };
    if byte_start >= byte_end {
        return None;
    }

    Some(TestModuleScope {
        line_start: selected.1,
        byte_start,
        byte_end,
    })
}

fn collect_cfg_test_module_ranges(
    items: &[Item],
    in_cfg_test: bool,
    depth: usize,
    out: &mut Vec<(usize, usize, usize)>,
) {
    for item in items {
        let Item::Mod(item_mod) = item else {
            continue;
        };
        let Some((_, nested_items)) = &item_mod.content else {
            continue;
        };
        let next_in_cfg_test = in_cfg_test || has_cfg_test_attr(&item_mod.attrs);
        if next_in_cfg_test {
            let span = item_mod.span();
            let start = span.start();
            let end = span.end();
            out.push((
                depth + 1,
                start.line.max(1),
                end.line.max(start.line.max(1)),
            ));
        }
        collect_cfg_test_module_ranges(nested_items, next_in_cfg_test, depth + 1, out);
    }
}

fn has_cfg_test_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg") && attr.to_token_stream().to_string().contains("test")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CompilerCode, Severity};
    use tempfile::tempdir;

    fn e0308_diagnostic(
        file_path: &Path,
        byte_start: usize,
        byte_end: usize,
        line_start: usize,
        line_end: usize,
        label: &str,
    ) -> Diagnostic {
        Diagnostic {
            message_type: Some("diagnostic".to_string()),
            code: Some(CompilerCode {
                code: ErrorCode::E0308,
                raw_code: None,
                explanation: None,
            }),
            message: "mismatched types".to_string(),
            span: vec![SpanInfo {
                file_path: file_path.to_path_buf(),
                byte_start,
                byte_end,
                line_start,
                line_end,
                col_start: 1,
                col_end: 1,
                is_primary: true,
                text: vec![],
                label: Some(label.to_string()),
                suggested_replacement: None,
                suggestion_applicability: None,
                expansion: None,
            }],
            severity: Severity::Error,
            children: vec![],
            rendered: None,
        }
    }

    #[test]
    fn e0308_patcher_reports_target_code() {
        let patcher = E0308Patcher::new();
        assert_eq!(patcher.error_code(), ErrorCode::E0308);
    }

    #[test]
    fn e0308_patcher_rewrites_nearest_nominal_binding_in_test_module() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("mkdir");
        let file = src.join("lib.rs");
        let source = r#"
mod wrapper {
    pub struct Duration;
}

#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {
        let p0 = std::time::Duration::from(std::time::Duration::new(5, 0));
        <wrapper::Duration>::into(p0);
    }
}
"#;
        fs::write(&file, source).expect("write source");
        let call = "<wrapper::Duration>::into(p0)";
        let byte_start = source.find("p0);").expect("symbol start");
        let byte_end = byte_start + 2;
        let line_start = source[..byte_start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        let line_end = line_start;
        assert!(source.contains(call));

        let diagnostic = e0308_diagnostic(
            &file,
            byte_start,
            byte_end,
            line_start,
            line_end,
            "expected `wrapper::Duration`, found `std::time::Duration`",
        );
        let patcher = E0308Patcher::new();
        let actions = patcher.analyze(&diagnostic).expect("analyze");
        assert_eq!(actions.len(), 1);
        let FixAction::Replace { new_content, .. } = &actions[0] else {
            panic!("expected replace");
        };
        assert_eq!(new_content, "wrapper::Duration");
    }

    #[test]
    fn e0308_patcher_fails_closed_when_multiple_binding_candidates_exist() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("mkdir");
        let file = src.join("lib.rs");
        let source = r#"
mod wrapper {
    pub struct Duration;
}

#[cfg(test)]
mod tests {
    #[test]
    fn case_a() {
        let p0 = std::time::Duration::from(std::time::Duration::new(5, 0));
        let p0 = std::time::Duration::from(std::time::Duration::new(6, 0));
        <wrapper::Duration>::into(p0);
    }
}
"#;
        fs::write(&file, source).expect("write source");
        let byte_start = source.find("p0);").expect("symbol start");
        let line_start = source[..byte_start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        let diagnostic = e0308_diagnostic(
            &file,
            byte_start,
            byte_start + 2,
            line_start,
            line_start,
            "expected `wrapper::Duration`, found `std::time::Duration`",
        );
        let patcher = E0308Patcher::new();
        let actions = patcher.analyze(&diagnostic).expect("analyze");
        assert!(actions.is_empty(), "ambiguous rewrite must be rejected");
    }

    #[test]
    fn e0308_patcher_skips_non_test_scope() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("src");
        fs::create_dir_all(&src).expect("mkdir");
        let file = src.join("lib.rs");
        let source = r#"
mod wrapper {
    pub struct Duration;
}

fn not_test() {
    let p0 = std::time::Duration::from(std::time::Duration::new(5, 0));
    <wrapper::Duration>::into(p0);
}
"#;
        fs::write(&file, source).expect("write source");
        let byte_start = source.find("p0);").expect("symbol start");
        let line_start = source[..byte_start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        let diagnostic = e0308_diagnostic(
            &file,
            byte_start,
            byte_start + 2,
            line_start,
            line_start,
            "expected `wrapper::Duration`, found `std::time::Duration`",
        );
        let patcher = E0308Patcher::new();
        let actions = patcher.analyze(&diagnostic).expect("analyze");
        assert!(actions.is_empty());
    }
}
