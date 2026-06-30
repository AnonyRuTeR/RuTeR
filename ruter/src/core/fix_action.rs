use super::SpanInfo;
use serde::{Deserialize, Serialize};

/// Actions suggested by the compiler to fix errors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FixAction {
    Insert { span: SpanInfo, content: String },
    Replace { span: SpanInfo, new_content: String },
    Delete { span: SpanInfo },
}
