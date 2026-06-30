use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ruter::core::TestFunction;
use ruter::core::{FixAction, SpanInfo};
use ruter::transformer::CodeTransformer;

use crate::llm::schema::{
    DEFAULT_RAW_RESPONSE_MAX_CHARS, LlmFailureKind, LlmReplayCandidate,
    NormalizedCandidateEvidence, normalize_function_text, normalize_replay_text,
    normalize_with_rustfmt, parse_item_fn, summarize_actions, truncate_for_artifact,
    validate_signature_and_attrs_unchanged,
};
use crate::runtime::function::index::{FunctionIndex, ScopeRange};

use super::workspace_ops::build_workspace_for_plan;

pub(super) struct CurrentFunctionSource {
    pub source: String,
    pub function_for_source: TestFunction,
}

pub(super) fn current_source_for_function(
    crate_path: &Path,
    current_plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    root_function: &TestFunction,
) -> Result<CurrentFunctionSource> {
    let baseline_workspace = build_workspace_for_plan(crate_path, current_plan)
        .context("failed to build baseline workspace for context")?;

    let baseline_index = FunctionIndex::build(&baseline_workspace)
        .context("failed to build baseline index for context")?;
    let source = (|| -> Result<CurrentFunctionSource> {
        let baseline_function = baseline_index
            .find_by_identity(
                &root_function.relative_file,
                &root_function.module_path,
                &root_function.fn_name,
            )
            .context("cannot map function identity in baseline workspace for context")?;
        let source = fs::read_to_string(&baseline_function.file_path).with_context(|| {
            format!(
                "failed to read source {}",
                baseline_function.file_path.display()
            )
        })?;
        let mut function_for_source = root_function.clone();
        // 中文说明：源码来自 baseline workspace，必须同步使用 baseline 的函数范围，
        // 否则在同文件前序补丁引起位移时会切错函数片段。
        function_for_source.byte_start = baseline_function.byte_start;
        function_for_source.byte_end = baseline_function.byte_end;
        function_for_source.line_start = baseline_function.line_start;
        function_for_source.line_end = baseline_function.line_end;
        Ok(CurrentFunctionSource {
            source,
            function_for_source,
        })
    })();
    let _ = fs::remove_dir_all(&baseline_workspace);
    source
}

pub(super) fn plan_without_function_actions(
    plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    crate_path: &Path,
    function_index: &FunctionIndex,
    function_id: &str,
) -> BTreeMap<PathBuf, Vec<FixAction>> {
    let mut next = BTreeMap::new();
    for (file, actions) in plan {
        let mut kept = Vec::new();
        for action in actions {
            let belongs = action_span(action)
                .and_then(|span| function_index.function_for_span(span, crate_path))
                .map(|function| function.id == function_id)
                .unwrap_or(false);
            if !belongs {
                kept.push(action.clone());
            }
        }
        if !kept.is_empty() {
            next.insert(file.clone(), kept);
        }
    }
    next
}

pub(super) fn resolve_candidate_actions(
    crate_path: &Path,
    current_plan: &BTreeMap<PathBuf, Vec<FixAction>>,
    root_function: &TestFunction,
    candidate: &LlmReplayCandidate,
) -> std::result::Result<(Vec<FixAction>, NormalizedCandidateEvidence), (LlmFailureKind, String)> {
    let baseline_workspace = build_workspace_for_plan(crate_path, current_plan).map_err(|err| {
        (
            LlmFailureKind::FunctionMappingFailed,
            format!("failed to build baseline workspace: {err:#}"),
        )
    })?;

    let baseline_index = match FunctionIndex::build(&baseline_workspace) {
        Ok(value) => value,
        Err(err) => {
            let _ = fs::remove_dir_all(&baseline_workspace);
            return Err((
                LlmFailureKind::FunctionMappingFailed,
                format!("failed to build baseline function index: {err:#}"),
            ));
        }
    };

    let Some(baseline_function) = baseline_index.find_by_identity(
        &root_function.relative_file,
        &root_function.module_path,
        &root_function.fn_name,
    ) else {
        let _ = fs::remove_dir_all(&baseline_workspace);
        return Err((
            LlmFailureKind::FunctionMappingFailed,
            "cannot map function identity in baseline workspace".to_string(),
        ));
    };

    let baseline_source = match fs::read_to_string(&baseline_function.file_path) {
        Ok(value) => value,
        Err(err) => {
            let _ = fs::remove_dir_all(&baseline_workspace);
            return Err((
                LlmFailureKind::FunctionMappingFailed,
                format!(
                    "failed to read baseline source {}: {err}",
                    baseline_function.file_path.display()
                ),
            ));
        }
    };

    let root_file_path = crate_path.join(&root_function.relative_file);
    let root_source = match fs::read_to_string(&root_file_path) {
        Ok(value) => value,
        Err(err) => {
            let _ = fs::remove_dir_all(&baseline_workspace);
            return Err((
                LlmFailureKind::FunctionMappingFailed,
                format!(
                    "failed to read root source {}: {err}",
                    root_file_path.display()
                ),
            ));
        }
    };

    let module_scope = baseline_index.enclosing_test_module_scope(&baseline_function);

    let outcome = if let Some(raw_text) = candidate.patched_function_text.as_ref() {
        let normalized_text = normalize_function_text(raw_text).map_err(|detail| {
            (
                LlmFailureKind::LlmOutputInvalidSchema,
                format!("invalid patched_function_text: {detail}"),
            )
        })?;
        let start = baseline_function.byte_start.min(baseline_source.len());
        let end = baseline_function.byte_end.min(baseline_source.len());
        if start >= end {
            Err((
                LlmFailureKind::FunctionMappingFailed,
                "invalid baseline function byte range".to_string(),
            ))
        } else {
            let original_function_text = &baseline_source[start..end];
            let merged_text =
                merge_fn_body_preserving_signature_attrs(original_function_text, &normalized_text)
                    .map_err(|detail| {
                        (
                            LlmFailureKind::LlmOutputInvalidSchema,
                            format!(
                                "failed to merge candidate into target function body: {detail}"
                            ),
                        )
                    })?;

            let action = FixAction::Replace {
                span: SpanInfo {
                    file_path: root_file_path.clone(),
                    byte_start: root_function.byte_start,
                    byte_end: root_function.byte_end,
                    line_start: root_function.line_start,
                    line_end: root_function.line_end,
                    col_start: 1,
                    col_end: 1,
                    is_primary: true,
                    text: vec![],
                    label: None,
                    suggested_replacement: None,
                    suggestion_applicability: None,
                    expansion: None,
                },
                new_content: merged_text.clone(),
            };

            let evidence = NormalizedCandidateEvidence {
                candidate_id: candidate.candidate_id.clone(),
                source_kind: "patched_function_text".to_string(),
                merge_strategy: Some("ast_block_overwrite_preserve_signature_attrs".to_string()),
                action_count: 1,
                action_summaries: summarize_actions(std::slice::from_ref(&action)),
                normalized_text_excerpt: Some(truncate_for_artifact(
                    &merged_text,
                    DEFAULT_RAW_RESPONSE_MAX_CHARS,
                )),
                rationale: candidate.rationale.clone(),
                risk_flags: candidate.risk_flags.clone(),
            };
            Ok((vec![action], evidence))
        }
    } else if let Some(actions) = candidate.actions.as_ref() {
        if actions.len() != 1 {
            Err((
                LlmFailureKind::LlmActionOutOfScope,
                "legacy action mode requires exactly one Replace action".to_string(),
            ))
        } else {
            match &actions[0] {
                FixAction::Replace { span, .. } => {
                    if span.file_path != root_file_path {
                        Err((
                            LlmFailureKind::LlmActionOutOfScope,
                            "legacy action file_path must match target function file".to_string(),
                        ))
                    } else {
                        let in_module_line_scope = span_within_scope_line_range(
                            span,
                            module_scope.as_ref(),
                            root_function,
                        );
                        let in_file_byte_scope =
                            span.byte_start < span.byte_end && span.byte_end <= root_source.len();
                        if !(in_module_line_scope && in_file_byte_scope) {
                            Err((
                                LlmFailureKind::LlmActionOutOfScope,
                                "legacy action range must stay inside target test module scope"
                                    .to_string(),
                            ))
                        } else if !root_source.is_char_boundary(span.byte_start)
                            || !root_source.is_char_boundary(span.byte_end)
                        {
                            Err((
                                LlmFailureKind::LlmActionOutOfScope,
                                "legacy action byte range is not UTF-8 boundary".to_string(),
                            ))
                        } else {
                            let evidence = NormalizedCandidateEvidence {
                                candidate_id: candidate.candidate_id.clone(),
                                source_kind: "legacy_actions".to_string(),
                                merge_strategy: None,
                                action_count: actions.len(),
                                action_summaries: summarize_actions(actions),
                                normalized_text_excerpt: None,
                                rationale: candidate.rationale.clone(),
                                risk_flags: candidate.risk_flags.clone(),
                            };
                            Ok((actions.clone(), evidence))
                        }
                    }
                }
                _ => Err((
                    LlmFailureKind::LlmActionOutOfScope,
                    "legacy action mode does not allow Insert/Delete".to_string(),
                )),
            }
        }
    } else {
        Err((
            LlmFailureKind::LlmOutputInvalidSchema,
            "candidate must provide patched_function_text or actions".to_string(),
        ))
    };

    let _ = fs::remove_dir_all(&baseline_workspace);
    outcome
}

fn span_within_scope_line_range(
    span: &SpanInfo,
    scope: Option<&ScopeRange>,
    root_function: &TestFunction,
) -> bool {
    if let Some(scope) = scope {
        return span.line_start >= scope.line_start && span.line_end <= scope.line_end;
    }
    span.line_start >= root_function.line_start && span.line_end <= root_function.line_end
}

/// 合并策略：仅替换函数体 block，签名与属性始终保留原函数定义。
fn merge_fn_body_preserving_signature_attrs(
    original_function_text: &str,
    patched_function_text: &str,
) -> std::result::Result<String, String> {
    let mut original = parse_item_fn(original_function_text)?;
    let patched = parse_item_fn(patched_function_text)?;
    original.block = patched.block;

    let merged_file = syn::File {
        shebang: None,
        attrs: Vec::new(),
        items: vec![syn::Item::Fn(original)],
    };
    let merged = prettyplease::unparse(&merged_file);
    let normalized =
        normalize_with_rustfmt(&merged).unwrap_or_else(|| normalize_replay_text(&merged));
    parse_item_fn(&normalized)?;
    validate_signature_and_attrs_unchanged(original_function_text, &normalized)?;
    Ok(normalized)
}

pub(super) fn merge_actions_without_conflict(
    union_plan: &mut BTreeMap<PathBuf, Vec<FixAction>>,
    actions: &[FixAction],
) -> bool {
    let mut next_plan = union_plan.clone();
    for action in actions {
        let file_path = match action {
            FixAction::Insert { span, .. }
            | FixAction::Replace { span, .. }
            | FixAction::Delete { span } => span.file_path.clone(),
        };
        next_plan.entry(file_path).or_default().push(action.clone());
    }

    for file_actions in next_plan.values_mut() {
        file_actions.sort_by(|a, b| {
            let (a_start, a_end) = action_range(a);
            let (b_start, b_end) = action_range(b);
            a_start.cmp(&b_start).then_with(|| a_end.cmp(&b_end))
        });
        if CodeTransformer::ensure_no_conflicts(file_actions).is_err() {
            return false;
        }
    }

    *union_plan = next_plan;
    true
}

fn action_range(action: &FixAction) -> (usize, usize) {
    match action {
        FixAction::Insert { span, .. }
        | FixAction::Replace { span, .. }
        | FixAction::Delete { span } => (span.byte_start, span.byte_end),
    }
}

fn action_span(action: &FixAction) -> Option<&SpanInfo> {
    match action {
        FixAction::Insert { span, .. }
        | FixAction::Replace { span, .. }
        | FixAction::Delete { span } => Some(span),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::schema::{
        LlmFailureKind, LlmReplayCandidate, validate_signature_and_attrs_unchanged,
    };
    use crate::runtime::function::index::FunctionIndex;
    use ruter::core::SpanInfo;
    use std::fs;
    use tempfile::tempdir;

    fn write_minimal_test_crate(source: &str) -> tempfile::TempDir {
        let dir = tempdir().expect("tempdir");
        let src_dir = dir.path().join("src");
        fs::create_dir_all(&src_dir).expect("create src dir");
        fs::write(src_dir.join("lib.rs"), source).expect("write lib.rs");
        dir
    }

    fn find_fn(index: &FunctionIndex, name: &str) -> TestFunction {
        index
            .functions()
            .iter()
            .find(|item| item.fn_name == name)
            .cloned()
            .expect("function should exist")
    }

    fn line_of_offset(source: &str, offset: usize) -> usize {
        source[..offset]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1
    }

    #[test]
    fn merge_body_preserves_original_signature_and_attrs() {
        let original = "#[test]\nfn case_a() { let _ = 1; }\n";
        let patched = "fn case_a(v: i32) { let _ = v; }\n";
        let merged = merge_fn_body_preserving_signature_attrs(original, patched).unwrap();
        validate_signature_and_attrs_unchanged(original, &merged).unwrap();
        assert!(merged.contains("let _ = v;"));
    }

    #[test]
    fn merge_body_rejects_invalid_candidate_fn_text() {
        let original = "#[test]\nfn case_a() { let _ = 1; }\n";
        let patched = "this is not rust";
        let err = merge_fn_body_preserving_signature_attrs(original, patched).unwrap_err();
        assert!(err.contains("patched_function_text"));
    }

    #[test]
    fn current_source_for_function_remaps_function_range_after_plan_shift() {
        let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn first() {
        let _ = 1;
    }

    #[test]
    fn target() {
        let p0 = 1;
        assert_eq!(p0, 1);
    }
}
"#;
        let crate_dir = write_minimal_test_crate(source);
        let crate_path = crate_dir.path().canonicalize().expect("canonicalize path");
        let index = FunctionIndex::build(&crate_path).expect("build index");
        let target_fn = find_fn(&index, "target");

        let root_source = fs::read_to_string(crate_path.join("src/lib.rs")).expect("read source");
        let prefix = &root_source[..target_fn.byte_start];
        let shifted_prefix = format!("{prefix}// shift-1\n// shift-2\n");
        let action = FixAction::Replace {
            span: SpanInfo {
                file_path: target_fn.file_path.clone(),
                byte_start: 0,
                byte_end: target_fn.byte_start,
                line_start: 1,
                line_end: target_fn.line_start,
                col_start: 1,
                col_end: 1,
                is_primary: true,
                text: vec![],
                label: None,
                suggested_replacement: None,
                suggestion_applicability: None,
                expansion: None,
            },
            new_content: shifted_prefix,
        };
        let mut plan = BTreeMap::new();
        plan.insert(target_fn.file_path.clone(), vec![action]);

        let mapped =
            current_source_for_function(&crate_path, &plan, &target_fn).expect("map source");
        assert_ne!(mapped.function_for_source.byte_start, target_fn.byte_start);
        assert_ne!(mapped.function_for_source.line_start, target_fn.line_start);
        let remapped_text = &mapped.source
            [mapped.function_for_source.byte_start..mapped.function_for_source.byte_end];
        assert!(remapped_text.contains("fn target()"));
        assert!(syn::parse_str::<syn::ItemFn>(remapped_text).is_ok());

        let stale_start = target_fn.byte_start.min(mapped.source.len());
        let stale_end = target_fn.byte_end.min(mapped.source.len());
        let stale_text = &mapped.source[stale_start..stale_end];
        assert_ne!(stale_text, remapped_text);
    }

    #[test]
    fn current_source_for_function_keeps_range_when_plan_empty() {
        let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn target() {
        let p0 = 1;
        assert_eq!(p0, 1);
    }
}
"#;
        let crate_dir = write_minimal_test_crate(source);
        let crate_path = crate_dir.path().canonicalize().expect("canonicalize path");
        let index = FunctionIndex::build(&crate_path).expect("build index");
        let target_fn = find_fn(&index, "target");

        let mapped =
            current_source_for_function(&crate_path, &BTreeMap::new(), &target_fn).unwrap();
        assert_eq!(mapped.function_for_source.byte_start, target_fn.byte_start);
        assert_eq!(mapped.function_for_source.byte_end, target_fn.byte_end);
        assert_eq!(mapped.function_for_source.line_start, target_fn.line_start);
        assert_eq!(mapped.function_for_source.line_end, target_fn.line_end);
    }

    #[test]
    fn legacy_action_inside_same_test_module_is_allowed() {
        let source = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn helper() {
        let _ = 1;
    }

    #[test]
    fn target() {
        let _ = 2;
    }
}
"#;
        let crate_dir = write_minimal_test_crate(source);
        let crate_path = crate_dir.path().canonicalize().expect("canonicalize path");
        let index = FunctionIndex::build(&crate_path).expect("build index");
        let target_fn = find_fn(&index, "target");
        let helper_expr = "let _ = 1;";
        let start = source.find(helper_expr).expect("helper expr");
        let end = start + helper_expr.len();

        let candidate = LlmReplayCandidate {
            candidate_id: "c1".to_string(),
            patched_function_text: None,
            actions: Some(vec![FixAction::Replace {
                span: SpanInfo {
                    file_path: target_fn.file_path.clone(),
                    byte_start: start,
                    byte_end: end,
                    line_start: line_of_offset(source, start),
                    line_end: line_of_offset(source, end),
                    col_start: 1,
                    col_end: helper_expr.len() + 1,
                    is_primary: true,
                    text: vec![],
                    label: None,
                    suggested_replacement: None,
                    suggestion_applicability: None,
                    expansion: None,
                },
                new_content: "let _ = 10;".to_string(),
            }]),
            rationale: None,
            risk_flags: vec![],
            raw_response: None,
        };

        let resolved =
            resolve_candidate_actions(&crate_path, &BTreeMap::new(), &target_fn, &candidate);
        assert!(
            resolved.is_ok(),
            "same test module action should be accepted"
        );
    }

    #[test]
    fn legacy_action_outside_test_module_is_rejected() {
        let source = r#"
fn prod_fn() {
    let _ = 0;
}

#[cfg(test)]
mod tests {
    #[test]
    fn target() {
        let _ = 2;
    }
}
"#;
        let crate_dir = write_minimal_test_crate(source);
        let crate_path = crate_dir.path().canonicalize().expect("canonicalize path");
        let index = FunctionIndex::build(&crate_path).expect("build index");
        let target_fn = find_fn(&index, "target");
        let prod_expr = "let _ = 0;";
        let start = source.find(prod_expr).expect("prod expr");
        let end = start + prod_expr.len();

        let candidate = LlmReplayCandidate {
            candidate_id: "c1".to_string(),
            patched_function_text: None,
            actions: Some(vec![FixAction::Replace {
                span: SpanInfo {
                    file_path: target_fn.file_path.clone(),
                    byte_start: start,
                    byte_end: end,
                    line_start: line_of_offset(source, start),
                    line_end: line_of_offset(source, end),
                    col_start: 1,
                    col_end: prod_expr.len() + 1,
                    is_primary: true,
                    text: vec![],
                    label: None,
                    suggested_replacement: None,
                    suggestion_applicability: None,
                    expansion: None,
                },
                new_content: "let _ = 10;".to_string(),
            }]),
            rationale: None,
            risk_flags: vec![],
            raw_response: None,
        };

        let resolved =
            resolve_candidate_actions(&crate_path, &BTreeMap::new(), &target_fn, &candidate);
        assert!(
            resolved.is_err(),
            "module-outside action should be rejected"
        );
        let Err((kind, _)) = resolved else {
            unreachable!("checked is_err above")
        };
        assert_eq!(kind, LlmFailureKind::LlmActionOutOfScope);
    }
}
