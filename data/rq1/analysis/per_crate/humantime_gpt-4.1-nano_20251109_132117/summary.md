# Analysis Report: humantime_gpt-4.1-nano_20251109_132117

## 1. Executive Summary
- Total Samples: 165
- Success: 20 (12.1%)
- Failures: 145 (87.9%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 130 | 89.7% |
| UNSTABLE_FEATURE(E0658) | 12 | 8.3% |
| OTHER_FAILURE | 3 | 2.1% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 80 | Failed to resolve import |
| E0432 | 32 | Unresolved import |
| E0308 | 27 | Type mismatch |
| E0063 | 24 | Missing struct fields |
| E0560 | 14 | Unknown struct field |
| E0658 | 12 | Unstable feature |
| E0599 | 8 | Method/field not found |
| E0061 | 7 | Wrong number of function arguments |
| E0425 | 6 | Unresolved name |
| E0283 | 4 |  |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | RUSTC_ERROR | Method/field not found: E0599 |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | RUSTC_ERROR | Compiler errors: E0404 |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals |
| duration::item | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| <u64 as duration::OverflowOp>::mul | RUSTC_ERROR | Failed to resolve import: E0433 |
| <u64 as duration::OverflowOp>::mul | RUSTC_ERROR | Failed to resolve import: E0433 |
| <u64 as duration::OverflowOp>::mul | RUSTC_ERROR | Failed to resolve import: E0433 |
| <u64 as duration::OverflowOp>::add | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0432, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0432, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0432, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0432, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0425, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0432, E0412 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0432, E0592 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0063, E0432, E0308 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0063, E0432, E0308 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0063, E0432, E0308 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0261, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0432, E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0063, E0432, E0308 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Type mismatch errors: E0308, E0560, E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0063, E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0560, E0107 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Type mismatch errors: E0308, E0560, E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Type mismatch errors: E0063, E0432, E0308 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::FormattedDuration::get_ref | OTHER_FAILURE | Compilation failed without specific error code |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Unresolved name/path: E0425, E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Compiler errors: E0432 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Compiler errors: E0432 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Compiler errors: E0432 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Compiler errors: E0432 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Compiler errors: E0432 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Compiler errors: E0432 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Compiler errors: E0432 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::format_rfc3339 | RUSTC_ERROR | Compiler errors: E0432 |
| date::format_rfc3339 | RUSTC_ERROR | Compiler errors: E0432 |
| date::format_rfc3339 | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::format_rfc3339 | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::format_rfc3339_micros | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::format_rfc3339_micros | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Unresolved name/path: E0425, E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Unresolved name/path: E0425, E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Unresolved name/path: E0425, E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Compiler errors: E0432 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | OTHER_FAILURE | Compilation failed without specific error code |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Unresolved name/path: E0425, E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Compiler errors: E0283 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0283, E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0283, E0433 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Compiler errors: E0432 |
| <wrapper::Duration as std::convert::From<std::time::Duration>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Method/field not found: E0599 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | OTHER_FAILURE | Compilation failed without specific error code |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0283, E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
