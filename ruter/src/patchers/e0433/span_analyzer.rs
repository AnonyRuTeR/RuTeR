use crate::core::SpanInfo;
use crate::patchers::e0433::types::{ExtendedSpan, PathSegment, PathStructure};

/// Span Analyzer for E0433 Patcher
///
/// Extend span and parse path structure from compiler diagnostics.
pub struct SpanAnalyzer;

impl SpanAnalyzer {
    /// Main function to analyze and extend span
    ///
    /// # Arguments
    /// - `span`: Original span from the compiler
    /// - `source_code`: Full source code as string
    /// # Returns
    /// - `ExtendedSpan` with extended span and path analysis
    pub fn analyze(original_span: &SpanInfo, source_code: &str) -> ExtendedSpan {
        // Step 1: Extend the span to cover the full path
        let (extended_span, segments) = Self::extend_span(&original_span, source_code);

        // Step 2: Analyze the path structure
        let structure = Self::analyze_structure(&segments, source_code, extended_span.byte_end);
        ExtendedSpan {
            original_span: original_span.clone(),
            extended_span,
            segments,
            structure,
        }
    }

    /// Extend the given span to cover the complete path
    ///
    /// Scan forward from original_span.byte_end, recognize "::" + Identifier pattern
    /// # Arguments
    /// - `original_span`: Original span from the compiler
    /// - `source_code`: Full source code as string
    /// # Returns
    /// - `(extended_span, path_parts)`
    fn extend_span(original_span: &SpanInfo, source_code: &str) -> (SpanInfo, Vec<PathSegment>) {
        let mut segments = Vec::new();

        // Extract initial identifier within original span
        let first_text = &source_code[original_span.byte_start..original_span.byte_end];
        segments.push(PathSegment {
            name: first_text.to_string(),
            byte_range: original_span.byte_start..original_span.byte_end,
        });

        // Scan forward to find additional path parts
        let mut current_pos = original_span.byte_end;
        let bytes = source_code.as_bytes();
        loop {
            let mut temp_pos = current_pos;
            while temp_pos < bytes.len() && bytes[temp_pos].is_ascii_whitespace() {
                temp_pos += 1;
            }

            // Check for "::" separator
            if temp_pos + 1 >= bytes.len() || &bytes[temp_pos..temp_pos + 2] != b"::" {
                break;
            }
            temp_pos += 2;

            // Skip whitespace after "::"
            while temp_pos < bytes.len() && bytes[temp_pos].is_ascii_whitespace() {
                temp_pos += 1;
            }

            // Read the next identifier
            let segment_start = temp_pos;
            if let Some((name, len)) = read_identifier(source_code, temp_pos) {
                let segment_end = segment_start + len;
                segments.push(PathSegment {
                    name,
                    byte_range: segment_start..segment_end,
                });
                current_pos = segment_end;
            } else {
                break;
            }
        }

        // Calculate new col_end
        let appended_str = &source_code[original_span.byte_end..current_pos];
        // let line_diff = appended_str.chars().filter(|&c| c == '\n').count();
        let line_diff = appended_str.matches('\n').count();
        let new_col_end = if line_diff == 0 {
            original_span.col_end + appended_str.len()
        } else {
            appended_str.lines().last().unwrap_or(&"").len() + 1
        };

        // Construct the extended span
        let extended_span = SpanInfo {
            byte_end: current_pos,
            line_end: original_span.line_end + line_diff,
            col_end: new_col_end,
            ..original_span.clone()
        };

        (extended_span, segments)
    }

    /// Analyze the path structure
    fn analyze_structure(
        segments: &Vec<PathSegment>,
        source_code: &str,
        span_end: usize,
    ) -> PathStructure {
        if segments.is_empty() {
            return PathStructure::SingleIdent;
        }

        // If it's Method Call
        let rest_code = &source_code[span_end..];
        let next_char = rest_code.trim_start().chars().next();
        if next_char == Some('(') || next_char == Some('!') {
            return if segments.len() == 1 {
                PathStructure::SingleIdent
            } else {
                PathStructure::MethodCall
            };
        }

        // Determine based on number of parts
        if segments.len() == 1 {
            PathStructure::SingleIdent
        } else {
            match segments.len() {
                2..=3 => PathStructure::ModuleAccess,
                _ => PathStructure::NestedPath,
            }
        }
    }
}

/// Helper function:
///
/// Read Identifier starting at given byte index
/// # Arguments
/// - `source`: Full source code string
/// - `pos`: Byte index to start reading
/// # Returns
/// - `Some((identifier, len))` if an identifier is found
/// - `None` if no valid identifier is found
fn read_identifier(source: &str, pos: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();

    // Check the bounds
    if pos >= bytes.len() {
        return None;
    }

    // Check the first character
    let first_char = bytes[pos] as char;
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return None;
    }

    // Read until non-identifier character
    let mut end = pos + 1;
    while end < bytes.len() {
        let c = bytes[end] as char;
        if c.is_ascii_alphanumeric() || c == '_' {
            end += 1;
        } else {
            break;
        }
    }

    // Extract the identifier string
    let identifier = source[pos..end].to_string();
    Some((identifier, end - pos))
}

/// Unit tests for read_identifier
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_span(byte_start: usize, byte_end: usize) -> SpanInfo {
        SpanInfo {
            file_path: PathBuf::from("test.rs"),
            byte_start,
            byte_end,
            line_start: 1,
            line_end: 1,
            col_start: byte_start + 1,
            col_end: byte_end + 1,
            is_primary: true,
            text: vec![],
            label: None,
            suggested_replacement: None,
            suggestion_applicability: None,
            expansion: None,
        }
    }

    #[test]
    fn test_single_identifier() {
        let source = "let x = value;";
        //                  0123456789012345
        //  Span:                   ^^^^^ 位置 8-13
        let span = create_test_span(8, 13);
        let result = SpanAnalyzer::analyze(&span, source);

        let names: Vec<&str> = result.segments.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["value"]);
        assert!(matches!(result.structure, PathStructure::SingleIdent));
    }

    #[test]
    fn test_module_access() {
        let source = "let x = foo::Bar;";
        //                  0123456789012345678
        //  Span:                   ^^^ 位置 8-11
        let span = create_test_span(8, 11);
        let result = SpanAnalyzer::analyze(&span, source);

        let names: Vec<&str> = result.segments.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["foo", "Bar"]);
        assert!(matches!(result.structure, PathStructure::ModuleAccess));
    }

    #[test]
    fn test_method_call() {
        let source = "let x = State::new();";
        //                  0123456789012345678901
        //  Span:                   ^^^^^ 位置 8-13
        let span = create_test_span(8, 13);
        let result = SpanAnalyzer::analyze(&span, source);

        let names: Vec<&str> = result.segments.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["State", "new"]);
        assert!(matches!(result.structure, PathStructure::MethodCall));
    }

    #[test]
    fn test_nested_path() {
        let source = "let x = a::b::c::Item;";
        //                  01234567890123456789012
        //  Span:                   ^ 位置 8-9
        let span = create_test_span(8, 9);
        let result = SpanAnalyzer::analyze(&span, source);

        let names: Vec<&str> = result.segments.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["a", "b", "c", "Item"]);
        assert!(matches!(result.structure, PathStructure::NestedPath));
    }

    #[test]
    fn test_with_whitespace() {
        let source = "let x = foo :: Bar;";
        //                  01234567890123456789
        //  Span:                   ^^^ 位置 8-11
        let span = create_test_span(8, 11);
        let result = SpanAnalyzer::analyze(&span, source);

        let names: Vec<&str> = result.segments.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["foo", "Bar"]);
    }

    #[test]
    fn test_byte_ranges() {
        let source = "let x = foo::Bar;";
        //                  0123456789012345678
        //  Span:                   ^^^ 位置 8-11
        //  foo::Bar:               ^^^^^^^^ 位置 8-15
        let span = create_test_span(8, 11);
        let result = SpanAnalyzer::analyze(&span, source);

        assert_eq!(result.segments.len(), 2);
        // First segment: "foo" at 8..11
        assert_eq!(result.segments[0].name, "foo");
        assert_eq!(result.segments[0].byte_range, 8..11);
        // Second segment: "Bar" at 13..16
        assert_eq!(result.segments[1].name, "Bar");
        assert_eq!(result.segments[1].byte_range, 13..16);
    }
}
