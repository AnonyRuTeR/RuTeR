# Analysis Report: humantime_gpt-3.5-turbo_20251109_000031

## 1. Executive Summary
- Total Samples: 157
- Success: 19 (12.1%)
- Failures: 138 (87.9%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 123 | 89.1% |
| UNSTABLE_FEATURE(E0658) | 12 | 8.7% |
| OTHER_FAILURE | 3 | 2.2% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 93 | Failed to resolve import |
| E0432 | 35 | Unresolved import |
| E0308 | 29 | Type mismatch |
| E0599 | 22 | Method/field not found |
| E0063 | 14 | Missing struct fields |
| E0261 | 14 |  |
| E0658 | 12 | Unstable feature |
| E0061 | 11 | Wrong number of function arguments |
| E0252 | 7 |  |
| E0412 | 5 | Cannot find type in this scope |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| duration::parse_duration | RUSTC_ERROR | Compiler errors: E0432 |
| duration::parse_duration | RUSTC_ERROR | Compiler errors: E0432 |
| duration::parse_duration | RUSTC_ERROR | Compiler errors: E0432 |
| duration::parse_duration | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::parse_duration | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| duration::parse_duration | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| duration::format_duration | RUSTC_ERROR | Compiler errors: E0432 |
| duration::format_duration | RUSTC_ERROR | Failed to resolve import: E0433, E0252 |
| duration::item_plural | RUSTC_ERROR | Compiler errors: E0432 |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | RUSTC_ERROR | Type mismatch errors: E0308 |
| duration::item_plural | RUSTC_ERROR | Type mismatch errors: E0308 |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| <u64 as duration::OverflowOp>::add | RUSTC_ERROR | Failed to resolve import: E0433 |
| <u64 as duration::OverflowOp>::add | RUSTC_ERROR | Failed to resolve import: E0433 |
| <u64 as duration::OverflowOp>::add | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0412, E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0432, E0412 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0432, E0412 |
| duration::Parser::<'a>::off | OTHER_FAILURE | Compilation failed without specific error code |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0432, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0063, E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0433 |
| duration::Parser::<'a>::parse_first_char | OTHER_FAILURE | Compilation failed without specific error code |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0586 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Type mismatch errors: E0308, E0261, E0586 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0261, E0586, E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Type mismatch errors: E0308, E0261, E0433, E0061 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0432 |
| duration::Parser::<'a>::parse_first_char | OTHER_FAILURE | Compilation failed without specific error code |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0063, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308, E0261, E0433, E0061 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308, E0433, E0061 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308, E0261, E0433, E0061 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0261, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308, E0261, E0433, E0061 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0261, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Method/field not found: E0433, E0061, E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0432, E0261 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0261, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308, E0261, E0433, E0061 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308, E0261, E0433, E0061 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Unresolved name/path: E0063, E0425, E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0425, E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0063, E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0063, E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Type mismatch errors: E0308, E0261, E0433, E0061 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0425, E0261, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0432 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0432, E0433, E0252 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Compiler errors: E0432 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Type mismatch errors: E0432, E0308 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Compiler errors: E0432 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Compiler errors: E0432 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::format_rfc3339 | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::format_rfc3339 | RUSTC_ERROR | Failed to resolve import: E0433, E0252 |
| date::format_rfc3339 | RUSTC_ERROR | Compiler errors: E0432 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Unresolved name/path: E0432, E0425, E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433, E0061 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Compiler errors: E0432 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Compiler errors: E0432, E0252 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0603, E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0255, E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Method/field not found: E0599 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Method/field not found: E0599 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Method/field not found: E0599 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433, E0252 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Compiler errors: E0432, E0412, E0252 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Compiler errors: E0432, E0412, E0252 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0277, E0432, E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0277, E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
