use crate::core::{Diagnostic, Result};
use serde_json;

// === JsonParser Implementations ===

pub struct JsonParser;

impl JsonParser {
    /// Parse JSON string from rustc and convert to Vec<Diagnostic>
    ///
    /// The rustc JSON output is newline-delimited JSON (NDJSON), where each line
    /// is a separate JSON object representing a diagnostic message.
    pub fn parse(json_str: &str) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        for line in json_str.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Directly deserialize into Diagnostic
            // serde will handle the field mapping based on annotations
            let diag: Diagnostic = serde_json::from_str(line)?;
            diagnostics.push(diag);
        }

        Ok(diagnostics)
    }
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ErrorCode, Severity};

    #[test]
    fn test_parse_single_valid_message() {
        let json = r#"{"$message_type":"diagnostic","message":"cannot find value `x` in this scope","code":{"code":"E0425"},"level":"error","spans":[{"file_name":"src/main.rs","byte_start":0,"byte_end":1,"line_start":10,"line_end":10,"column_start":5,"column_end":6,"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":null}"#;

        let result = JsonParser::parse(json).expect("Failed to parse JSON");
        assert_eq!(result.len(), 1);

        let diag = &result[0];
        assert_eq!(diag.code.as_ref().map(|c| c.code), Some(ErrorCode::E0425));
        assert_eq!(
            diag.code.as_ref().and_then(|c| c.raw_code.clone()),
            Some("E0425".to_string())
        );
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "cannot find value `x` in this scope");
        assert_eq!(diag.span.len(), 1);
    }

    #[test]
    fn test_parse_multiple_messages() {
        let json = r#"{"$message_type":"diagnostic","message":"error1","code":{"code":"E0433"},"level":"error","spans":[],"children":[],"rendered":null}
{"$message_type":"diagnostic","message":"error2","code":{"code":"E0308"},"level":"warning","spans":[],"children":[],"rendered":null}"#;

        let result = JsonParser::parse(json).expect("Failed to parse JSON");
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].code.as_ref().map(|c| c.code),
            Some(ErrorCode::E0433)
        );
        assert_eq!(
            result[0].code.as_ref().and_then(|c| c.raw_code.clone()),
            Some("E0433".to_string())
        );
        assert_eq!(
            result[1].code.as_ref().map(|c| c.code),
            Some(ErrorCode::E0308)
        );
        assert_eq!(
            result[1].code.as_ref().and_then(|c| c.raw_code.clone()),
            Some("E0308".to_string())
        );
    }

    #[test]
    fn test_parse_nested_diagnostics() {
        let json = r#"{"$message_type":"diagnostic","message":"parent error","code":{"code":"E0433"},"level":"error","spans":[],"children":[{"message":"child note","code":{"code":"E0425"},"level":"note","spans":[],"children":[],"rendered":null}],"rendered":null}"#;

        let result = JsonParser::parse(json).expect("Failed to parse JSON");
        assert_eq!(result.len(), 1);

        let parent = &result[0];
        assert_eq!(parent.code.as_ref().map(|c| c.code), Some(ErrorCode::E0433));
        assert_eq!(parent.children.len(), 1);
        assert_eq!(
            parent.children[0].code.as_ref().map(|c| c.code),
            Some(ErrorCode::E0425)
        );
    }

    #[test]
    fn test_message_without_code() {
        let json = r#"{"$message_type":"diagnostic","message":"some message without code","code":null,"level":"error","spans":[],"children":[],"rendered":null}"#;

        let result = JsonParser::parse(json).expect("Failed to parse JSON");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, None);
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = "{ invalid json }";
        let result = JsonParser::parse(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_suggestions() {
        let json = r#"{"$message_type":"diagnostic","message":"unused variable","code":{"code":"E0425"},"level":"warning","spans":[{"file_name":"src/main.rs","byte_start":10,"byte_end":15,"line_start":5,"line_end":5,"column_start":9,"column_end":14,"is_primary":true,"text":[{"text":"    let unused = 42;","highlight_start":9,"highlight_end":14}],"label":"help: if this is intentional, prefix it with an underscore","suggested_replacement":"_unused","suggestion_applicability":"MachineApplicable","expansion":null}],"children":[],"rendered":null}"#;

        let result = JsonParser::parse(json).expect("Failed to parse JSON");
        assert_eq!(result.len(), 1);

        let span = &result[0].span[0];
        assert_eq!(span.suggested_replacement, Some("_unused".to_string()));
    }

    #[test]
    fn test_parse_unknown_warning_code_does_not_fail() {
        let json = r#"{"$message_type":"diagnostic","message":"unexpected cfg","code":{"code":"unexpected_cfgs","explanation":null},"level":"warning","spans":[],"children":[],"rendered":null}"#;

        let result = JsonParser::parse(json).expect("Failed to parse JSON");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, Severity::Warning);
        assert_eq!(
            result[0].code.as_ref().map(|c| c.code),
            Some(ErrorCode::Unknown)
        );
        assert_eq!(
            result[0].code.as_ref().and_then(|c| c.raw_code.clone()),
            Some("unexpected_cfgs".to_string())
        );
    }
}
