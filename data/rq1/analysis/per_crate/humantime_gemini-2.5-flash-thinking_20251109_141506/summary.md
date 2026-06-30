# Analysis Report: humantime_gemini-2.5-flash-thinking_20251109_141506

## 1. Executive Summary
- Total Samples: 109
- Success: 31 (28.4%)
- Failures: 78 (71.6%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 78 | 100.0% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0599 | 34 | Method/field not found |
| E0433 | 30 | Failed to resolve import |
| E0560 | 12 | Unknown struct field |
| E0432 | 11 | Unresolved import |
| E0063 | 2 | Missing struct fields |
| E0308 | 2 | Type mismatch |
| E0609 | 1 |  |
| E0107 | 1 |  |
| E0027 | 1 |  |
| E0026 | 1 |  |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0432 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0432 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0560, E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0609, E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0107, E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308, E0560, E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0560, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0560, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Method/field not found: E0560, E0433, E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0432, E0560, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0027, E0026, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| date::two_digits | RUSTC_ERROR | Compiler errors: E0432 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::format_rfc3339 | RUSTC_ERROR | Compiler errors: E0432 |
| date::format_rfc3339 | RUSTC_ERROR | Compiler errors: E0432 |
| date::format_rfc3339 | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Method/field not found: E0599 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
