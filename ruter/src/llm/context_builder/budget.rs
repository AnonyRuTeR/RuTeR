use std::collections::BTreeSet;

use super::{ContextBuildOutcome, FunctionContextBundleV1};

pub(super) fn prune_bundle_to_budget(
    bundle: &mut FunctionContextBundleV1,
    max_chars: usize,
) -> ContextBuildOutcome {
    let mut seen_truncated = BTreeSet::new();
    loop {
        let char_count = context_char_count(bundle);
        if char_count <= max_chars {
            return ContextBuildOutcome::Ready {
                bundle: bundle.clone(),
                char_count,
            };
        }

        let mut trimmed = false;

        if !bundle.related_impl_blocks.is_empty() {
            let _ = bundle.related_impl_blocks.pop();
            trimmed = true;
            if seen_truncated.insert("impl".to_string()) {
                bundle.truncated_sections.push("impl".to_string());
            }
        } else if !bundle.related_fn_defs.is_empty() {
            let _ = bundle.related_fn_defs.pop();
            trimmed = true;
            if seen_truncated.insert("fn".to_string()) {
                bundle.truncated_sections.push("fn".to_string());
            }
        } else if !bundle.related_type_defs.is_empty() {
            let _ = bundle.related_type_defs.pop();
            trimmed = true;
            if seen_truncated.insert("type".to_string()) {
                bundle.truncated_sections.push("type".to_string());
            }
        } else if !bundle.related_imports.is_empty() {
            let _ = bundle.related_imports.pop();
            trimmed = true;
            if seen_truncated.insert("import".to_string()) {
                bundle.truncated_sections.push("import".to_string());
            }
        } else if !bundle.diagnostics_digest.primary_items.is_empty() {
            let _ = bundle.diagnostics_digest.primary_items.pop();
            trimmed = true;
            if seen_truncated.insert("diagnostic".to_string()) {
                bundle.truncated_sections.push("diagnostic".to_string());
            }
        } else if trim_local_rule_failure_digest(bundle, &mut seen_truncated) {
            trimmed = true;
        } else if trim_rule_error_drift_digest(bundle, &mut seen_truncated) {
            trimmed = true;
        } else if trim_preflight_interceptor_digest(bundle, &mut seen_truncated) {
            trimmed = true;
        } else if trim_previous_round_failure_digest(bundle, &mut seen_truncated) {
            trimmed = true;
        }

        if !trimmed {
            return ContextBuildOutcome::TooLarge {
                function_id: bundle.function_id.clone(),
                char_count,
                max_chars,
                reason: "context cannot be reduced without truncating target function".to_string(),
            };
        }
    }
}

fn trim_local_rule_failure_digest(
    bundle: &mut FunctionContextBundleV1,
    seen_truncated: &mut BTreeSet<String>,
) -> bool {
    let Some(local_digest) = bundle.local_rule_failure_digest.as_mut() else {
        return false;
    };
    if local_digest.summary_lines.is_empty() {
        return false;
    }
    let _ = local_digest.summary_lines.pop();
    if seen_truncated.insert("local_rule".to_string()) {
        bundle.truncated_sections.push("local_rule".to_string());
    }
    true
}

fn trim_previous_round_failure_digest(
    bundle: &mut FunctionContextBundleV1,
    seen_truncated: &mut BTreeSet<String>,
) -> bool {
    let Some(history) = bundle.previous_round_failure_digest.as_mut() else {
        return false;
    };
    if history.candidate_failures.is_empty() {
        return false;
    }
    let _ = history.candidate_failures.pop();
    if seen_truncated.insert("history".to_string()) {
        bundle.truncated_sections.push("history".to_string());
    }
    true
}

fn trim_rule_error_drift_digest(
    bundle: &mut FunctionContextBundleV1,
    seen_truncated: &mut BTreeSet<String>,
) -> bool {
    let Some(digest) = bundle.rule_error_drift_digest.as_mut() else {
        return false;
    };
    if !digest.failed_rule_fix_summaries.is_empty() {
        let _ = digest.failed_rule_fix_summaries.pop();
    } else if !digest.drift_pairs.is_empty() {
        let _ = digest.drift_pairs.pop();
    } else if digest.original_target_function_text.is_some() {
        digest.original_target_function_text = None;
    } else {
        return false;
    }

    if seen_truncated.insert("rule_drift".to_string()) {
        bundle.truncated_sections.push("rule_drift".to_string());
    }
    true
}

fn trim_preflight_interceptor_digest(
    bundle: &mut FunctionContextBundleV1,
    seen_truncated: &mut BTreeSet<String>,
) -> bool {
    let Some(preflight) = bundle.preflight_interceptor_digest.as_mut() else {
        return false;
    };
    if preflight.notes.is_empty() {
        return false;
    }
    let _ = preflight.notes.pop();
    if seen_truncated.insert("preflight".to_string()) {
        bundle.truncated_sections.push("preflight".to_string());
    }
    true
}

pub(super) fn context_char_count(bundle: &FunctionContextBundleV1) -> usize {
    serde_json::to_string(bundle)
        .map(|json| json.chars().count())
        .unwrap_or(usize::MAX)
}
