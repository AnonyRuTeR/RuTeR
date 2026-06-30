# Analysis Report: rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158

## 1. Executive Summary
- Total Samples: 180
- Success: 12 (6.7%)
- Failures: 168 (93.3%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 143 | 85.1% |
| UNSTABLE_FEATURE(E0658) | 17 | 10.1% |
| TRUNCATED_BRACES | 4 | 2.4% |
| OTHER_FAILURE | 2 | 1.2% |
| TRUNCATED_STRING | 2 | 1.2% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 108 | Failed to resolve import |
| E0599 | 73 | Method/field not found |
| E0560 | 41 | Unknown struct field |
| E0432 | 31 | Unresolved import |
| E0308 | 26 | Type mismatch |
| E0277 | 18 | Trait not implemented |
| E0658 | 17 | Unstable feature |
| E0412 | 16 | Cannot find type in this scope |
| E0369 | 15 | Binary operation not supported |
| E0261 | 9 |  |

## 4. Truncation Issues
| Type | Count | Percentage |
|---|---|---|
| TRUNCATED_BRACES | 4 | 2.4% |
| TRUNCATED_STRING | 2 | 1.2% |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| legacy::demangle | RUSTC_ERROR | Method/field not found: E0599 |
| legacy::demangle | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::HexNibbles::<'s>::try_parse_str_chars::{closure#2}::{closure#0}::utf8_len_from_first_byte | RUSTC_ERROR | Compiler errors: E0432, E0223 |
| v0::HexNibbles::<'s>::try_parse_str_chars::{closure#2}::{closure#0}::utf8_len_from_first_byte | OTHER_FAILURE | Compilation failed without specific error code |
| v0::HexNibbles::<'s>::try_parse_str_chars::{closure#2}::{closure#0}::utf8_len_from_first_byte | OTHER_FAILURE | Compilation failed without specific error code |
| v0::Ident::<'s>::try_small_punycode_decode | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Ident::<'s>::try_small_punycode_decode | RUSTC_ERROR | Compiler errors: E0560 |
| v0::Ident::<'s>::try_small_punycode_decode | RUSTC_ERROR | Compiler errors: E0560 |
| v0::Ident::<'s>::try_small_punycode_decode | RUSTC_ERROR | Method/field not found: E0261, E0433, E0599 |
| v0::Ident::<'s>::try_small_punycode_decode | RUSTC_ERROR | Method/field not found: E0432, E0107, E0599, E0560, E0261 |
| v0::Ident::<'s>::try_small_punycode_decode | RUSTC_ERROR | Method/field not found: E0412, E0433, E0599 |
| v0::Ident::<'s>::try_small_punycode_decode | RUSTC_ERROR | Compiler errors: E0560 |
| v0::Ident::<'s>::punycode_decode | TRUNCATED_BRACES | Unbalanced braces: { (13) vs } (11) |
| v0::Ident::<'s>::punycode_decode | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Ident::<'s>::punycode_decode | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Ident::<'s>::punycode_decode | TRUNCATED_STRING | Unclosed char detected. |
| v0::HexNibbles::<'s>::try_parse_uint | RUSTC_ERROR | Method/field not found: E0599 |
| v0::HexNibbles::<'s>::try_parse_uint | RUSTC_ERROR | Method/field not found: E0599 |
| v0::HexNibbles::<'s>::try_parse_uint | RUSTC_ERROR | Method/field not found: E0599 |
| v0::HexNibbles::<'s>::try_parse_uint | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::HexNibbles::<'s>::try_parse_str_chars | RUSTC_ERROR | Failed to resolve import: E0412, E0433 |
| v0::HexNibbles::<'s>::try_parse_str_chars | RUSTC_ERROR | Type mismatch errors: E0308, E0412 |
| v0::HexNibbles::<'s>::try_parse_str_chars | RUSTC_ERROR | Failed to resolve import: E0412, E0433 |
| v0::HexNibbles::<'s>::try_parse_str_chars | RUSTC_ERROR | Failed to resolve import: E0412, E0433 |
| v0::Parser::<'s>::push_depth | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::push_depth | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::push_depth | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::push_depth | RUSTC_ERROR | Compiler errors: E0560 |
| v0::Parser::<'s>::push_depth | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::push_depth | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::push_depth | RUSTC_ERROR | Compiler errors: E0560 |
| v0::Parser::<'s>::pop_depth | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::pop_depth | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::pop_depth | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::pop_depth | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::peek | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::peek | RUSTC_ERROR | Method/field not found: E0261, E0599 |
| v0::Parser::<'s>::peek | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::peek | RUSTC_ERROR | Failed to resolve import: E0560, E0433 |
| v0::Parser::<'s>::eat | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::eat | RUSTC_ERROR | Failed to resolve import: E0261, E0433 |
| v0::Parser::<'s>::eat | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::eat | RUSTC_ERROR | Compiler errors: E0560 |
| v0::Parser::<'s>::next | RUSTC_ERROR | Failed to resolve import: E0261, E0433 |
| v0::Parser::<'s>::next | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::next | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::next | RUSTC_ERROR | Compiler errors: E0560 |
| v0::Parser::<'s>::hex_nibbles | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0277 |
| v0::Parser::<'s>::hex_nibbles | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0277 |
| v0::Parser::<'s>::hex_nibbles | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::hex_nibbles | RUSTC_ERROR | Type mismatch errors: E0063, E0308, E0277 |
| v0::Parser::<'s>::digit_10 | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::digit_10 | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::digit_10 | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::digit_10 | RUSTC_ERROR | Compiler errors: E0560 |
| v0::Parser::<'s>::digit_62 | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::digit_62 | RUSTC_ERROR | Failed to resolve import: E0261, E0433 |
| v0::Parser::<'s>::digit_62 | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::digit_62 | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::integer_62 | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::integer_62 | RUSTC_ERROR | Method/field not found: E0261, E0599 |
| v0::Parser::<'s>::integer_62 | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::integer_62 | TRUNCATED_STRING | Unclosed string detected. |
| v0::Parser::<'s>::opt_integer_62 | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::opt_integer_62 | RUSTC_ERROR | Method/field not found: E0432, E0261, E0599 |
| v0::Parser::<'s>::opt_integer_62 | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::opt_integer_62 | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::disambiguator | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::disambiguator | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| v0::Parser::<'s>::disambiguator | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::disambiguator | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::namespace | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| v0::Parser::<'s>::namespace | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::namespace | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Parser::<'s>::namespace | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| v0::Parser::<'s>::backref | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::ident | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::ident | RUSTC_ERROR | Type mismatch errors: E0277, E0308, E0560, E0261 |
| v0::Parser::<'s>::ident | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Parser::<'s>::ident | RUSTC_ERROR | Compiler errors: E0277 |
| v0::ParseError::message | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::eat | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::eat | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::eat | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::eat | RUSTC_ERROR | Type mismatch errors: E0432, E0599, E0308, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::eat | RUSTC_ERROR | Type mismatch errors: E0432, E0560, E0308, E0599 |
| v0::Printer::<'a, 'b, 's>::eat | RUSTC_ERROR | Type mismatch errors: E0412, E0599, E0308, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::eat | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| v0::Printer::<'a, 'b, 's>::skipping_printing | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| v0::Printer::<'a, 'b, 's>::skipping_printing | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| v0::Printer::<'a, 'b, 's>::skipping_printing | RUSTC_ERROR | Failed to resolve import: E0562, E0433 |
| v0::Printer::<'a, 'b, 's>::skipping_printing | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_backref | RUSTC_ERROR | Type mismatch errors: E0432, E0599, E0308, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::print_backref | RUSTC_ERROR | Type mismatch errors: E0277, E0432, E0599, E0308, E0369, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::print_backref | RUSTC_ERROR | Failed to resolve import: E0432, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::print_backref | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::pop_depth | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::pop_depth | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::pop_depth | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::pop_depth | RUSTC_ERROR | Method/field not found: E0599 |
| v0::Printer::<'a, 'b, 's>::print | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| v0::Printer::<'a, 'b, 's>::print | RUSTC_ERROR | Type mismatch errors: E0308, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::print | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals |
| v0::Printer::<'a, 'b, 's>::print | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::print_quoted_escaped_chars | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::print_quoted_escaped_chars | RUSTC_ERROR | Type mismatch errors: E0412, E0308, E0609, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::print_quoted_escaped_chars | RUSTC_ERROR | Failed to resolve import: E0412, E0433 |
| v0::Printer::<'a, 'b, 's>::print_quoted_escaped_chars | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::print_lifetime_from_index | RUSTC_ERROR | Failed to resolve import: E0277, E0560, E0369, E0433 |
| v0::Printer::<'a, 'b, 's>::print_lifetime_from_index | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals, fmt_internals, fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_lifetime_from_index | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_lifetime_from_index | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| v0::Printer::<'a, 'b, 's>::in_binder | RUSTC_ERROR | Failed to resolve import: E0277, E0560, E0369, E0433 |
| v0::Printer::<'a, 'b, 's>::in_binder | RUSTC_ERROR | Failed to resolve import: E0277, E0432, E0369, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::in_binder | RUSTC_ERROR | Method/field not found: E0277, E0599, E0609, E0369, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::in_binder | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_sep_list | RUSTC_ERROR | Method/field not found: E0432, E0599, E0609, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::print_sep_list | RUSTC_ERROR | Type mismatch errors: E0308, E0560, E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_sep_list | RUSTC_ERROR | Type mismatch errors: E0433, E0432, E0599, E0308, E0609, E0560, E0412 |
| v0::Printer::<'a, 'b, 's>::print_sep_list | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| v0::Printer::<'a, 'b, 's>::print_path | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_path | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_path | RUSTC_ERROR | Type mismatch errors: E0308, E0560, E0599 |
| v0::Printer::<'a, 'b, 's>::print_path | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_generic_arg | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_generic_arg | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_generic_arg | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::print_generic_arg | RUSTC_ERROR | Failed to resolve import: E0609, E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::print_type | TRUNCATED_BRACES | Unbalanced braces: { (21) vs } (19) |
| v0::Printer::<'a, 'b, 's>::print_type | TRUNCATED_BRACES | Unbalanced braces: { (56) vs } (53) |
| v0::Printer::<'a, 'b, 's>::print_type | RUSTC_ERROR | Type mismatch errors: E0432, E0560, E0433, E0308 |
| v0::Printer::<'a, 'b, 's>::print_type | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_path_maybe_open_generics | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::print_path_maybe_open_generics | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_path_maybe_open_generics | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_path_maybe_open_generics | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_dyn_trait | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::print_dyn_trait | RUSTC_ERROR | Failed to resolve import: E0560, E0433 |
| v0::Printer::<'a, 'b, 's>::print_dyn_trait | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::print_dyn_trait | RUSTC_ERROR | Method/field not found: E0560, E0599 |
| v0::Printer::<'a, 'b, 's>::print_pat | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| v0::Printer::<'a, 'b, 's>::print_pat | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_pat | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_pat | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_const | RUSTC_ERROR | Failed to resolve import: E0412, E0433 |
| v0::Printer::<'a, 'b, 's>::print_const | TRUNCATED_BRACES | Unbalanced braces: { (26) vs } (25) |
| v0::Printer::<'a, 'b, 's>::print_const | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::print_const | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::print_const_uint | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals, fmt_internals, fmt_internals, fm... |
| v0::Printer::<'a, 'b, 's>::print_const_uint | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_const_uint | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_const_uint | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_const_str_literal | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals |
| v0::Printer::<'a, 'b, 's>::print_const_str_literal | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| v0::Printer::<'a, 'b, 's>::print_const_str_literal | RUSTC_ERROR | Failed to resolve import: E0433 |
| v0::Printer::<'a, 'b, 's>::print_const_str_literal | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| Demangle::<'a>::as_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| Demangle::<'a>::as_str | RUSTC_ERROR | Compiler errors: E0432 |
| Demangle::<'a>::as_str | RUSTC_ERROR | Compiler errors: E0432 |
| Demangle::<'a>::as_str | RUSTC_ERROR | Method/field not found: E0560, E0599 |
| Demangle::<'a>::as_str | RUSTC_ERROR | Compiler errors: E0063 |
| Demangle::<'a>::as_str | RUSTC_ERROR | Compiler errors: E0063 |
| Demangle::<'a>::as_str | RUSTC_ERROR | Compiler errors: E0560 |
| <SizeLimitedFmtAdapter<F> as core::fmt::Write>::write_str | RUSTC_ERROR | Method/field not found: E0599 |
| <SizeLimitedFmtAdapter<F> as core::fmt::Write>::write_str | RUSTC_ERROR | Method/field not found: E0599 |
| <SizeLimitedFmtAdapter<F> as core::fmt::Write>::write_str | RUSTC_ERROR | Method/field not found: E0599 |
| <SizeLimitedFmtAdapter<F> as core::fmt::Write>::write_str | RUSTC_ERROR | Type mismatch errors: E0412, E0599, E0308, E0369, E0433 |
| <SizeLimitedFmtAdapter<F> as core::fmt::Write>::write_str | RUSTC_ERROR | Type mismatch errors: E0308, E0412, E0369, E0433 |
