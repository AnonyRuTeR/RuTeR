# Longitudinal Analysis: Gemini-2.5-Flash-NoThinking Performance Across Crates

**Generated:** 2026-01-29 20:11:26
**Analyzer:** aggregate_reports.py

---

## 1. Executive Summary

This report presents a longitudinal analysis of the Gemini-2.5-Flash-NoThinking model's performance across multiple Rust crates. The analysis focuses on test generation success rates, compiler error patterns, and failure categorization.

- **Total Crates Analyzed:** 10
- **Total Test Samples:** 3,183
- **Overall Success Rate:** 731/3183 (22.97%)
- **Overall Failure Rate:** 2452/3183 (77.03%)

## 2. Per-Crate Performance Overview

### 2.1 Success Rate Comparison

| Crate | Total | Success | Failures | Success Rate | Failure Rate |
|-------|-------|---------|----------|--------------|--------------|
| chrono | 1510 | 334 | 1176 | 22.12% | 77.88% |
| humantime | 124 | 31 | 93 | 25.00% | 75.00% |
| itoa | 29 | 15 | 14 | 51.72% | 48.28% |
| log | 213 | 45 | 168 | 21.13% | 78.87% |
| mio | 224 | 4 | 220 | 1.79% | 98.21% |
| rand | 682 | 174 | 508 | 25.51% | 74.49% |
| rust-crc32fast | 39 | 29 | 10 | 74.36% | 25.64% |
| rustc-demangle | 180 | 12 | 168 | 6.67% | 93.33% |
| ryu | 52 | 35 | 17 | 67.31% | 32.69% |
| semver | 130 | 52 | 78 | 40.00% | 60.00% |

### 2.2 Key Observations

- **Best Performance:** `rust-crc32fast` with 74.36% success rate
- **Worst Performance:** `mio` with 1.79% success rate
- **Performance Variance:** 72.57 percentage points

## 3. Failure Category Distribution Across Crates

### 3.1 Aggregate Category Distribution

| Category | Total Count | Percentage of Failures |
|----------|-------------|------------------------|
| RUSTC_ERROR | 2287 | 93.27% |
| OTHER_FAILURE | 76 | 3.10% |
| UNSTABLE_FEATURE(E0658) | 46 | 1.88% |
| TRUNCATED_BRACES | 32 | 1.31% |
| TRUNCATED_STRING | 7 | 0.29% |
| TRUNCATED_DANGLED | 3 | 0.12% |
| UNKNOWN_FAILURE | 1 | 0.04% |

### 3.2 Category Distribution by Crate

| Crate | RUSTC_ERROR | COMPILE_FAILED | TRUNCATED_* | PANIC | UNSTABLE_FEATURE | OTHER |
|-------|-------------|----------------|-------------|-------|------------------|-------|
| chrono | 1136 | 0 | 30 | 0 | 0 | 10 |
| humantime | 82 | 0 | 1 | 0 | 0 | 10 |
| itoa | 14 | 0 | 0 | 0 | 0 | 0 |
| log | 168 | 0 | 0 | 0 | 0 | 0 |
| mio | 167 | 0 | 2 | 0 | 0 | 51 |
| rand | 482 | 0 | 1 | 0 | 0 | 25 |
| rust-crc32fast | 10 | 0 | 0 | 0 | 0 | 0 |
| rustc-demangle | 143 | 0 | 6 | 0 | 0 | 19 |
| ryu | 16 | 0 | 1 | 0 | 0 | 0 |
| semver | 69 | 0 | 1 | 0 | 0 | 8 |

## 4. Compiler Error Code Analysis

### 4.1 Top Most Frequent Error Codes

| Rank | Error Code | Total Occurrences | Description |
|------|------------|-------------------|-------------|
| 1 | E0433 | 1210 | An undeclared crate, module, or type was used. |
| 2 | E0432 | 880 | An import was unresolved. |
| 3 | E0599 | 549 | This error occurs when a method is used on a type which doesn't implement it. |
| 4 | E0308 | 207 | Expected type did not match the received type. |
| 5 | E0603 | 185 | A private item was used outside its scope. |
| 6 | E0560 | 120 | An unknown field was specified into a structure. |
| 7 | E0061 | 110 | An invalid number of arguments was passed when calling a function. |
| 8 | E0277 | 81 | You tried to use a type which doesn't implement some trait in a place which expected that trait. |
| 9 | E0412 | 56 | type name is not in scope. |
| 10 | E0658 | 46 | An unstable feature was used. |
| 11 | E0369 | 33 | A binary operation was attempted on a type which doesn't support it. |
| 12 | E0282 | 31 | The compiler could not infer a type and asked for a type annotation. |
| 13 | E0425 | 27 | An unresolved name was used. |
| 14 | E0063 | 25 | A struct's or struct-like enum variant's field was not provided. |
| 15 | E0609 | 17 | Attempted to access a nonexistent field in a struct. |
| 16 | E0107 | 16 | An incorrect number of generic arguments was provided. |
| 17 | E0119 | 15 | There are conflicting trait implementations for the same type. |
| 18 | E0261 | 10 | An undeclared lifetime was used. |
| 19 | E0405 | 9 | The code refers to a trait that is not in scope. |
| 20 | E0562 | 9 | `impl Trait` is only allowed as a function return and argument type. |
| 21 | E0423 | 8 | An identifier was used like a function name or a value was expected and the identifier exists but it belongs to a different namespace. |
| 22 | E0574 | 7 | Something other than a struct, variant or union has been used when one was expected. |
| 23 | E0407 | 5 | A definition of a method not in the implemented trait was given in a trait implementation. |
| 24 | E0766 | 4 | A double quote byte string (`b"`) was not terminated. |
| 25 | E0283 | 4 | The compiler could not infer a type and asked for a type annotation. |
| 26 | E0624 | 3 | A private item was used outside of its scope. |
| 27 | E0223 | 3 | An attempt was made to retrieve an associated type, but the type was ambiguous. |
| 28 | E0116 | 3 | An inherent implementation was defined for a type outside the current crate. |
| 29 | E0765 | 3 | A double quote string (`"`) was not terminated. |
| 30 | E0463 | 2 | A crate was declared but cannot be found. |
| 31 | E0040 | 2 | It is not allowed to manually call destructors in Rust. |
| 32 | E0191 | 1 | An associated type wasn't specified for a trait object. |
| 33 | E0600 | 1 | An unary operator was used on a type which doesn't implement it. |
| 34 | E0532 | 1 | Pattern arm did not match expected kind. |
| 35 | E0616 | 1 | Attempted to access a private field on a struct. |
| 36 | E0422 | 1 | An identifier that is neither defined nor a struct was used. |
| 37 | E0164 | 1 | Something which is neither a tuple struct nor a tuple variant was used as a pattern. |
| 38 | E0533 | 1 | An item which isn't a unit struct, a variant, nor a constant has been used as a match pattern. |
| 39 | E0404 | 1 | A type that is not a trait was used in a trait position, such as a bound or `impl`. |
| 40 | E0615 | 1 | Attempted to access a method like a field. |
| 41 | E0071 | 1 | A structure-literal syntax was used to create an item that is not a structure or enum variant. |
| 42 | E0284 | 1 | This error occurs when the compiler is unable to unambiguously infer the return type of a function or method which is generic on return type, such as the `collect` method for `Iterator`s. |
| 43 | E0437 | 1 | An associated type whose name does not match any of the associated types in the trait was used when implementing the trait. |
| 44 | E0046 | 1 | Items are missing in a trait implementation. |
| 45 | E0053 | 1 | The parameters of any trait method must match between a trait implementation and the trait definition. |
| 46 | E0689 | 1 | A method was called on an ambiguous numeric type. |
| 47 | E0608 | 1 | Attempted to index a value whose type doesn't implement the `std::ops::Index` trait. |
| 48 | E0790 | 1 | You need to specify a specific implementation of the trait in order to call the method. |
| 49 | E0252 | 1 | Two items of the same name cannot be imported without rebinding one of the items under a new local name. |
| 50 | E0576 | 1 | An associated item wasn't found in the given type. |
| 51 | E0593 | 1 | You tried to supply an `Fn`-based type with an incorrect number of arguments than what was expected. |

### 4.2 Error Code Distribution Heatmap

Top 10 error codes across crates:

| Crate | E0433 | E0432 | E0599 | E0308 | E0603 | E0560 | E0061 | E0277 | E0412 | E0658 |
|-------|------|------|------|------|------|------|------|------|------|------|
| chrono | 503 | 614 | 231 | 88 | 149 | 31 | 75 | 18 | 12 | 7 |
| humantime | 47 | 4 | 22 | 2 | 0 | 13 | 0 | 0 | 0 | 10 |
| itoa | 0 | 0 | 4 | 9 | 0 | 0 | 0 | 0 | 0 | 0 |
| log | 95 | 45 | 38 | 30 | 0 | 27 | 13 | 5 | 0 | 0 |
| mio | 76 | 60 | 117 | 4 | 9 | 1 | 7 | 4 | 0 | 4 |
| rand | 336 | 99 | 37 | 40 | 11 | 6 | 14 | 30 | 28 | 1 |
| rust-crc32fast | 8 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| rustc-demangle | 108 | 31 | 73 | 26 | 0 | 41 | 0 | 18 | 16 | 17 |
| ryu | 11 | 8 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| semver | 26 | 18 | 26 | 6 | 16 | 1 | 1 | 6 | 0 | 7 |

## 5. Code Truncation Analysis

### 5.1 Truncation Type Distribution

| Truncation Type | Count | Percentage of Total Failures |
|-----------------|-------|------------------------------|
| TRUNCATED_BRACES | 32 | 1.31% |
| TRUNCATED_STRING | 7 | 0.29% |
| TRUNCATED_DANGLED | 3 | 0.12% |

### 5.2 Truncation by Crate

| Crate | Total Truncations | Truncation Rate |
|-------|-------------------|-----------------|
| chrono | 30 | 2.55% |
| humantime | 1 | 1.08% |
| itoa | 0 | 0.00% |
| log | 0 | 0.00% |
| mio | 2 | 0.91% |
| rand | 1 | 0.20% |
| rust-crc32fast | 0 | 0.00% |
| rustc-demangle | 6 | 3.57% |
| ryu | 1 | 5.88% |
| semver | 1 | 1.28% |

## 6. Statistical Analysis

### 6.1 Success Rate Statistics

- **Mean Success Rate:** 33.56%
- **Standard Deviation:** 23.09%
- **Minimum:** 1.79%
- **Maximum:** 74.36%
- **Range:** 72.57%

## 7. Insights and Recommendations

### 7.1 Key Findings

1. **Import Resolution Issues Dominate:** The top error code `E0433` (Failed to resolve import) accounts for 1210 occurrences, suggesting significant challenges in handling module imports and dependencies.

2. **Type System Challenges:** Error codes like `E0432` (Unresolved import) with 880 occurrences indicate difficulties in type inference and matching.

3. **High-Performing Crates:** rust-crc32fast, ryu demonstrate above-average success rates, possibly due to simpler API surfaces or better-documented interfaces.

4. **Low-Performing Crates:** mio, rustc-demangle show below-average performance, warranting deeper investigation into API complexity or documentation quality.

### 7.2 Recommendations for Model Improvement

1. **Enhanced Import Resolution:** Implement better context understanding for module structures and dependency graphs.
2. **Type System Training:** Increase training data diversity for complex type scenarios, trait implementations, and generic constraints.
3. **Context Window Optimization:** Address truncation issues by improving code generation strategies within token limits.
4. **Crate-Specific Patterns:** Develop fine-tuning approaches for crates with consistently low performance.

## 8. Methodology

### 8.1 Data Collection

- **Analysis Tool:** local_analyzer.py (mechanical analysis engine)
- **Model Under Study:** Gemini-2.5-Flash-NoThinking
- **Crates Analyzed:** 10
- **Total Test Samples:** 3,183

### 8.2 Classification Framework

Failures are categorized using a rule-based mechanical analyzer that identifies:
- **RUSTC_ERROR:** Compilation errors with specific error codes
- **COMPILE_FAILED:** Compilation failures without specific codes
- **TRUNCATED_*:** Code generation truncation issues (braces, strings, keywords)
- **PANIC:** Runtime panic conditions
- **UNSTABLE_FEATURE:** Use of unstable Rust features

## Appendix: Detailed Per-Crate Breakdown

### chrono

- Total: 1510
- Success: 334 (22.12%)
- Failures: 1176 (77.88%)

**Top 5 Error Codes:**

- E0432: 614 occurrences (Unresolved import)
- E0433: 503 occurrences (Failed to resolve import)
- E0599: 231 occurrences (Method/field not found)
- E0603: 149 occurrences (Private item access)
- E0308: 88 occurrences (Type mismatch)

### humantime

- Total: 124
- Success: 31 (25.00%)
- Failures: 93 (75.00%)

**Top 5 Error Codes:**

- E0433: 47 occurrences (Failed to resolve import)
- E0599: 22 occurrences (Method/field not found)
- E0560: 13 occurrences (Unknown struct field)
- E0658: 10 occurrences (Unstable feature)
- E0063: 4 occurrences (Missing struct fields)

### itoa

- Total: 29
- Success: 15 (51.72%)
- Failures: 14 (48.28%)

**Top 5 Error Codes:**

- E0308: 9 occurrences (Type mismatch)
- E0599: 4 occurrences (Method/field not found)
- E0282: 3 occurrences (Type inference failed)

### log

- Total: 213
- Success: 45 (21.13%)
- Failures: 168 (78.87%)

**Top 5 Error Codes:**

- E0433: 95 occurrences (Failed to resolve import)
- E0432: 45 occurrences (Unresolved import)
- E0599: 38 occurrences (Method/field not found)
- E0308: 30 occurrences (Type mismatch)
- E0560: 27 occurrences (Unknown struct field)

### mio

- Total: 224
- Success: 4 (1.79%)
- Failures: 220 (98.21%)

**Top 5 Error Codes:**

- E0599: 117 occurrences (Method/field not found)
- E0433: 76 occurrences (Failed to resolve import)
- E0432: 60 occurrences (Unresolved import)
- E0603: 9 occurrences (Private item access)
- E0425: 8 occurrences (Unresolved name)

### rand

- Total: 682
- Success: 174 (25.51%)
- Failures: 508 (74.49%)

**Top 5 Error Codes:**

- E0433: 336 occurrences (Failed to resolve import)
- E0432: 99 occurrences (Unresolved import)
- E0308: 40 occurrences (Type mismatch)
- E0599: 37 occurrences (Method/field not found)
- E0277: 30 occurrences (Trait not implemented)

### rust-crc32fast

- Total: 39
- Success: 29 (74.36%)
- Failures: 10 (25.64%)

**Top 5 Error Codes:**

- E0433: 8 occurrences (Failed to resolve import)
- E0432: 1 occurrences (Unresolved import)
- E0308: 1 occurrences (Type mismatch)

### rustc-demangle

- Total: 180
- Success: 12 (6.67%)
- Failures: 168 (93.33%)

**Top 5 Error Codes:**

- E0433: 108 occurrences (Failed to resolve import)
- E0599: 73 occurrences (Method/field not found)
- E0560: 41 occurrences (Unknown struct field)
- E0432: 31 occurrences (Unresolved import)
- E0308: 26 occurrences (Type mismatch)

### ryu

- Total: 52
- Success: 35 (67.31%)
- Failures: 17 (32.69%)

**Top 5 Error Codes:**

- E0433: 11 occurrences (Failed to resolve import)
- E0432: 8 occurrences (Unresolved import)
- E0308: 1 occurrences (Type mismatch)
- E0599: 1 occurrences (Method/field not found)

### semver

- Total: 130
- Success: 52 (40.00%)
- Failures: 78 (60.00%)

**Top 5 Error Codes:**

- E0433: 26 occurrences (Failed to resolve import)
- E0599: 26 occurrences (Method/field not found)
- E0432: 18 occurrences (Unresolved import)
- E0603: 16 occurrences (Private item access)
- E0658: 7 occurrences (Unstable feature)

---

*Report generated by aggregate_reports.py*