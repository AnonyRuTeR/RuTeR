# Analysis Report: rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808

## 1. Executive Summary
- Total Samples: 39
- Success: 29 (74.4%)
- Failures: 10 (25.6%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 10 | 100.0% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 8 | Failed to resolve import |
| E0432 | 1 | Unresolved import |
| E0308 | 1 | Type mismatch |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| baseline::State::new | RUSTC_ERROR | Failed to resolve import: E0433 |
| Hasher::update | RUSTC_ERROR | Compiler errors: E0432 |
| specialized::State::new | RUSTC_ERROR | Failed to resolve import: E0433 |
| specialized::State::update | RUSTC_ERROR | Failed to resolve import: E0433 |
| specialized::State::update | RUSTC_ERROR | Type mismatch errors: E0308 |
| specialized::State::update | RUSTC_ERROR | Failed to resolve import: E0433 |
| specialized::State::update | RUSTC_ERROR | Failed to resolve import: E0433 |
| specialized::State::finalize | RUSTC_ERROR | Failed to resolve import: E0433 |
| specialized::State::reset | RUSTC_ERROR | Failed to resolve import: E0433 |
| specialized::State::combine | RUSTC_ERROR | Failed to resolve import: E0433 |
