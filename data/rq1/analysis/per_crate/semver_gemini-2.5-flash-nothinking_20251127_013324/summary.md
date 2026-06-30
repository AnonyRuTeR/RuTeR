# Analysis Report: semver_gemini-2.5-flash-nothinking_20251127_013324

## 1. Executive Summary
- Total Samples: 130
- Success: 52 (40.0%)
- Failures: 78 (60.0%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 69 | 88.5% |
| UNSTABLE_FEATURE(E0658) | 7 | 9.0% |
| OTHER_FAILURE | 1 | 1.3% |
| TRUNCATED_BRACES | 1 | 1.3% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 26 | Failed to resolve import |
| E0599 | 26 | Method/field not found |
| E0432 | 18 | Unresolved import |
| E0603 | 16 |  |
| E0658 | 7 | Unstable feature |
| E0308 | 6 | Type mismatch |
| E0277 | 6 | Trait not implemented |
| E0574 | 4 |  |
| E0369 | 3 | Binary operation not supported |
| E0040 | 2 |  |

## 4. Truncation Issues
| Type | Count | Percentage |
|---|---|---|
| TRUNCATED_BRACES | 1 | 1.3% |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| display::pad | RUSTC_ERROR | Compiler errors: E0603 |
| display::pad | RUSTC_ERROR | Failed to resolve import: E0433 |
| display::pad | RUSTC_ERROR | Failed to resolve import: E0433 |
| display::pad | RUSTC_ERROR | Compiler errors: E0603, E0624, E0593 |
| display::pad | RUSTC_ERROR | Failed to resolve import: E0433 |
| display::pad | RUSTC_ERROR | Failed to resolve import: E0603, E0560, E0433 |
| display::pad | RUSTC_ERROR | Compiler errors: E0603 |
| display::pad | RUSTC_ERROR | Compiler errors: E0603 |
| display::pad | RUSTC_ERROR | Failed to resolve import: E0433 |
| display::pad | RUSTC_ERROR | Compiler errors: E0603 |
| display::pad | RUSTC_ERROR | Type mismatch errors: E0603, E0308 |
| display::pad | RUSTC_ERROR | Failed to resolve import: E0433 |
| display::pad | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals |
| display::pad | UNSTABLE_FEATURE(E0658) | Unstable feature used: fmt_internals, fmt_internals, fmt_internals, fmt_internals, fmt_internals, fm... |
| eval::matches_req | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| eval::matches_req | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| eval::matches_comparator | RUSTC_ERROR | Compiler errors: E0432 |
| eval::matches_comparator | RUSTC_ERROR | Compiler errors: E0432 |
| eval::matches_exact | RUSTC_ERROR | Method/field not found: E0599 |
| eval::matches_less | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| eval::matches_less | RUSTC_ERROR | Method/field not found: E0599 |
| eval::pre_is_compatible | RUSTC_ERROR | Failed to resolve import: E0433 |
| eval::pre_is_compatible | RUSTC_ERROR | Method/field not found: E0599 |
| identifier::ptr_to_repr | RUSTC_ERROR | Method/field not found: E0599 |
| identifier::ptr_to_repr | RUSTC_ERROR | Failed to resolve import: E0433 |
| identifier::decode_len::decode_len_cold | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| identifier::decode_len::decode_len_cold | RUSTC_ERROR | Failed to resolve import: E0433 |
| identifier::ptr_as_str | OTHER_FAILURE | Compilation failed without specific error code |
| identifier::ptr_as_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| identifier::bytes_for_varint | RUSTC_ERROR | Type mismatch errors: E0308 |
| identifier::bytes_for_varint | RUSTC_ERROR | Failed to resolve import: E0433 |
| identifier::Identifier::new_unchecked | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| identifier::Identifier::new_unchecked | RUSTC_ERROR | Compiler errors: E0432 |
| identifier::Identifier::new_unchecked | RUSTC_ERROR | Compiler errors: E0432 |
| identifier::Identifier::is_empty_or_inline | RUSTC_ERROR | Method/field not found: E0599 |
| identifier::Identifier::is_empty_or_inline | RUSTC_ERROR | Method/field not found: E0599 |
| <identifier::Identifier as std::ops::Drop>::drop | RUSTC_ERROR | Compiler errors: E0040 |
| <identifier::Identifier as std::ops::Drop>::drop | RUSTC_ERROR | Compiler errors: E0040 |
| <identifier::Identifier as std::cmp::PartialEq>::eq | RUSTC_ERROR | Method/field not found: E0277, E0599 |
| <identifier::Identifier as std::cmp::PartialEq>::eq | RUSTC_ERROR | Method/field not found: E0277, E0599 |
| parse::numeric_identifier | RUSTC_ERROR | Method/field not found: E0603, E0369, E0599 |
| parse::numeric_identifier | RUSTC_ERROR | Failed to resolve import: E0603, E0433 |
| parse::dot | RUSTC_ERROR | Method/field not found: E0277, E0369, E0599 |
| parse::dot | RUSTC_ERROR | Failed to resolve import: E0603, E0433 |
| parse::prerelease_identifier | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| parse::prerelease_identifier | RUSTC_ERROR | Type mismatch errors: E0603, E0308, E0599 |
| parse::build_identifier | RUSTC_ERROR | Method/field not found: E0603, E0432, E0599 |
| parse::build_identifier | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| parse::identifier | RUSTC_ERROR | Method/field not found: E0599 |
| parse::identifier | RUSTC_ERROR | Failed to resolve import: E0433 |
| parse::comparator | TRUNCATED_BRACES | Unbalanced braces: { (30) vs } (28) |
| parse::comparator | RUSTC_ERROR | Method/field not found: E0277, E0432, E0433, E0599 |
| parse::<impl std::str::FromStr for Version>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| parse::<impl std::str::FromStr for VersionReq>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| parse::<impl std::str::FromStr for Comparator>::from_str | RUSTC_ERROR | Type mismatch errors: E0603, E0308, E0599 |
| parse::<impl std::str::FromStr for BuildMetadata>::from_str | RUSTC_ERROR | Type mismatch errors: E0308, E0061, E0599 |
| parse::Error::new | RUSTC_ERROR | Compiler errors: E0574 |
| parse::Error::new | RUSTC_ERROR | Compiler errors: E0574 |
| parse::Error::new | RUSTC_ERROR | Compiler errors: E0574 |
| parse::Error::new | RUSTC_ERROR | Failed to resolve import: E0574, E0433 |
| parse::Error::new | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| parse::Error::new | RUSTC_ERROR | Method/field not found: E0599 |
| impls::<impl std::hash::Hash for identifier::Identifier>::hash | UNSTABLE_FEATURE(E0658) | Unstable feature used: hashmap_internals, hashmap_internals |
| impls::<impl std::hash::Hash for identifier::Identifier>::hash | UNSTABLE_FEATURE(E0658) | Unstable feature used: hashmap_internals, hashmap_internals |
| impls::<impl std::hash::Hash for identifier::Identifier>::hash | UNSTABLE_FEATURE(E0658) | Unstable feature used: hashmap_internals, hashmap_internals |
| impls::<impl std::hash::Hash for identifier::Identifier>::hash | RUSTC_ERROR | Compiler errors: E0603 |
| impls::<impl std::hash::Hash for identifier::Identifier>::hash | RUSTC_ERROR | Compiler errors: E0603 |
| impls::<impl std::hash::Hash for identifier::Identifier>::hash | RUSTC_ERROR | Compiler errors: E0603 |
| impls::<impl std::hash::Hash for identifier::Identifier>::hash | UNSTABLE_FEATURE(E0658) | Unstable feature used: hashmap_internals, hashmap_internals, hashmap_internals |
| impls::<impl std::hash::Hash for identifier::Identifier>::hash | UNSTABLE_FEATURE(E0658) | Unstable feature used: hashmap_internals, hashmap_internals, hashmap_internals, hashmap_internals |
| impls::<impl std::ops::Deref for Prerelease>::deref | RUSTC_ERROR | Compiler errors: E0432 |
| impls::<impl std::ops::Deref for BuildMetadata>::deref | RUSTC_ERROR | Compiler errors: E0432 |
| impls::<impl std::ops::Deref for BuildMetadata>::deref | RUSTC_ERROR | Failed to resolve import: E0433 |
| impls::<impl std::cmp::PartialOrd for BuildMetadata>::partial_cmp | RUSTC_ERROR | Failed to resolve import: E0433 |
| impls::<impl std::cmp::PartialOrd for BuildMetadata>::partial_cmp | RUSTC_ERROR | Compiler errors: E0432 |
| impls::<impl std::cmp::Ord for Prerelease>::cmp | RUSTC_ERROR | Compiler errors: E0432 |
| impls::<impl std::iter::FromIterator<Comparator> for VersionReq>::from_iter | RUSTC_ERROR | Type inference failed: E0282 |
| impls::<impl std::iter::FromIterator<Comparator> for VersionReq>::from_iter | RUSTC_ERROR | Type mismatch errors: E0308, E0433, E0599 |
