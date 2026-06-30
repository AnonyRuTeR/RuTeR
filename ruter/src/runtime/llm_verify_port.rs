use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use ruter::core::FixAction;

use crate::llm::executor::verify_port::LlmVerifyPort;
use crate::runtime::artifacts::ArtifactPaths;
use crate::runtime::reporter::Reporter;
use crate::runtime::stages::{PartialUnionVerifyResult, verify_partial_union_plan_with_tag};

pub struct RuntimeLlmVerifyPort;

impl LlmVerifyPort for RuntimeLlmVerifyPort {
    fn verify_partial_union_plan_with_tag(
        &self,
        crate_path: &Path,
        plan: &BTreeMap<PathBuf, Vec<FixAction>>,
        target_function_ids: &BTreeSet<String>,
        artifacts: &ArtifactPaths,
        keep_updated_sources: bool,
        tag: &str,
        reporter: &mut Reporter,
    ) -> Result<PartialUnionVerifyResult> {
        verify_partial_union_plan_with_tag(
            crate_path,
            plan,
            target_function_ids,
            artifacts,
            keep_updated_sources,
            tag,
            reporter,
        )
    }
}
