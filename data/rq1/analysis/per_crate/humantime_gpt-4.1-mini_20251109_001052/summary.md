# Analysis Report: humantime_gpt-4.1-mini_20251109_001052

## 1. Executive Summary
- Total Samples: 139
- Success: 25 (18.0%)
- Failures: 114 (82.0%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 101 | 88.6% |
| UNSTABLE_FEATURE(E0658) | 12 | 10.5% |
| TRUNCATED_BRACES | 1 | 0.9% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 76 | Failed to resolve import |
| E0599 | 14 | Method/field not found |
| E0658 | 13 | Unstable feature |
| E0560 | 13 | Unknown struct field |
| E0063 | 7 | Missing struct fields |
| E0432 | 7 | Unresolved import |
| E0308 | 2 | Type mismatch |
| E0434 | 1 | Cannot capture dynamic environment |
| E0277 | 1 | Trait not implemented |
| E0412 | 1 | Cannot find type in this scope |

## 4. Truncation Issues
| Type | Count | Percentage |
|---|---|---|
| TRUNCATED_BRACES | 1 | 0.9% |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item_plural | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | RUSTC_ERROR | Method/field not found: E0599 |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::item | RUSTC_ERROR | Failed to resolve import: E0433, E0434 |
| duration::item | TRUNCATED_BRACES | Unbalanced braces: { (29) vs } (28) |
| duration::item | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Type mismatch errors: E0063, E0308 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::off | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0404 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0432, E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_first_char | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0034 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0063, E0432, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0432, E0560, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0063, E0432 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Compiler errors: E0063 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0560, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0063, E0433 |
| duration::Parser::<'a>::parse_unit | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Compiler errors: E0560 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::Parser::<'a>::parse | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| duration::FormattedDuration::get_ref | RUSTC_ERROR | Compiler errors: E0432 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::two_digits::two_digits_inner | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| date::Rfc3339Timestamp::get_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Duration as std::convert::AsRef<std::time::Duration>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
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
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::AsRef<std::time::SystemTime>>::as_ref | RUSTC_ERROR | Compiler errors: E0614 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::ops::Deref>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::Into<std::time::SystemTime>>::into | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::convert::From<std::time::SystemTime>>::from | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| <wrapper::Timestamp as std::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
