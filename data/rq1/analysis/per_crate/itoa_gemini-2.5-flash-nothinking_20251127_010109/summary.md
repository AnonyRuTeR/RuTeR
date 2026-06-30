# Analysis Report: itoa_gemini-2.5-flash-nothinking_20251127_010109

## 1. Executive Summary
- Total Samples: 29
- Success: 15 (51.7%)
- Failures: 14 (48.3%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 14 | 100.0% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0308 | 9 | Type mismatch |
| E0599 | 4 | Method/field not found |
| E0282 | 3 | Type inference failed |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| <impl private::Sealed for i8>::write | RUSTC_ERROR | Type inference failed: E0282 |
| <impl private::Sealed for i8>::write | RUSTC_ERROR | Type inference failed: E0282 |
| <impl private::Sealed for i8>::write | RUSTC_ERROR | Type inference failed: E0282 |
| <impl private::Sealed for u8>::write | RUSTC_ERROR | Type mismatch errors: E0308 |
| <impl private::Sealed for u8>::write | RUSTC_ERROR | Type mismatch errors: E0308 |
| <impl private::Sealed for u16>::write | RUSTC_ERROR | Type mismatch errors: E0308 |
| <impl private::Sealed for i32>::write | RUSTC_ERROR | Type mismatch errors: E0308 |
| <impl private::Sealed for isize>::write | RUSTC_ERROR | Type mismatch errors: E0308, E0599 |
| <impl private::Sealed for isize>::write | RUSTC_ERROR | Method/field not found: E0599 |
| <impl private::Sealed for usize>::write | RUSTC_ERROR | Method/field not found: E0599 |
| <impl private::Sealed for usize>::write | RUSTC_ERROR | Type mismatch errors: E0308, E0599 |
| <impl private::Sealed for i128>::write | RUSTC_ERROR | Type mismatch errors: E0308 |
| <impl private::Sealed for i128>::write | RUSTC_ERROR | Type mismatch errors: E0308 |
| <impl private::Sealed for u128>::write | RUSTC_ERROR | Type mismatch errors: E0308 |
