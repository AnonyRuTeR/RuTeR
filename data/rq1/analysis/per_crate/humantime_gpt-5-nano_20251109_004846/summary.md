# Analysis Report: humantime_gpt-5-nano_20251109_004846

## 1. Executive Summary
- Total Samples: 153
- Success: 22 (14.4%)
- Failures: 131 (85.6%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 120 | 91.6% |
| OTHER_FAILURE | 6 | 4.6% |
| UNSTABLE_FEATURE(E0658) | 5 | 3.8% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 103 | Failed to resolve import |
| E0308 | 10 | Type mismatch |
| E0061 | 8 | Wrong number of function arguments |
| E0599 | 8 | Method/field not found |
| E0658 | 5 | Unstable feature |
| E0283 | 5 |  |
| E0107 | 4 |  |
| E0412 | 2 | Cannot find type in this scope |
| E0261 | 1 |  |
| E0063 | 1 | Missing struct fields |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | OTHER_FAILURE | Compilation failed without specific error code |
| duration::item_plural | OTHER_FAILURE | Compilation failed without specific error code |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | OTHER_FAILURE | Compilation failed without specific error code |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0261, E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | OTHER_FAILURE | Compilation failed without specific error code |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308, E0061 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308, E0107, E0061 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Type mismatch errors: E0308, E0107, E0061 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Method/field not found: E0107, E0061, E0599 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | OTHER_FAILURE | Compilation failed without specific error code |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0107, E0599 |
| duration::Parser::<'a>::parse | OTHER_FAILURE | Compilation failed without specific error code |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Unresolved name/path: E0425, E0412 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Type mismatch errors: E0308 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Type mismatch errors: E0308 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Type mismatch errors: E0308 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
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
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Type mismatch errors: E0308 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Type mismatch errors: E0308 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433, E0412 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
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
| <wrapper::Duration as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0283, E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0283, E0433 |
| <wrapper::Duration as std::convert::Into<std::time::Duration>>::into | RUSTC_ERROR | Failed to resolve import: E0283, E0433 |
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
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0283, E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0283, E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
