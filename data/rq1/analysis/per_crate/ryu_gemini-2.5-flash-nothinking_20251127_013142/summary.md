# Analysis Report: ryu_gemini-2.5-flash-nothinking_20251127_013142

## 1. Executive Summary
- Total Samples: 52
- Success: 35 (67.3%)
- Failures: 17 (32.7%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 16 | 94.1% |
| TRUNCATED_BRACES | 1 | 5.9% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 11 | Failed to resolve import |
| E0432 | 8 | Unresolved import |
| E0308 | 1 | Type mismatch |
| E0599 | 1 | Method/field not found |

## 4. Truncation Issues
| Type | Count | Percentage |
|---|---|---|
| TRUNCATED_BRACES | 1 | 5.9% |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| d2s::d2d | RUSTC_ERROR | Compiler errors: E0432 |
| d2s::d2d | RUSTC_ERROR | Failed to resolve import: E0433 |
| pretty::mantissa::write_mantissa_long | RUSTC_ERROR | Type mismatch errors: E0308 |
| pretty::mantissa::write_mantissa_long | RUSTC_ERROR | Failed to resolve import: E0433 |
| pretty::format64 | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| pretty::format64 | TRUNCATED_BRACES | Unbalanced braces: { (25) vs } (23) |
| pretty::format32 | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| pretty::format32 | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| buffer::Buffer::format | RUSTC_ERROR | Compiler errors: E0432 |
| buffer::Buffer::format | RUSTC_ERROR | Compiler errors: E0432 |
| buffer::Buffer::format | RUSTC_ERROR | Compiler errors: E0432 |
| buffer::Buffer::format | RUSTC_ERROR | Failed to resolve import: E0433 |
| buffer::Buffer::format_finite | RUSTC_ERROR | Failed to resolve import: E0433 |
| <buffer::Buffer as core::clone::Clone>::clone | RUSTC_ERROR | Failed to resolve import: E0433 |
| <buffer::Buffer as core::clone::Clone>::clone | RUSTC_ERROR | Failed to resolve import: E0433 |
| <buffer::Buffer as core::default::Default>::default | RUSTC_ERROR | Method/field not found: E0599 |
| <f64 as buffer::Sealed>::write_to_ryu_buffer | RUSTC_ERROR | Failed to resolve import: E0433 |
