use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use crate::core::{Diagnostic, ErrorCode, FixAction, Result, RuTeRError};
use crate::patchers::PatcherRegistry;
use crate::patchers::e0433::{E0433Patcher, RankedFixAction};
use crate::transformer::CodeTransformer;
use serde::{Deserialize, Serialize};

/// Coordinates patch planning and application across files.
pub struct PatchCoordinator {
    registry: PatcherRegistry,
    transformer: CodeTransformer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCandidatePlan {
    pub candidate_id: String,
    pub score: i32,
    pub plan: BTreeMap<PathBuf, Vec<FixAction>>,
}

#[derive(Debug, Clone)]
struct BeamState {
    score: i32,
    plan: BTreeMap<PathBuf, Vec<FixAction>>,
}

impl PatchCoordinator {
    pub fn new(registry: PatcherRegistry, transformer: CodeTransformer) -> Self {
        Self {
            registry,
            transformer,
        }
    }

    /// Build a conflict-free action plan grouped by target file.
    ///
    /// # Arguments
    /// - &self
    /// - `diagnostics`: List of compiler diagnostics to analyze
    /// # Returns
    /// - `Result<BTreeMap<PathBuf, Vec<FixAction>>>`: Mapping of file paths to their respective fix actions
    pub fn plan(&self, diagnostics: &[Diagnostic]) -> Result<BTreeMap<PathBuf, Vec<FixAction>>> {
        let mut grouped: BTreeMap<PathBuf, Vec<FixAction>> = BTreeMap::new();

        // Transform diagnostics into fix actions grouped by file
        for diagnostic in diagnostics {
            // Find suitable patcher for the specified diagnostic
            let Some(patcher) = self.registry.find_patcher(diagnostic) else {
                continue;
            };

            // Analyze diagnostic to get fix actions
            let actions = patcher.analyze(diagnostic)?;
            for action in actions {
                let file = Self::action_file_path(&action).ok_or_else(|| {
                    RuTeRError::ParseError(
                        "fix action missing file path information".to_string(),
                    )
                })?;
                grouped.entry(file).or_default().push(action);
            }
        }

        // Ensure no conflicts within each file's actions
        for actions in grouped.values_mut() {
            actions.sort_by(|a, b| {
                let (a_start, a_end) = Self::action_range(a);
                let (b_start, b_end) = Self::action_range(b);
                a_start.cmp(&b_start).then_with(|| a_end.cmp(&b_end))
            });
            CodeTransformer::ensure_no_conflicts(actions)?;
        }

        Ok(grouped)
    }

    /// Build top-k global candidate plans for E0433 repair.
    ///
    /// This method uses beam search across diagnostics and keeps at most `k` plans.
    pub fn plan_top_k(
        &self,
        diagnostics: &[Diagnostic],
        k: usize,
    ) -> Result<Vec<GlobalCandidatePlan>> {
        if k == 0 {
            return Ok(Vec::new());
        }

        let e0433 = E0433Patcher::new();
        let mut beam = vec![BeamState {
            score: 0,
            plan: BTreeMap::new(),
        }];

        for diagnostic in diagnostics {
            if diagnostic
                .code
                .as_ref()
                .map(|code| code.code == ErrorCode::E0433)
                .unwrap_or(false)
            {
                let ranked_actions = e0433.analyze_top_k(diagnostic, k)?;
                if ranked_actions.is_empty() {
                    continue;
                }
                beam = Self::expand_beam_with_ranked_actions(beam, ranked_actions, k)?;
                if beam.is_empty() {
                    break;
                }
                continue;
            }

            let Some(patcher) = self.registry.find_patcher(diagnostic) else {
                continue;
            };
            let actions = patcher.analyze(diagnostic)?;
            if actions.is_empty() {
                continue;
            }

            for state in &mut beam {
                for action in &actions {
                    Self::insert_action_checked(&mut state.plan, action.clone())?;
                }
            }
        }

        beam.retain(|state| !state.plan.is_empty());
        if beam.is_empty() {
            return Ok(Vec::new());
        }

        beam.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| Self::plan_sort_key(&a.plan).cmp(&Self::plan_sort_key(&b.plan)))
        });

        Ok(beam
            .into_iter()
            .take(k)
            .enumerate()
            .map(|(idx, state)| GlobalCandidatePlan {
                candidate_id: format!("candidate-{}", idx + 1),
                score: state.score,
                plan: state.plan,
            })
            .collect())
    }

    /// Apply planned actions using preloaded source text per file.
    pub fn apply_planned(
        &self,
        plan: &BTreeMap<PathBuf, Vec<FixAction>>,
        sources: &HashMap<PathBuf, String>,
    ) -> Result<HashMap<PathBuf, String>> {
        let mut outputs = HashMap::new();

        // Apply actions file by file
        for (file, actions) in plan {
            let source = sources
                .get(file)
                .ok_or_else(|| RuTeRError::SourceFileNotFound(file.display().to_string()))?;
            let updated = self.transformer.apply_replacements(source, actions)?;
            outputs.insert(file.clone(), updated);
        }

        Ok(outputs)
    }

    fn action_file_path(action: &FixAction) -> Option<PathBuf> {
        match action {
            FixAction::Insert { span, .. }
            | FixAction::Replace { span, .. }
            | FixAction::Delete { span } => Some(span.file_path.clone()),
        }
    }

    fn action_range(action: &FixAction) -> (usize, usize) {
        match action {
            FixAction::Insert { span, .. }
            | FixAction::Replace { span, .. }
            | FixAction::Delete { span } => (span.byte_start, span.byte_end),
        }
    }

    fn expand_beam_with_ranked_actions(
        beam: Vec<BeamState>,
        ranked_actions: Vec<RankedFixAction>,
        k: usize,
    ) -> Result<Vec<BeamState>> {
        let mut expanded = Vec::new();

        for state in beam {
            for ranked in &ranked_actions {
                let mut next = state.clone();
                match Self::insert_action_checked(&mut next.plan, ranked.action.clone()) {
                    Ok(()) => {}
                    Err(RuTeRError::ConflictingFixActions(_)) => {
                        continue;
                    }
                    Err(err) => return Err(err),
                }
                next.score += ranked.score;
                expanded.push(next);
            }
        }

        expanded.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| Self::plan_sort_key(&a.plan).cmp(&Self::plan_sort_key(&b.plan)))
        });
        expanded.truncate(k);
        Ok(expanded)
    }

    fn insert_action_checked(
        plan: &mut BTreeMap<PathBuf, Vec<FixAction>>,
        action: FixAction,
    ) -> Result<()> {
        let file = Self::action_file_path(&action).ok_or_else(|| {
            RuTeRError::ParseError("fix action missing file path information".to_string())
        })?;
        let actions = plan.entry(file).or_default();
        actions.push(action);
        actions.sort_by(|a, b| {
            let (a_start, a_end) = Self::action_range(a);
            let (b_start, b_end) = Self::action_range(b);
            a_start.cmp(&b_start).then_with(|| a_end.cmp(&b_end))
        });
        CodeTransformer::ensure_no_conflicts(actions)?;
        Ok(())
    }

    fn plan_sort_key(plan: &BTreeMap<PathBuf, Vec<FixAction>>) -> String {
        let mut parts = Vec::new();
        for (file, actions) in plan {
            let mut action_tokens = Vec::new();
            for action in actions {
                let (start, end) = Self::action_range(action);
                let token = match action {
                    FixAction::Replace { new_content, .. } => {
                        format!("R:{start}:{end}:{new_content}")
                    }
                    FixAction::Insert { content, .. } => format!("I:{start}:{end}:{content}"),
                    FixAction::Delete { .. } => format!("D:{start}:{end}"),
                };
                action_tokens.push(token);
            }
            parts.push(format!("{}|{}", file.display(), action_tokens.join(",")));
        }
        parts.join(";")
    }
}

#[cfg(test)]
mod tests;
