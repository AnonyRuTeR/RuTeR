# Horizontal Analysis: Multi-Model Performance on 'humantime' Crate

**Generated:** 2026-01-29 20:11:26
**Analyzer:** aggregate_reports.py

---

## 1. Executive Summary

This report presents a comparative analysis of multiple language models' performance on the `humantime` Rust crate. The analysis evaluates test generation capabilities, compiler error patterns, and model-specific failure characteristics.

- **Crate Under Study:** `humantime`
- **Models Compared:** 9

**Models Analyzed:**
- claude-3-5-haiku-20241022
- deepseek-v3
- gemini-2.5-flash-nothinking
- gemini-2.5-flash-thinking
- gpt-3.5-turbo
- gpt-4.1-mini
- gpt-4.1-nano
- gpt-4o-mini
- gpt-5-nano

## 2. Model Performance Comparison

### 2.1 Overall Success Rates

| Model | Total Tests | Success | Failures | Success Rate | Failure Rate |
|-------|-------------|---------|----------|--------------|--------------|
| claude-3-5-haiku-20241022 | 157 | 19 | 138 | 12.10% | 87.90% |
| deepseek-v3 | 121 | 23 | 98 | 19.01% | 80.99% |
| gemini-2.5-flash-nothinking | 124 | 31 | 93 | 25.00% | 75.00% |
| gemini-2.5-flash-thinking | 109 | 31 | 78 | 28.44% | 71.56% |
| gpt-3.5-turbo | 157 | 19 | 138 | 12.10% | 87.90% |
| gpt-4.1-mini | 139 | 25 | 114 | 17.99% | 82.01% |
| gpt-4.1-nano | 165 | 20 | 145 | 12.12% | 87.88% |
| gpt-4o-mini | 152 | 21 | 131 | 13.82% | 86.18% |
| gpt-5-nano | 153 | 22 | 131 | 14.38% | 85.62% |

### 2.2 Model Rankings

| Rank | Model | Success Rate |
|------|-------|--------------|
| 1 | gemini-2.5-flash-thinking | 28.44% |
| 2 | gemini-2.5-flash-nothinking | 25.00% |
| 3 | deepseek-v3 | 19.01% |
| 4 | gpt-4.1-mini | 17.99% |
| 5 | gpt-5-nano | 14.38% |
| 6 | gpt-4o-mini | 13.82% |
| 7 | gpt-4.1-nano | 12.12% |
| 8 | claude-3-5-haiku-20241022 | 12.10% |
| 9 | gpt-3.5-turbo | 12.10% |

### 2.3 Key Observations

- **Best Performer:** `gemini-2.5-flash-thinking` with 28.44% success rate
- **Lowest Performer:** `gpt-3.5-turbo` with 12.10% success rate
- **Performance Gap:** 16.34 percentage points
- **Average Success Rate:** 17.22%

## 3. Failure Category Distribution by Model

| Model |OTHER_FAILURE | RUSTC_ERROR | TRUNCATED_BRACES | TRUNCATED_STRING | UNSTABLE_FEATURE(E0658) |
|-------|------|------|------|------|------|
| claude-3-5-haiku-20241022 | 0 | 138 | 0 | 0 | 0 |
| deepseek-v3 | 0 | 88 | 0 | 1 | 9 |
| gemini-2.5-flash-nothinking | 0 | 82 | 0 | 1 | 10 |
| gemini-2.5-flash-thinking | 0 | 78 | 0 | 0 | 0 |
| gpt-3.5-turbo | 3 | 123 | 0 | 0 | 12 |
| gpt-4.1-mini | 0 | 101 | 1 | 0 | 12 |
| gpt-4.1-nano | 3 | 130 | 0 | 0 | 12 |
| gpt-4o-mini | 0 | 116 | 0 | 0 | 15 |
| gpt-5-nano | 6 | 120 | 0 | 0 | 5 |

### 3.1 Category Analysis

**OTHER_FAILURE:**

- gpt-5-nano: 6 (4.6% of failures)
- gpt-3.5-turbo: 3 (2.2% of failures)
- gpt-4.1-nano: 3 (2.1% of failures)
- claude-3-5-haiku-20241022: 0 (0.0% of failures)
- deepseek-v3: 0 (0.0% of failures)
- gemini-2.5-flash-nothinking: 0 (0.0% of failures)
- gemini-2.5-flash-thinking: 0 (0.0% of failures)
- gpt-4.1-mini: 0 (0.0% of failures)
- gpt-4o-mini: 0 (0.0% of failures)

**RUSTC_ERROR:**

- claude-3-5-haiku-20241022: 138 (100.0% of failures)
- gpt-4.1-nano: 130 (89.7% of failures)
- gpt-3.5-turbo: 123 (89.1% of failures)
- gpt-5-nano: 120 (91.6% of failures)
- gpt-4o-mini: 116 (88.5% of failures)
- gpt-4.1-mini: 101 (88.6% of failures)
- deepseek-v3: 88 (89.8% of failures)
- gemini-2.5-flash-nothinking: 82 (88.2% of failures)
- gemini-2.5-flash-thinking: 78 (100.0% of failures)

**TRUNCATED_BRACES:**

- gpt-4.1-mini: 1 (0.9% of failures)
- claude-3-5-haiku-20241022: 0 (0.0% of failures)
- deepseek-v3: 0 (0.0% of failures)
- gemini-2.5-flash-nothinking: 0 (0.0% of failures)
- gemini-2.5-flash-thinking: 0 (0.0% of failures)
- gpt-3.5-turbo: 0 (0.0% of failures)
- gpt-4.1-nano: 0 (0.0% of failures)
- gpt-4o-mini: 0 (0.0% of failures)
- gpt-5-nano: 0 (0.0% of failures)

**TRUNCATED_STRING:**

- deepseek-v3: 1 (1.0% of failures)
- gemini-2.5-flash-nothinking: 1 (1.1% of failures)
- claude-3-5-haiku-20241022: 0 (0.0% of failures)
- gemini-2.5-flash-thinking: 0 (0.0% of failures)
- gpt-3.5-turbo: 0 (0.0% of failures)
- gpt-4.1-mini: 0 (0.0% of failures)
- gpt-4.1-nano: 0 (0.0% of failures)
- gpt-4o-mini: 0 (0.0% of failures)
- gpt-5-nano: 0 (0.0% of failures)

**UNSTABLE_FEATURE(E0658):**

- gpt-4o-mini: 15 (11.5% of failures)
- gpt-3.5-turbo: 12 (8.7% of failures)
- gpt-4.1-mini: 12 (10.5% of failures)
- gpt-4.1-nano: 12 (8.3% of failures)
- gemini-2.5-flash-nothinking: 10 (10.8% of failures)
- deepseek-v3: 9 (9.2% of failures)
- gpt-5-nano: 5 (3.8% of failures)
- claude-3-5-haiku-20241022: 0 (0.0% of failures)
- gemini-2.5-flash-thinking: 0 (0.0% of failures)

## 4. Compiler Error Code Comparison

### 4.1 Top 15 Error Codes Across All Models

| Rank | Error Code | Total Occurrences | Description |
|------|------------|-------------------|-------------|
| 1 | E0433 | 643 | Failed to resolve import |
| 2 | E0599 | 183 | Method/field not found |
| 3 | E0432 | 134 | Unresolved import |
| 4 | E0308 | 88 | Type mismatch |
| 5 | E0063 | 86 | Missing struct fields |
| 6 | E0658 | 76 | Unstable feature |
| 7 | E0560 | 58 | Unknown struct field |
| 8 | E0061 | 48 | Wrong number of function arguments |
| 9 | E0277 | 16 | Trait not implemented |
| 10 | E0261 | 16 | — |
| 11 | E0425 | 15 | Unresolved name |
| 12 | E0107 | 14 | — |
| 13 | E0283 | 11 | Type annotations needed |
| 14 | E0412 | 11 | Cannot find type in this scope |
| 15 | E0252 | 7 | — |

### 4.2 Error Code Distribution Heatmap

Top 10 error codes by model:

| Model | E0433 | E0599 | E0432 | E0308 | E0063 | E0658 | E0560 | E0061 | E0277 | E0261 |
|-------|------|------|------|------|------|------|------|------|------|------|
| claude-3-5-haiku-20241022 | 96 | 23 | 11 | 3 | 4 | 0 | 2 | 8 | 0 | 0 |
| deepseek-v3 | 59 | 16 | 1 | 0 | 13 | 9 | 3 | 0 | 8 | 0 |
| gemini-2.5-flash-nothinking | 47 | 22 | 4 | 2 | 4 | 10 | 13 | 0 | 0 | 0 |
| gemini-2.5-flash-thinking | 30 | 34 | 11 | 2 | 2 | 0 | 12 | 0 | 0 | 0 |
| gpt-3.5-turbo | 93 | 22 | 35 | 29 | 14 | 12 | 0 | 11 | 2 | 14 |
| gpt-4.1-mini | 76 | 14 | 7 | 2 | 7 | 13 | 13 | 0 | 1 | 0 |
| gpt-4.1-nano | 80 | 8 | 32 | 27 | 24 | 12 | 14 | 7 | 0 | 1 |
| gpt-4o-mini | 59 | 36 | 33 | 13 | 17 | 15 | 1 | 14 | 5 | 0 |
| gpt-5-nano | 103 | 8 | 0 | 10 | 1 | 5 | 0 | 8 | 0 | 1 |

### 4.3 Model-Specific Error Patterns

**claude-3-5-haiku-20241022:**

- E0433 (Failed to resolve import): 96 occurrences (69.6% of failures)
- E0599 (Method/field not found): 23 occurrences (16.7% of failures)
- E0432 (Unresolved import): 11 occurrences (8.0% of failures)

**deepseek-v3:**

- E0433 (Failed to resolve import): 59 occurrences (60.2% of failures)
- E0599 (Method/field not found): 16 occurrences (16.3% of failures)
- E0063 (Missing struct fields): 13 occurrences (13.3% of failures)

**gemini-2.5-flash-nothinking:**

- E0433 (Failed to resolve import): 47 occurrences (50.5% of failures)
- E0599 (Method/field not found): 22 occurrences (23.7% of failures)
- E0560 (Unknown struct field): 13 occurrences (14.0% of failures)

**gemini-2.5-flash-thinking:**

- E0599 (Method/field not found): 34 occurrences (43.6% of failures)
- E0433 (Failed to resolve import): 30 occurrences (38.5% of failures)
- E0560 (Unknown struct field): 12 occurrences (15.4% of failures)

**gpt-3.5-turbo:**

- E0433 (Failed to resolve import): 93 occurrences (67.4% of failures)
- E0432 (Unresolved import): 35 occurrences (25.4% of failures)
- E0308 (Type mismatch): 29 occurrences (21.0% of failures)

**gpt-4.1-mini:**

- E0433 (Failed to resolve import): 76 occurrences (66.7% of failures)
- E0599 (Method/field not found): 14 occurrences (12.3% of failures)
- E0658 (Unstable feature): 13 occurrences (11.4% of failures)

**gpt-4.1-nano:**

- E0433 (Failed to resolve import): 80 occurrences (55.2% of failures)
- E0432 (Unresolved import): 32 occurrences (22.1% of failures)
- E0308 (Type mismatch): 27 occurrences (18.6% of failures)

**gpt-4o-mini:**

- E0433 (Failed to resolve import): 59 occurrences (45.0% of failures)
- E0599 (Method/field not found): 36 occurrences (27.5% of failures)
- E0432 (Unresolved import): 33 occurrences (25.2% of failures)

**gpt-5-nano:**

- E0433 (Failed to resolve import): 103 occurrences (78.6% of failures)
- E0308 (Type mismatch): 10 occurrences (7.6% of failures)
- E0061 (Wrong number of function arguments): 8 occurrences (6.1% of failures)

## 5. Code Truncation Comparison

| Model | Total Truncations | Truncation Types |
|-------|-------------------|------------------|
| claude-3-5-haiku-20241022 | 0 | None |
| deepseek-v3 | 1 | TRUNCATED_STRING(1) |
| gemini-2.5-flash-nothinking | 1 | TRUNCATED_STRING(1) |
| gemini-2.5-flash-thinking | 0 | None |
| gpt-3.5-turbo | 0 | None |
| gpt-4.1-mini | 1 | TRUNCATED_BRACES(1) |
| gpt-4.1-nano | 0 | None |
| gpt-4o-mini | 0 | None |
| gpt-5-nano | 0 | None |

## 6. Statistical Analysis

### 6.1 Success Rate Distribution

- **Mean:** 17.22%
- **Standard Deviation:** 5.66%
- **Minimum:** 12.10%
- **Maximum:** 28.44%
- **Range:** 16.34%
- **Coefficient of Variation:** 32.87%

## 7. Comparative Insights and Findings

### 7.1 Model Architecture Impact

- **GPT Family:** Average success rate 14.08% across 5 models
- **Gemini Family:** Average success rate 26.72% across 2 models
- **Claude Family:** Average success rate 12.10% across 1 models
- **DeepSeek Family:** Average success rate 19.01% across 1 models

### 7.2 Error Pattern Analysis

- **Universal Challenge:** Error code `E0433` (Failed to resolve import) is the most common across all models with 643 total occurrences, indicating a shared difficulty in handling this aspect of the `humantime` API.

### 7.3 Model-Specific Strengths and Weaknesses

**E0433 (Failed to resolve import):**
- Handles best: gemini-2.5-flash-thinking (30 occurrences, 38.5% of failures)
- Struggles most: gpt-5-nano (103 occurrences, 78.6% of failures)

**E0599 (Method/field not found):**
- Handles best: gpt-4.1-nano (8 occurrences, 5.5% of failures)
- Struggles most: gemini-2.5-flash-thinking (34 occurrences, 43.6% of failures)

**E0063 (Missing struct fields):**
- Handles best: gpt-5-nano (1 occurrences, 0.8% of failures)
- Struggles most: gpt-4.1-nano (24 occurrences, 16.6% of failures)

## 8. Recommendations

### 8.1 For Model Selection

- For `humantime` specifically, consider using `gemini-2.5-flash-thinking` which demonstrates the highest success rate.
- Models showing below-average performance may benefit from targeted fine-tuning on similar time/date parsing libraries.

### 8.2 For Model Development

- **Import Resolution:** All models struggle with `E0433` errors; improving context awareness of crate structure is critical.
- **API Signature Accuracy:** Frequent `E0599` and `E0061` errors suggest need for better API documentation understanding.
- **Type Inference:** Training on more complex type scenarios could reduce `E0282` and `E0308` errors.

## 9. Methodology

### 9.1 Experimental Setup

- **Target Crate:** `humantime`
- **Models Tested:** 9
- **Analysis Framework:** Mechanical rule-based classification using local_analyzer.py
- **Metrics:** Success rate, failure categorization, error code distribution

### 9.2 Limitations

- Analysis is limited to compile-time correctness; runtime behavior is not evaluated
- Different models may have been tested with different API subsets or test configurations
- Timestamps indicate models were tested at different times, potentially with different RUG versions

## Appendix: Detailed Model Profiles

### claude-3-5-haiku-20241022

- **Timestamp:** 20251109_133407
- **Total Tests:** 157
- **Success Rate:** 12.10%

**Failure Categories:**

- RUSTC_ERROR: 138 (100.0%)

**Top 5 Error Codes:**

- E0433: 96 (Failed to resolve import)
- E0599: 23 (Method/field not found)
- E0432: 11 (Unresolved import)
- E0061: 8 (Wrong number of function arguments)
- E0063: 4 (Missing struct fields)

### deepseek-v3

- **Timestamp:** 20251109_140004
- **Total Tests:** 121
- **Success Rate:** 19.01%

**Failure Categories:**

- RUSTC_ERROR: 88 (89.8%)
- UNSTABLE_FEATURE(E0658): 9 (9.2%)
- TRUNCATED_STRING: 1 (1.0%)

**Top 5 Error Codes:**

- E0433: 59 (Failed to resolve import)
- E0599: 16 (Method/field not found)
- E0063: 13 (Missing struct fields)
- E0658: 9 (Unstable feature)
- E0277: 8 (Trait not implemented)

### gemini-2.5-flash-nothinking

- **Timestamp:** 20251109_134926
- **Total Tests:** 124
- **Success Rate:** 25.00%

**Failure Categories:**

- RUSTC_ERROR: 82 (88.2%)
- UNSTABLE_FEATURE(E0658): 10 (10.8%)
- TRUNCATED_STRING: 1 (1.1%)

**Top 5 Error Codes:**

- E0433: 47 (Failed to resolve import)
- E0599: 22 (Method/field not found)
- E0560: 13 (Unknown struct field)
- E0658: 10 (Unstable feature)
- E0063: 4 (Missing struct fields)

### gemini-2.5-flash-thinking

- **Timestamp:** 20251109_141506
- **Total Tests:** 109
- **Success Rate:** 28.44%

**Failure Categories:**

- RUSTC_ERROR: 78 (100.0%)

**Top 5 Error Codes:**

- E0599: 34 (Method/field not found)
- E0433: 30 (Failed to resolve import)
- E0560: 12 (Unknown struct field)
- E0432: 11 (Unresolved import)
- E0063: 2 (Missing struct fields)

### gpt-3.5-turbo

- **Timestamp:** 20251109_000031
- **Total Tests:** 157
- **Success Rate:** 12.10%

**Failure Categories:**

- RUSTC_ERROR: 123 (89.1%)
- UNSTABLE_FEATURE(E0658): 12 (8.7%)
- OTHER_FAILURE: 3 (2.2%)

**Top 5 Error Codes:**

- E0433: 93 (Failed to resolve import)
- E0432: 35 (Unresolved import)
- E0308: 29 (Type mismatch)
- E0599: 22 (Method/field not found)
- E0063: 14 (Missing struct fields)

### gpt-4.1-mini

- **Timestamp:** 20251109_001052
- **Total Tests:** 139
- **Success Rate:** 17.99%

**Failure Categories:**

- RUSTC_ERROR: 101 (88.6%)
- UNSTABLE_FEATURE(E0658): 12 (10.5%)
- TRUNCATED_BRACES: 1 (0.9%)

**Top 5 Error Codes:**

- E0433: 76 (Failed to resolve import)
- E0599: 14 (Method/field not found)
- E0658: 13 (Unstable feature)
- E0560: 13 (Unknown struct field)
- E0063: 7 (Missing struct fields)

### gpt-4.1-nano

- **Timestamp:** 20251109_132117
- **Total Tests:** 165
- **Success Rate:** 12.12%

**Failure Categories:**

- RUSTC_ERROR: 130 (89.7%)
- UNSTABLE_FEATURE(E0658): 12 (8.3%)
- OTHER_FAILURE: 3 (2.1%)

**Top 5 Error Codes:**

- E0433: 80 (Failed to resolve import)
- E0432: 32 (Unresolved import)
- E0308: 27 (Type mismatch)
- E0063: 24 (Missing struct fields)
- E0560: 14 (Unknown struct field)

### gpt-4o-mini

- **Timestamp:** 20251108_234236
- **Total Tests:** 152
- **Success Rate:** 13.82%

**Failure Categories:**

- RUSTC_ERROR: 116 (88.5%)
- UNSTABLE_FEATURE(E0658): 15 (11.5%)

**Top 5 Error Codes:**

- E0433: 59 (Failed to resolve import)
- E0599: 36 (Method/field not found)
- E0432: 33 (Unresolved import)
- E0063: 17 (Missing struct fields)
- E0658: 15 (Unstable feature)

### gpt-5-nano

- **Timestamp:** 20251109_004846
- **Total Tests:** 153
- **Success Rate:** 14.38%

**Failure Categories:**

- RUSTC_ERROR: 120 (91.6%)
- OTHER_FAILURE: 6 (4.6%)
- UNSTABLE_FEATURE(E0658): 5 (3.8%)

**Top 5 Error Codes:**

- E0433: 103 (Failed to resolve import)
- E0308: 10 (Type mismatch)
- E0061: 8 (Wrong number of function arguments)
- E0599: 8 (Method/field not found)
- E0658: 5 (Unstable feature)

---

*Report generated by aggregate_reports.py*