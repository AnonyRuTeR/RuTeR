use crate::core::{Diagnostic, ErrorCode, FixAction, Result};

/// Uni API for all patchers
///
/// Every error patcher must implement this trait
/// Use strategy pattern to dynamically select the appropriate patcher
/// To be extended to support more error types
pub trait Patcher {
    /// Returns the error code this patcher can handle
    fn error_code(&self) -> ErrorCode;

    /// Judge if this patcher can handle the given diagnostic
    fn can_handle(&self, diagnostic: &Diagnostic) -> bool {
        diagnostic
            .code
            .as_ref()
            .map(|c| c.code == self.error_code())
            .unwrap_or(false)
    }

    /// Analyze the diagnostic and generate fix actions
    ///
    /// Returns:
    /// - Ok(Vec<FixAction>) if fix actions are generated successfully
    /// - Err(RuTeRError) if any error occurs during analysis or generation
    fn analyze(&self, diagnostic: &Diagnostic) -> Result<Vec<FixAction>>;

    /// Returns a brief description of the patcher
    fn description(&self) -> &'static str;
}

pub mod registry;
pub use registry::PatcherRegistry;

pub mod common;

pub mod e0433;
pub use e0433::E0433Patcher;

pub mod e0599;
pub use e0599::E0599Patcher;

pub mod e0308;
pub use e0308::E0308Patcher;

pub mod e0432;
pub use e0432::E0432Patcher;

pub mod e0560;
pub use e0560::E0560Patcher;
