use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Error codes by the compiler.
#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    EnumString,
    Display,
)]
pub enum ErrorCode {
    E0433,
    E0432,
    E0599,
    E0308,
    E0560,
    E0425,
    E0382,
    /// Fallback variant for non-rustc-E* diagnostic codes,
    /// e.g. warning code `unexpected_cfgs`.
    #[serde(other)]
    Unknown,
}
