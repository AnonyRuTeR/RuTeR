# Analysis Report: humantime_gemini-2.5-flash-nothinking_20251109_134926

## 1. Executive Summary
- Total Samples: 124
- Success: 31 (25.0%)
- Failures: 93 (75.0%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 82 | 88.2% |
| UNSTABLE_FEATURE(E0658) | 10 | 10.8% |
| TRUNCATED_STRING | 1 | 1.1% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 47 | Failed to resolve import |
| E0599 | 22 | Method/field not found |
| E0560 | 13 | Unknown struct field |
| E0658 | 10 | Unstable feature |
| E0063 | 4 | Missing struct fields |
| E0432 | 4 | Unresolved import |
| E0107 | 4 |  |
| E0116 | 3 |  |
| E0308 | 2 | Type mismatch |
| E0164 | 1 |  |

## 4. Truncation Issues
| Type | Count | Percentage |
|---|---|---|
| TRUNCATED_STRING | 1 | 1.1% |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::item_plural | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals |
| duration::item_plural | RUSTC_ERROR | Compiler errors: E0116 |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | RUSTC_ERROR | Failed to resolve import: E0116, E0433 |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | RUSTC_ERROR | Failed to resolve import: E0116, E0433 |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals, fmt_internals, fmt_internals |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0432, E0107, E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0107, E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0107, E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0164, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0107, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | TRUNCATED_STRING | Unclosed string detected. |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0533, E0599 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
