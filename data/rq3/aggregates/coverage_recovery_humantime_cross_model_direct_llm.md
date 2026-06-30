# Coverage Recovery Fixed-Crate Cross-Model (humantime)

This slice isolates the repeated humantime runs and must not be collapsed into a cross-crate comparison.

## Raw Observed

| model | run_id | gate status | original generation total | original success assets | failed generation attempts | normalized repaired | compile-executable repaired | original success delta (L/F/R) | repaired delta (L/F/R) |
|---|---|---|---:|---:|---:|---:|---:|---|---|
| claude-3-5-haiku-20241022 | humantime_claude-3-5-haiku-20241022_20251109_133407 | stabilized_pass_gated | 139 | 16 | 123 | 27 | 27 | L 103 / F 26 / R 192 | L 128 / F 32 / R 223 |
| deepseek-v3 | humantime_deepseek-v3_20251109_140004 | stabilized_pass_gated | 101 | 21 | 80 | 13 | 13 | L 164 / F 41 / R 298 | L 58 / F 13 / R 90 |
| gemini-2.5-flash-nothinking | humantime_gemini-2.5-flash-nothinking_20251109_134926 | stabilized_pass_gated | 110 | 25 | 85 | 19 | 19 | L 266 / F 44 / R 571 | L 57 / F 13 / R 126 |
| gemini-2.5-flash-thinking | humantime_gemini-2.5-flash-thinking_20251109_141506 | stabilized_pass_gated | 95 | 25 | 70 | 4 | 4 | L 313 / F 52 / R 713 | L 23 / F 4 / R 49 |
| gpt-3.5-turbo | humantime_gpt-3.5-turbo_20251109_000031 | stabilized_pass_gated | 132 | 18 | 114 | 5 | 5 | L 95 / F 27 / R 173 | L 23 / F 5 / R 51 |
| gpt-4.1-mini | humantime_gpt-4.1-mini_20251109_001052 | stabilized_pass_gated | 122 | 21 | 101 | 15 | 15 | L 169 / F 36 / R 348 | L 152 / F 23 / R 279 |
| gpt-4.1-nano | humantime_gpt-4.1-nano_20251109_132117 | stabilized_pass_gated | 145 | 17 | 128 | 7 | 7 | L 100 / F 23 / R 191 | L 44 / F 7 / R 79 |
| gpt-4o-mini | humantime_gpt-4o-mini_20251108_234236 | stabilized_pass_gated | 137 | 16 | 121 | 2 | 2 | L 90 / F 20 / R 168 | L 10 / F 3 / R 24 |
| gpt-5-nano | humantime_gpt-5-nano_20251109_004846 | stabilized_pass_gated | 138 | 16 | 122 | 10 | 10 | L 108 / F 23 / R 200 | L 47 / F 10 / R 95 |

## Stabilized Pass-Gated Conservative

| model | run_id | original generation total | original success assets | failed generation attempts | normalized repaired | compile-executable repaired | original success delta (L/F/R) | repaired delta (L/F/R) |
|---|---|---:|---:|---:|---:|---:|---|---|
| claude-3-5-haiku-20241022 | humantime_claude-3-5-haiku-20241022_20251109_133407 | 139 | 16 | 123 | 27 | 27 | L 103 / F 26 / R 192 | L 128 / F 32 / R 223 |
| deepseek-v3 | humantime_deepseek-v3_20251109_140004 | 101 | 21 | 80 | 13 | 13 | L 164 / F 41 / R 298 | L 58 / F 13 / R 90 |
| gemini-2.5-flash-nothinking | humantime_gemini-2.5-flash-nothinking_20251109_134926 | 110 | 25 | 85 | 19 | 19 | L 266 / F 44 / R 571 | L 57 / F 13 / R 126 |
| gemini-2.5-flash-thinking | humantime_gemini-2.5-flash-thinking_20251109_141506 | 95 | 25 | 70 | 4 | 4 | L 313 / F 52 / R 713 | L 23 / F 4 / R 49 |
| gpt-3.5-turbo | humantime_gpt-3.5-turbo_20251109_000031 | 132 | 18 | 114 | 5 | 5 | L 95 / F 27 / R 173 | L 23 / F 5 / R 51 |
| gpt-4.1-mini | humantime_gpt-4.1-mini_20251109_001052 | 122 | 21 | 101 | 15 | 15 | L 169 / F 36 / R 348 | L 152 / F 23 / R 279 |
| gpt-4.1-nano | humantime_gpt-4.1-nano_20251109_132117 | 145 | 17 | 128 | 7 | 7 | L 100 / F 23 / R 191 | L 44 / F 7 / R 79 |
| gpt-4o-mini | humantime_gpt-4o-mini_20251108_234236 | 137 | 16 | 121 | 2 | 2 | L 90 / F 20 / R 168 | L 10 / F 3 / R 24 |
| gpt-5-nano | humantime_gpt-5-nano_20251109_004846 | 138 | 16 | 122 | 10 | 10 | L 108 / F 23 / R 200 | L 47 / F 10 / R 95 |
