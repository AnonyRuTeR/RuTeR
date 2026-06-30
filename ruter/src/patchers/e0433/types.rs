/// Candidate fix method
///
/// Extracted from compiler output diagnostics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateFix {
    /// The bare identifier extracted from compiler diagnostics (e.g., "HashMap", "serde", "Result")
    ///
    /// # Semantic Constraint
    /// This field MUST always contain the bare identifier (simple name) without any path qualifiers.
    /// Examples:
    /// - For "HashMap" -> "HashMap"
    /// - For path "std::collections::HashMap" -> "HashMap"
    /// - For "serde" -> "serde"
    ///
    /// # Note
    /// Currently not used by PathResolver, but retained for:
    /// 1. Future smart path resolution (identifier-to-path matching)
    /// 2. Debugging and user-facing output
    /// 3. Handling ambiguous candidates with user interaction
    pub bare_identifier: String,
    /// Complete import path suggested by the compiler (e.g., "std::collections::HashMap")
    /// Which can be used to add or replace, with the identifier
    pub suggested_path: Option<String>,
    /// Confidence level of this fix suggestion
    pub confidence: Confidence,
    /// Candidate category used by sanitizer/resolver constraints.
    pub kind: CandidateKind,
    /// Why this candidate cannot be executed.
    pub blocked_reason: Option<String>,
}

/// Confidence level of the candidate fix
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

/// Candidate category for E0433 decision chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateKind {
    PathRewrite,
    UseImport,
    DependencyAction,
    NonCodeHint,
}

impl CandidateFix {
    /// Whether this candidate can be consumed by resolver/fix planner.
    pub fn is_executable(&self) -> bool {
        self.kind != CandidateKind::NonCodeHint && self.suggested_path.is_some()
    }
}

/// Path structure category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathStructure {
    /// Single identifier path. E.g., `State`
    SingleIdent,
    /// Module path. E.g., `std::collections::HashMap`
    ModuleAccess,
    /// Relevant method call path. E.g., `State::new()`
    MethodCall,
    /// Multiple nested paths. E.g., `a::b::c::Item`
    NestedPath,
}

/// Path Segment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSegment {
    pub name: String,
    pub byte_range: std::ops::Range<usize>,
}

/// Extended span analysis result
///
/// The span reported by the compiler may only cover part of the relevant code.
/// May need to be extended to cover the full expression.
#[derive(Debug, Clone)]
pub struct ExtendedSpan {
    /// Original span from the compiler
    pub original_span: crate::core::SpanInfo,
    /// Extended span covering the full relevant code
    pub extended_span: crate::core::SpanInfo,
    /// Different parts of the path. E.g., `["specialized", "State"]`
    pub segments: Vec<PathSegment>,
    /// Structure of the path
    pub structure: PathStructure,
}
