# Analysis Report: log_gemini-2.5-flash-nothinking_20251127_025911

## 1. Executive Summary
- Total Samples: 213
- Success: 45 (21.1%)
- Failures: 168 (78.9%)

## 2. Failure Distribution
| Category | Count | Percentage |
|---|---|---|
| RUSTC_ERROR | 168 | 100.0% |

## 3. Top Rust Error Codes
| Code | Count | Description |
|---|---|---|
| E0433 | 95 | Failed to resolve import |
| E0432 | 45 | Unresolved import |
| E0599 | 38 | Method/field not found |
| E0308 | 30 | Type mismatch |
| E0560 | 27 | Unknown struct field |
| E0061 | 13 | Wrong number of function arguments |
| E0425 | 5 | Unresolved name |
| E0277 | 5 | Trait not implemented |
| E0063 | 3 | Missing struct fields |
| E0369 | 1 | Binary operation not supported |

## 5. Details
| Node ID | Category | Reason |
|---|---|---|
| eq_ignore_ascii_case::to_ascii_uppercase | RUSTC_ERROR | Failed to resolve import: E0433 |
| eq_ignore_ascii_case::to_ascii_uppercase | RUSTC_ERROR | Failed to resolve import: E0433 |
| set_logger | RUSTC_ERROR | Failed to resolve import: E0433 |
| set_logger | RUSTC_ERROR | Unresolved name/path: E0425, E0433 |
| set_logger | RUSTC_ERROR | Unresolved name/path: E0425, E0433 |
| set_logger | RUSTC_ERROR | Compiler errors: E0432, E0369 |
| set_logger | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| set_logger_inner | RUSTC_ERROR | Compiler errors: E0432 |
| set_logger_racy | RUSTC_ERROR | Failed to resolve import: E0433 |
| set_logger_racy | RUSTC_ERROR | Failed to resolve import: E0433 |
| logger | RUSTC_ERROR | Type mismatch errors: E0308 |
| logger | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| __private_api_log | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| __private_api_log | RUSTC_ERROR | Compiler errors: E0432 |
| __private_api_enabled | RUSTC_ERROR | Compiler errors: E0432 |
| __private_api_enabled | RUSTC_ERROR | Compiler errors: E0432 |
| <Level as core::clone::Clone>::clone | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::clone::Clone>::clone | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialEq>::eq | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialEq>::eq | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialEq<LevelFilter>>::eq | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialEq<LevelFilter>>::eq | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd>::partial_cmp | RUSTC_ERROR | Method/field not found: E0425, E0433, E0599 |
| <Level as core::cmp::PartialOrd>::partial_cmp | RUSTC_ERROR | Method/field not found: E0433, E0061, E0599 |
| <Level as core::cmp::PartialOrd>::lt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd>::lt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd>::le | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd>::le | RUSTC_ERROR | Type mismatch errors: E0308, E0433 |
| <Level as core::cmp::PartialOrd>::gt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd>::gt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd>::ge | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd>::ge | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd<LevelFilter>>::partial_cmp | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd<LevelFilter>>::partial_cmp | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <Level as core::cmp::PartialOrd<LevelFilter>>::lt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd<LevelFilter>>::lt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd<LevelFilter>>::le | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd<LevelFilter>>::le | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd<LevelFilter>>::gt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd<LevelFilter>>::gt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::PartialOrd<LevelFilter>>::ge | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <Level as core::cmp::PartialOrd<LevelFilter>>::ge | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::cmp::Ord>::cmp | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <Level as core::cmp::Ord>::cmp | RUSTC_ERROR | Failed to resolve import: E0433 |
| <Level as core::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| Level::to_level_filter | RUSTC_ERROR | Method/field not found: E0599 |
| <LevelFilter as core::clone::Clone>::clone | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::clone::Clone>::clone | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialEq>::eq | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialEq>::eq | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialEq<Level>>::eq | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialEq<Level>>::eq | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd>::partial_cmp | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd>::partial_cmp | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| <LevelFilter as core::cmp::PartialOrd>::lt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd>::lt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd>::le | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd>::le | RUSTC_ERROR | Type mismatch errors: E0308, E0433 |
| <LevelFilter as core::cmp::PartialOrd>::gt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd>::gt | RUSTC_ERROR | Type mismatch errors: E0308, E0433 |
| <LevelFilter as core::cmp::PartialOrd>::ge | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd>::ge | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd<Level>>::partial_cmp | RUSTC_ERROR | Type mismatch errors: E0432, E0433, E0308 |
| <LevelFilter as core::cmp::PartialOrd<Level>>::partial_cmp | RUSTC_ERROR | Method/field not found: E0432, E0433, E0061, E0599 |
| <LevelFilter as core::cmp::PartialOrd<Level>>::lt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd<Level>>::lt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd<Level>>::le | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd<Level>>::le | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd<Level>>::gt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd<Level>>::gt | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd<Level>>::ge | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::PartialOrd<Level>>::ge | RUSTC_ERROR | Failed to resolve import: E0433 |
| <LevelFilter as core::cmp::Ord>::cmp | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| <LevelFilter as core::cmp::Ord>::cmp | RUSTC_ERROR | Method/field not found: E0433, E0061, E0599 |
| <LevelFilter as core::str::FromStr>::from_str | RUSTC_ERROR | Failed to resolve import: E0433 |
| MaybeStaticStr::<'a>::get | RUSTC_ERROR | Compiler errors: E0432 |
| MaybeStaticStr::<'a>::get | RUSTC_ERROR | Compiler errors: E0432 |
| MaybeStaticStr::<'a>::get | RUSTC_ERROR | Compiler errors: E0432 |
| Record::<'a>::args | RUSTC_ERROR | Compiler errors: E0560 |
| Record::<'a>::args | RUSTC_ERROR | Compiler errors: E0063 |
| Record::<'a>::args | RUSTC_ERROR | Compiler errors: E0063 |
| Record::<'a>::args | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| Record::<'a>::args | RUSTC_ERROR | Type mismatch errors: E0277, E0432, E0560, E0308 |
| Record::<'a>::args | RUSTC_ERROR | Type mismatch errors: E0432, E0560, E0308 |
| Record::<'a>::args | RUSTC_ERROR | Type mismatch errors: E0432, E0560, E0308 |
| Record::<'a>::metadata | RUSTC_ERROR | Method/field not found: E0599 |
| Record::<'a>::metadata | RUSTC_ERROR | Method/field not found: E0261, E0599 |
| Record::<'a>::metadata | RUSTC_ERROR | Method/field not found: E0433, E0599 |
| Record::<'a>::metadata | RUSTC_ERROR | Compiler errors: E0432 |
| Record::<'a>::level | RUSTC_ERROR | Type mismatch errors: E0432, E0433, E0308, E0599 |
| Record::<'a>::level | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| Record::<'a>::level | RUSTC_ERROR | Method/field not found: E0599 |
| Record::<'a>::level | RUSTC_ERROR | Method/field not found: E0599 |
| Record::<'a>::target | RUSTC_ERROR | Method/field not found: E0599 |
| Record::<'a>::target | RUSTC_ERROR | Method/field not found: E0599 |
| Record::<'a>::target | RUSTC_ERROR | Type mismatch errors: E0308, E0599 |
| Record::<'a>::target | RUSTC_ERROR | Method/field not found: E0599 |
| Record::<'a>::module_path | RUSTC_ERROR | Compiler errors: E0404, E0432, E0405 |
| Record::<'a>::module_path | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| Record::<'a>::module_path | RUSTC_ERROR | Compiler errors: E0432 |
| Record::<'a>::module_path | RUSTC_ERROR | Failed to resolve import: E0560, E0433 |
| Record::<'a>::module_path_static | RUSTC_ERROR | Type mismatch errors: E0432, E0433, E0308, E0599 |
| Record::<'a>::module_path_static | RUSTC_ERROR | Failed to resolve import: E0433 |
| Record::<'a>::module_path_static | RUSTC_ERROR | Method/field not found: E0432, E0560, E0433, E0599 |
| Record::<'a>::module_path_static | RUSTC_ERROR | Type mismatch errors: E0308 |
| Record::<'a>::file | RUSTC_ERROR | Failed to resolve import: E0432, E0615, E0433 |
| Record::<'a>::file | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| Record::<'a>::file | RUSTC_ERROR | Type mismatch errors: E0432, E0560, E0433, E0308 |
| Record::<'a>::file | RUSTC_ERROR | Type mismatch errors: E0277, E0308, E0560, E0433 |
| Record::<'a>::file_static | RUSTC_ERROR | Method/field not found: E0560, E0599 |
| Record::<'a>::file_static | RUSTC_ERROR | Method/field not found: E0560, E0599 |
| Record::<'a>::file_static | RUSTC_ERROR | Compiler errors: E0560 |
| Record::<'a>::file_static | RUSTC_ERROR | Method/field not found: E0560, E0433, E0599 |
| Record::<'a>::line | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| Record::<'a>::line | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| Record::<'a>::line | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| Record::<'a>::line | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| RecordBuilder::<'a>::args | RUSTC_ERROR | Unresolved name/path: E0425, E0433 |
| RecordBuilder::<'a>::args | RUSTC_ERROR | Unresolved name/path: E0425, E0433 |
| RecordBuilder::<'a>::args | RUSTC_ERROR | Failed to resolve import: E0433 |
| RecordBuilder::<'a>::args | RUSTC_ERROR | Compiler errors: E0432 |
| RecordBuilder::<'a>::args | RUSTC_ERROR | Method/field not found: E0432, E0061, E0599 |
| RecordBuilder::<'a>::args | RUSTC_ERROR | Method/field not found: E0432, E0599 |
| RecordBuilder::<'a>::args | RUSTC_ERROR | Compiler errors: E0432 |
| RecordBuilder::<'a>::metadata | RUSTC_ERROR | Compiler errors: E0432 |
| RecordBuilder::<'a>::metadata | RUSTC_ERROR | Compiler errors: E0432 |
| RecordBuilder::<'a>::metadata | RUSTC_ERROR | Failed to resolve import: E0433 |
| RecordBuilder::<'a>::metadata | RUSTC_ERROR | Method/field not found: E0061, E0599 |
| RecordBuilder::<'a>::metadata | RUSTC_ERROR | Compiler errors: E0277, E0560 |
| RecordBuilder::<'a>::level | RUSTC_ERROR | Method/field not found: E0061, E0599 |
| RecordBuilder::<'a>::level | RUSTC_ERROR | Type mismatch errors: E0277, E0308 |
| RecordBuilder::<'a>::level | RUSTC_ERROR | Type mismatch errors: E0277, E0308, E0560 |
| RecordBuilder::<'a>::level | RUSTC_ERROR | Compiler errors: E0061 |
| RecordBuilder::<'a>::target | RUSTC_ERROR | Compiler errors: E0063, E0560 |
| RecordBuilder::<'a>::module_path | RUSTC_ERROR | Failed to resolve import: E0433 |
| RecordBuilder::<'a>::module_path_static | RUSTC_ERROR | Failed to resolve import: E0432, E0433, E0061 |
| RecordBuilder::<'a>::module_path_static | RUSTC_ERROR | Method/field not found: E0432, E0433, E0599 |
| RecordBuilder::<'a>::module_path_static | RUSTC_ERROR | Method/field not found: E0432, E0433, E0061, E0599 |
| RecordBuilder::<'a>::module_path_static | RUSTC_ERROR | Method/field not found: E0061, E0599 |
| RecordBuilder::<'a>::build | RUSTC_ERROR | Method/field not found: E0599 |
| RecordBuilder::<'a>::build | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| RecordBuilder::<'a>::build | RUSTC_ERROR | Type mismatch errors: E0308, E0560 |
| Metadata::<'a>::level | RUSTC_ERROR | Method/field not found: E0432, E0061, E0599 |
| Metadata::<'a>::level | RUSTC_ERROR | Type mismatch errors: E0308, E0061 |
| Metadata::<'a>::level | RUSTC_ERROR | Compiler errors: E0560 |
| Metadata::<'a>::level | RUSTC_ERROR | Type mismatch errors: E0308, E0061 |
| Metadata::<'a>::target | RUSTC_ERROR | Compiler errors: E0560 |
| Metadata::<'a>::target | RUSTC_ERROR | Method/field not found: E0599 |
| Metadata::<'a>::target | RUSTC_ERROR | Compiler errors: E0560 |
| Metadata::<'a>::target | RUSTC_ERROR | Compiler errors: E0560 |
| MetadataBuilder::<'a>::new | RUSTC_ERROR | Compiler errors: E0432 |
| MetadataBuilder::<'a>::new | RUSTC_ERROR | Compiler errors: E0432 |
| MetadataBuilder::<'a>::new | RUSTC_ERROR | Compiler errors: E0432 |
| MetadataBuilder::<'a>::level | RUSTC_ERROR | Failed to resolve import: E0433 |
| MetadataBuilder::<'a>::level | RUSTC_ERROR | Failed to resolve import: E0433 |
| <NopLogger as Log>::enabled | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| <NopLogger as Log>::enabled | RUSTC_ERROR | Failed to resolve import: E0433 |
| <NopLogger as Log>::enabled | RUSTC_ERROR | Failed to resolve import: E0433 |
| <NopLogger as Log>::enabled | RUSTC_ERROR | Type mismatch errors: E0308, E0599 |
| <NopLogger as Log>::enabled | RUSTC_ERROR | Failed to resolve import: E0432, E0433 |
| <NopLogger as Log>::log | RUSTC_ERROR | Failed to resolve import: E0433 |
| <NopLogger as Log>::log | RUSTC_ERROR | Failed to resolve import: E0433 |
| <NopLogger as Log>::log | RUSTC_ERROR | Failed to resolve import: E0433 |
| <NopLogger as Log>::log | RUSTC_ERROR | Compiler errors: E0432 |
| <&T as Log>::enabled | RUSTC_ERROR | Type mismatch errors: E0308 |
| <&T as Log>::log | RUSTC_ERROR | Type mismatch errors: E0432, E0308 |
| <&T as Log>::log | RUSTC_ERROR | Compiler errors: E0432 |
| <&T as Log>::flush | RUSTC_ERROR | Type mismatch errors: E0308 |
