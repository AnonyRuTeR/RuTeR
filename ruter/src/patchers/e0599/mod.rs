pub mod analyzer;
pub mod patcher;

pub use analyzer::{E0599Analysis, E0599Classification, E0599Target, analyze_e0599_against_crate};
pub use patcher::E0599Patcher;
