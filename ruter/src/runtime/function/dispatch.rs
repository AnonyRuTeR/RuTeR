use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::path::PathBuf;

use ruter::core::{Diagnostic, ErrorCode, Severity, SpanInfo};
use serde::{Deserialize, Serialize};

use crate::runtime::function::index::FunctionIndex;
use crate::runtime::function::low_value::{LowValueStatus, analyze_test_function_low_value};

/// 函数任务中的诊断引用，保留原始索引以保证稳定排序与冲突仲裁可回放。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRef {
    pub diagnostic_index: usize,
    pub code: String,
    pub diagnostic: Diagnostic,
}

/// 每个测试函数的补丁任务输入。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionPatchTask {
    pub function_id: String,
    pub file_path: PathBuf,
    pub function_line_span: (usize, usize),
    pub diagnostics_with_index: Vec<DiagnosticRef>,
    pub error_code_counts: BTreeMap<String, usize>,
    pub implemented_rule_codes_present: BTreeSet<ErrorCode>,
    pub unimplemented_codes_present: BTreeSet<String>,
    pub low_value_status: LowValueStatus,
    pub low_value_reason: String,
    pub low_value_markers: Vec<String>,
}

/// 函数级分发决策。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionDispatchDecision {
    RulePatcher {
        selected_rule_codes: Vec<String>,
        deferred_codes: Vec<String>,
        reason: String,
    },
    LlmPatcher {
        reason: String,
    },
}

/// 因 span 冲突被抑制的诊断记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressedDiagnostic {
    pub diagnostic_index: usize,
    pub code: String,
    pub reason: String,
}

/// 函数内规则诊断仲裁结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionRuleDiagnosticSet {
    pub selected_diagnostics: Vec<DiagnosticRef>,
    pub suppressed_diagnostics: Vec<SuppressedDiagnostic>,
    pub suppressed_reason: Option<String>,
}

/// 函数分发可观测报告项。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDispatchReportItem {
    pub function_id: String,
    pub decision: String,
    pub reason: String,
    pub error_code_counts: BTreeMap<String, usize>,
    pub implemented_rule_codes: Vec<String>,
    pub deferred_codes: Vec<String>,
    pub selected_diagnostic_indexes: Vec<usize>,
    pub suppressed_diagnostics: Vec<SuppressedDiagnostic>,
    pub low_value_status: LowValueStatus,
    pub low_value_reason: String,
    pub low_value_markers: Vec<String>,
}

#[derive(Debug, Clone)]
struct SpanWindow {
    file_path: PathBuf,
    byte_start: usize,
    byte_end: usize,
    has_primary: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionDispatchOutput {
    pub tasks: Vec<FunctionPatchTask>,
    pub decisions: BTreeMap<String, FunctionDispatchDecision>,
    pub rule_diagnostic_sets: BTreeMap<String, FunctionRuleDiagnosticSet>,
    pub report_items: Vec<FunctionDispatchReportItem>,
    pub target_function_ids: BTreeSet<String>,
    pub rule_function_ids: BTreeSet<String>,
    pub llm_routed_function_ids: BTreeSet<String>,
}

impl FunctionDispatchOutput {
    pub fn empty() -> Self {
        Self {
            tasks: Vec::new(),
            decisions: BTreeMap::new(),
            rule_diagnostic_sets: BTreeMap::new(),
            report_items: Vec::new(),
            target_function_ids: BTreeSet::new(),
            rule_function_ids: BTreeSet::new(),
            llm_routed_function_ids: BTreeSet::new(),
        }
    }
}

/// 构建函数任务、执行 RuleFirst 分发，并输出可观测报告。
pub fn build_dispatch_output(
    diagnostics: &[Diagnostic],
    crate_path: &Path,
    function_index: &FunctionIndex,
    implemented_codes: &BTreeSet<ErrorCode>,
) -> FunctionDispatchOutput {
    let tasks =
        build_function_patch_tasks(diagnostics, crate_path, function_index, implemented_codes);
    if tasks.is_empty() {
        return FunctionDispatchOutput::empty();
    }

    let mut decisions = BTreeMap::new();
    let mut rule_sets = BTreeMap::new();
    let mut report_items = Vec::new();
    let mut target_function_ids = BTreeSet::new();
    let mut rule_function_ids = BTreeSet::new();
    let mut llm_routed_function_ids = BTreeSet::new();

    for task in &tasks {
        target_function_ids.insert(task.function_id.clone());
        let decision = dispatch_task(task);
        let rule_set = match &decision {
            FunctionDispatchDecision::RulePatcher { .. } => {
                let set = arbitrate_rule_diagnostics(task, &decision);
                rule_function_ids.insert(task.function_id.clone());
                set
            }
            FunctionDispatchDecision::LlmPatcher { .. } => {
                llm_routed_function_ids.insert(task.function_id.clone());
                FunctionRuleDiagnosticSet {
                    selected_diagnostics: Vec::new(),
                    suppressed_diagnostics: Vec::new(),
                    suppressed_reason: None,
                }
            }
        };

        let report = build_report_item(task, &decision, &rule_set);
        decisions.insert(task.function_id.clone(), decision);
        rule_sets.insert(task.function_id.clone(), rule_set);
        report_items.push(report);
    }

    FunctionDispatchOutput {
        tasks,
        decisions,
        rule_diagnostic_sets: rule_sets,
        report_items,
        target_function_ids,
        rule_function_ids,
        llm_routed_function_ids,
    }
}

/// 按函数聚合 Error 级诊断，形成函数补丁任务。
pub fn build_function_patch_tasks(
    diagnostics: &[Diagnostic],
    crate_path: &Path,
    function_index: &FunctionIndex,
    implemented_codes: &BTreeSet<ErrorCode>,
) -> Vec<FunctionPatchTask> {
    let mut grouped: BTreeMap<String, Vec<DiagnosticRef>> = BTreeMap::new();

    for (idx, diagnostic) in diagnostics.iter().enumerate() {
        if !matches!(diagnostic.severity, Severity::Error) {
            continue;
        }
        let Some(span) = primary_span(diagnostic) else {
            continue;
        };
        let Some(function) = function_index.function_for_span(span, crate_path) else {
            continue;
        };

        let code = diagnostic
            .code
            .as_ref()
            .map(|value| value.code.to_string())
            .unwrap_or_else(|| "NO_CODE".to_string());

        grouped
            .entry(function.id.clone())
            .or_default()
            .push(DiagnosticRef {
                diagnostic_index: idx,
                code,
                diagnostic: diagnostic.clone(),
            });
    }

    let mut tasks = Vec::new();
    for (function_id, mut refs) in grouped {
        refs.sort_by_key(|item| item.diagnostic_index);
        let Some(function) = function_index.get(&function_id) else {
            continue;
        };

        let mut error_code_counts = BTreeMap::new();
        let mut implemented_rule_codes_present = BTreeSet::new();
        let mut unimplemented_codes_present = BTreeSet::new();

        for item in &refs {
            *error_code_counts.entry(item.code.clone()).or_insert(0) += 1;
            match item.diagnostic.code.as_ref() {
                Some(code) if implemented_codes.contains(&code.code) => {
                    implemented_rule_codes_present.insert(code.code);
                }
                Some(code) => {
                    unimplemented_codes_present.insert(code.code.to_string());
                }
                None => {
                    unimplemented_codes_present.insert("NO_CODE".to_string());
                }
            }
        }

        let low_value = analyze_test_function_low_value(function);

        tasks.push(FunctionPatchTask {
            function_id,
            file_path: function.file_path.clone(),
            function_line_span: (function.line_start, function.line_end),
            diagnostics_with_index: refs,
            error_code_counts,
            implemented_rule_codes_present,
            unimplemented_codes_present,
            low_value_status: low_value.status,
            low_value_reason: low_value.reason,
            low_value_markers: low_value.markers,
        });
    }

    tasks
}

/// RuleFirst：存在已实现规则错误码则优先规则 patcher，否则直接路由 LLM。
pub fn dispatch_task(task: &FunctionPatchTask) -> FunctionDispatchDecision {
    if task.implemented_rule_codes_present.is_empty() {
        return FunctionDispatchDecision::LlmPatcher {
            reason: "no implemented rule patcher code in function diagnostics".to_string(),
        };
    }

    let mut selected_rule_codes = task
        .implemented_rule_codes_present
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    selected_rule_codes.sort();

    let mut deferred_codes = task
        .unimplemented_codes_present
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    deferred_codes.sort();

    FunctionDispatchDecision::RulePatcher {
        selected_rule_codes,
        deferred_codes,
        reason: "rule-first: function contains implemented rule codes".to_string(),
    }
}

/// 场景 A/B 仲裁：span 非重叠追加，span 重叠按主错误优先。
pub fn arbitrate_rule_diagnostics(
    task: &FunctionPatchTask,
    decision: &FunctionDispatchDecision,
) -> FunctionRuleDiagnosticSet {
    let FunctionDispatchDecision::RulePatcher {
        selected_rule_codes,
        ..
    } = decision
    else {
        return FunctionRuleDiagnosticSet {
            selected_diagnostics: Vec::new(),
            suppressed_diagnostics: Vec::new(),
            suppressed_reason: None,
        };
    };

    let selected_code_set: BTreeSet<String> = selected_rule_codes.iter().cloned().collect();
    let mut candidates = task
        .diagnostics_with_index
        .iter()
        .filter(|item| selected_code_set.contains(&item.code))
        .cloned()
        .collect::<Vec<_>>();

    if candidates.len() <= 1 {
        return FunctionRuleDiagnosticSet {
            selected_diagnostics: candidates,
            suppressed_diagnostics: Vec::new(),
            suppressed_reason: None,
        };
    }

    candidates.sort_by_key(|item| item.diagnostic_index);

    let mut parent: Vec<usize> = (0..candidates.len()).collect();
    for i in 0..candidates.len() {
        for j in (i + 1)..candidates.len() {
            if diagnostics_overlap(&candidates[i].diagnostic, &candidates[j].diagnostic) {
                union(&mut parent, i, j);
            }
        }
    }

    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for idx in 0..candidates.len() {
        let root = find(&mut parent, idx);
        groups.entry(root).or_default().push(idx);
    }

    let mut selected_positions = BTreeSet::new();
    let mut suppressed = Vec::new();

    for indexes in groups.values() {
        let mut group_sorted = indexes.clone();
        group_sorted.sort_by(|a, b| {
            let aw = span_window(&candidates[*a].diagnostic);
            let bw = span_window(&candidates[*b].diagnostic);
            // 主 span 优先，其次按诊断原始顺序。
            let a_key = (
                !aw.as_ref().map(|w| w.has_primary).unwrap_or(false),
                candidates[*a].diagnostic_index,
            );
            let b_key = (
                !bw.as_ref().map(|w| w.has_primary).unwrap_or(false),
                candidates[*b].diagnostic_index,
            );
            a_key.cmp(&b_key)
        });

        if let Some(&winner) = group_sorted.first() {
            selected_positions.insert(winner);
            for &loser in group_sorted.iter().skip(1) {
                suppressed.push(SuppressedDiagnostic {
                    diagnostic_index: candidates[loser].diagnostic_index,
                    code: candidates[loser].code.clone(),
                    reason: "span_overlap_primary_wins".to_string(),
                });
            }
        }
    }

    let mut selected = Vec::new();
    for (idx, candidate) in candidates.into_iter().enumerate() {
        if selected_positions.contains(&idx) {
            selected.push(candidate);
        }
    }
    selected.sort_by_key(|item| item.diagnostic_index);
    suppressed.sort_by_key(|item| item.diagnostic_index);

    FunctionRuleDiagnosticSet {
        selected_diagnostics: selected,
        suppressed_diagnostics: suppressed.clone(),
        suppressed_reason: if suppressed.is_empty() {
            None
        } else {
            Some("span_overlap_primary_wins".to_string())
        },
    }
}

fn build_report_item(
    task: &FunctionPatchTask,
    decision: &FunctionDispatchDecision,
    rule_set: &FunctionRuleDiagnosticSet,
) -> FunctionDispatchReportItem {
    match decision {
        FunctionDispatchDecision::RulePatcher {
            selected_rule_codes,
            deferred_codes,
            reason,
        } => FunctionDispatchReportItem {
            function_id: task.function_id.clone(),
            decision: "RulePatcher".to_string(),
            reason: reason.clone(),
            error_code_counts: task.error_code_counts.clone(),
            implemented_rule_codes: selected_rule_codes.clone(),
            deferred_codes: deferred_codes.clone(),
            selected_diagnostic_indexes: rule_set
                .selected_diagnostics
                .iter()
                .map(|item| item.diagnostic_index)
                .collect(),
            suppressed_diagnostics: rule_set.suppressed_diagnostics.clone(),
            low_value_status: task.low_value_status.clone(),
            low_value_reason: task.low_value_reason.clone(),
            low_value_markers: task.low_value_markers.clone(),
        },
        FunctionDispatchDecision::LlmPatcher { reason } => FunctionDispatchReportItem {
            function_id: task.function_id.clone(),
            decision: "LlmPatcher".to_string(),
            reason: reason.clone(),
            error_code_counts: task.error_code_counts.clone(),
            implemented_rule_codes: Vec::new(),
            deferred_codes: task.unimplemented_codes_present.iter().cloned().collect(),
            selected_diagnostic_indexes: Vec::new(),
            suppressed_diagnostics: Vec::new(),
            low_value_status: task.low_value_status.clone(),
            low_value_reason: task.low_value_reason.clone(),
            low_value_markers: task.low_value_markers.clone(),
        },
    }
}

fn diagnostics_overlap(left: &Diagnostic, right: &Diagnostic) -> bool {
    let Some(a) = span_window(left) else {
        return false;
    };
    let Some(b) = span_window(right) else {
        return false;
    };

    if a.file_path != b.file_path {
        return false;
    }

    a.byte_start < b.byte_end && b.byte_start < a.byte_end
}

fn span_window(diagnostic: &Diagnostic) -> Option<SpanWindow> {
    let primary = diagnostic.span.iter().find(|span| span.is_primary);
    let selected = primary.or_else(|| diagnostic.span.first())?;
    Some(SpanWindow {
        file_path: selected.file_path.clone(),
        byte_start: selected.byte_start,
        byte_end: selected.byte_end,
        has_primary: primary.is_some(),
    })
}

fn primary_span(diagnostic: &Diagnostic) -> Option<&SpanInfo> {
    diagnostic
        .span
        .iter()
        .find(|span| span.is_primary)
        .or_else(|| diagnostic.span.first())
}

fn find(parent: &mut [usize], x: usize) -> usize {
    if parent[x] == x {
        return x;
    }
    let root = find(parent, parent[x]);
    parent[x] = root;
    root
}

fn union(parent: &mut [usize], a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra != rb {
        parent[rb] = ra;
    }
}

#[cfg(test)]
mod tests;
