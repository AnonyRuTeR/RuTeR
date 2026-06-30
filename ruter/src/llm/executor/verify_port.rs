use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use ruter::core::FixAction;

use crate::runtime::artifacts::ArtifactPaths;
use crate::runtime::reporter::Reporter;
use crate::runtime::stages::PartialUnionVerifyResult;

/// Port for LLM-stage verify calls.
/// Runtime provides the adapter so LLM code does not depend on verify implementation details.
pub trait LlmVerifyPort {
    fn verify_partial_union_plan_with_tag(
        &self,
        crate_path: &Path,
        plan: &BTreeMap<PathBuf, Vec<FixAction>>,
        target_function_ids: &BTreeSet<String>,
        artifacts: &ArtifactPaths,
        keep_updated_sources: bool,
        tag: &str,
        reporter: &mut Reporter,
    ) -> Result<PartialUnionVerifyResult>;
}
