# Analysis Report: humantime_deepseek-v3_20251109_140004

## 1. Executive Summary
- Total Samples: 121
- Success: 23 (19.0%)
- Failures: 98 (81.0%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 88 | 89.8% |
| UNSTABLE_FEATURE(E0658) | 9 | 9.2% |
| TRUNCATED_STRING | 1 | 1.0% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 59 | Failed to resolve import |
| E0599 | 16 | Method/field not found |
| E0063 | 13 | Missing struct fields |
| E0658 | 9 | Unstable feature |
| E0277 | 8 | Trait not implemented |
| E0425 | 4 | Unresolved name |
| E0560 | 3 | Unknown struct field |
| E0432 | 1 | Unresolved import |
| E0765 | 1 |  |

## 4. Truncation Issues
| Type | Count | Percentage |
|---|---|---|
| TRUNCATED_STRING | 1 | 1.0% |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | RUSTC_ERROR | Method/field not found: E0599 |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | RUSTC_ERROR | Method/field not found: E0599 |
| duration::item | RUSTC_ERROR | Method/field not found: E0599 |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::item | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0432, E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | TRUNCATED_STRING | Unclosed string detected. |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Unresolved name/path: E0425 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Unresolved name/path: E0425 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Unresolved name/path: E0425 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Unresolved name/path: E0425, E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
