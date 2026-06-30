# Utility Recovery Ratio Top Table

This paper-facing slice is sourced only from `canonical.stabilized_pass_gated_recovery.run_level`.

- Raw observed canonical runs: 17
- Stabilized pass-gated runs: 17
- Positive repaired-added runs: 9
- Infinite-ratio runs: 0
- Aggregate repaired/original ratio: 15.40%
- Finite per-run mean ratio: 24.15%
- Table slice: All 7 positive runs + 2 zero-gain controls

| slice role | run_id | crate | model | original generation total | failed generation attempts | normalized repaired | original success added (lines) | repaired added (lines) | repaired/original |
|---|---|---|---|---:|---:|---:|---:|---:|---|
| positive_top_1 | humantime_claude-3-5-haiku-20241022_20251109_133407 | humantime | claude-3-5-haiku-20241022 | 139 | 123 | 68 | 103 | 123 | 119.42% |
| positive_top_2 | itoa_gemini-2.5-flash-nothinking_20251127_010109 | itoa | gemini-2.5-flash-nothinking | 25 | 11 | 8 | 222 | 212 | 95.50% |
| positive_top_3 | mio_gemini-2.5-flash-nothinking_20251127_012706 | mio | gemini-2.5-flash-nothinking | 179 | 175 | 6 | 45 | 24 | 53.33% |
| positive_top_4 | log_gemini-2.5-flash-nothinking_20251127_025911 | log | gemini-2.5-flash-nothinking | 178 | 145 | 41 | 230 | 121 | 52.61% |
| positive_top_5 | humantime_deepseek-v3_20251109_140004 | humantime | deepseek-v3 | 101 | 80 | 47 | 164 | 78 | 47.56% |
| positive_top_6 | humantime_gemini-2.5-flash-thinking_20251109_141506 | humantime | gemini-2.5-flash-thinking | 95 | 70 | 28 | 313 | 86 | 27.48% |
| positive_top_7 | rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | rustc-demangle | gemini-2.5-flash-nothinking | 161 | 150 | 19 | 172 | 17 | 9.88% |
| zero_gain_highest_normalized_repaired | humantime_gpt-4.1-nano_20251109_132117 | humantime | gpt-4.1-nano | 145 | 128 | 66 | 100 | 0 | 0.00% |
| zero_gain_highest_original_success | semver_gemini-2.5-flash-nothinking_20251127_013324 | semver | gemini-2.5-flash-nothinking | 94 | 55 | 10 | 557 | 0 | 0.00% |
