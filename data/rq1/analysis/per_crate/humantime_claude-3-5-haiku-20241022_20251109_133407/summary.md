# Analysis Report: humantime_claude-3-5-haiku-20241022_20251109_133407

## 1. Executive Summary
- Total Samples: 157
- Success: 19 (12.1%)
- Failures: 138 (87.9%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 138 | 100.0% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 96 | Failed to resolve import |
| E0599 | 23 | Method/field not found |
| E0432 | 11 | Unresolved import |
| E0061 | 8 | Wrong number of function arguments |
| E0063 | 4 | Missing struct fields |
| E0308 | 3 | Type mismatch |
| E0560 | 2 | Unknown struct field |
| E0283 | 2 |  |
| E0624 | 1 |  |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| duration::item_plural | RUSTC_ERROR | Compiler errors: E0061 |
| duration::item_plural | RUSTC_ERROR | Compiler errors: E0624, E0061 |
| duration::item_plural | RUSTC_ERROR | Method/field not found: E0599 |
| duration::item_plural | RUSTC_ERROR | Method/field not found: E0599 |
| duration::item_plural | RUSTC_ERROR | Method/field not found: E0599 |
| duration::item_plural | RUSTC_ERROR | Method/field not found: E0599 |
| duration::item_plural | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::item_plural | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::item_plural | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::item | RUSTC_ERROR | Type mismatch errors: E0308, E0061 |
| duration::item | RUSTC_ERROR | Type mismatch errors: E0308, E0061 |
| duration::item | RUSTC_ERROR | Type mismatch errors: E0308, E0061 |
| duration::item | RUSTC_ERROR | Failed to resolve import: E0433, E0061 |
| duration::item | RUSTC_ERROR | Failed to resolve import: E0433, E0061 |
| duration::item | RUSTC_ERROR | Failed to resolve import: E0433, E0061 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0432 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0432 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0432 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0432 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0432 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0560 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Method/field not found: E0599 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Method/field not found: E0599 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Method/field not found: E0599 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Method/field not found: E0599 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Compiler errors: E0432 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Method/field not found: E0599 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Method/field not found: E0599 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Method/field not found: E0599 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Compiler errors: E0283 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Compiler errors: E0283 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
