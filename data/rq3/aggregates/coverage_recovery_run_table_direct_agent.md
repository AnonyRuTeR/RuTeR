# Coverage Recovery Run Table

This table is the main run-level utility audit for canonical replay.
Coverage metrics reported: lines, functions, regions.
Branch coverage reported: False.
Branch note: Current coverage replay exports only provide usable lines/functions/regions totals; branch coverage is excluded from the Phase1A+ main summary.
Raw observed runs: 17
Stabilized pass-gated runs: 17

## Raw Observed

| run_id | crate | model | gate status | original generation total | original success assets | failed generation attempts | normalized repaired | compile-executable repaired | runtime included repaired | runtime quarantined original | runtime quarantined repaired | coverage effective |
|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| humantime_claude-3-5-haiku-20241022_20251109_133407 | humantime | claude-3-5-haiku-20241022 | stabilized_pass_gated | 139 | 16 | 123 | 22 | 22 | 21 | 0 | 1 | yes |
| humantime_deepseek-v3_20251109_140004 | humantime | deepseek-v3 | stabilized_pass_gated | 101 | 21 | 80 | 20 | 20 | 19 | 1 | 1 | yes |
| humantime_gemini-2.5-flash-nothinking_20251109_134926 | humantime | gemini-2.5-flash-nothinking | stabilized_pass_gated | 110 | 25 | 85 | 20 | 20 | 16 | 4 | 4 | yes |
| humantime_gemini-2.5-flash-thinking_20251109_141506 | humantime | gemini-2.5-flash-thinking | stabilized_pass_gated | 95 | 25 | 70 | 17 | 17 | 17 | 7 | 0 | yes |
| humantime_gpt-3.5-turbo_20251109_000031 | humantime | gpt-3.5-turbo | stabilized_pass_gated | 132 | 18 | 114 | 24 | 24 | 22 | 0 | 2 | yes |
| humantime_gpt-4.1-mini_20251109_001052 | humantime | gpt-4.1-mini | stabilized_pass_gated | 122 | 21 | 101 | 31 | 4 | 4 | 1 | 0 | yes |
| humantime_gpt-4.1-nano_20251109_132117 | humantime | gpt-4.1-nano | stabilized_pass_gated | 145 | 17 | 128 | 31 | 31 | 30 | 0 | 1 | yes |
| humantime_gpt-4o-mini_20251108_234236 | humantime | gpt-4o-mini | stabilized_pass_gated | 137 | 16 | 121 | 32 | 32 | 24 | 2 | 8 | yes |
| humantime_gpt-5-nano_20251109_004846 | humantime | gpt-5-nano | stabilized_pass_gated | 138 | 16 | 122 | 33 | 33 | 33 | 0 | 0 | yes |
| itoa_gemini-2.5-flash-nothinking_20251127_010109 | itoa | gemini-2.5-flash-nothinking | stabilized_pass_gated | 25 | 14 | 11 | 4 | 4 | 4 | 0 | 0 | yes |
| log_gemini-2.5-flash-nothinking_20251127_025911 | log | gemini-2.5-flash-nothinking | stabilized_pass_gated | 178 | 33 | 145 | 27 | 27 | 27 | 3 | 0 | yes |
| mio_gemini-2.5-flash-nothinking_20251127_012706 | mio | gemini-2.5-flash-nothinking | stabilized_pass_gated | 179 | 4 | 175 | 26 | 21 | 15 | 0 | 6 | yes |
| rand_gemini-2.5-flash-nothinking_20251127_025504 | rand | gemini-2.5-flash-nothinking | stabilized_pass_gated | 496 | 141 | 355 | 95 | 95 | 93 | 1 | 2 | yes |
| rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808 | rust-crc32fast | gemini-2.5-flash-nothinking | stabilized_pass_gated | 35 | 26 | 9 | 5 | 5 | 5 | 2 | 0 | yes |
| rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | rustc-demangle | gemini-2.5-flash-nothinking | stabilized_pass_gated | 161 | 11 | 150 | 6 | 6 | 5 | 1 | 1 | yes |
| ryu_gemini-2.5-flash-nothinking_20251127_013142 | ryu | gemini-2.5-flash-nothinking | stabilized_pass_gated | 48 | 34 | 14 | 4 | 4 | 3 | 9 | 1 | yes |
| semver_gemini-2.5-flash-nothinking_20251127_013324 | semver | gemini-2.5-flash-nothinking | stabilized_pass_gated | 94 | 39 | 55 | 10 | 10 | 10 | 7 | 0 | yes |

| run_id | original success delta (L/F/R) | repaired delta (L/F/R) |
|---|---|---|
| humantime_claude-3-5-haiku-20241022_20251109_133407 | L 103 / F 26 / R 192 | L 119 / F 26 / R 192 |
| humantime_deepseek-v3_20251109_140004 | L 164 / F 41 / R 298 | L 103 / F 20 / R 174 |
| humantime_gemini-2.5-flash-nothinking_20251109_134926 | L 266 / F 44 / R 570 | L 67 / F 15 / R 147 |
| humantime_gemini-2.5-flash-thinking_20251109_141506 | L 313 / F 52 / R 714 | L 174 / F 20 / R 385 |
| humantime_gpt-3.5-turbo_20251109_000031 | L 95 / F 27 / R 173 | L 108 / F 26 / R 222 |
| humantime_gpt-4.1-mini_20251109_001052 | L 169 / F 36 / R 348 | L 47 / F 5 / R 93 |
| humantime_gpt-4.1-nano_20251109_132117 | L 100 / F 23 / R 190 | L 199 / F 35 / R 340 |
| humantime_gpt-4o-mini_20251108_234236 | L 90 / F 20 / R 169 | L 126 / F 34 / R 288 |
| humantime_gpt-5-nano_20251109_004846 | L 108 / F 23 / R 199 | L 141 / F 38 / R 278 |
| itoa_gemini-2.5-flash-nothinking_20251127_010109 | L 222 / F 29 / R 513 | L 59 / F 10 / R 128 |
| log_gemini-2.5-flash-nothinking_20251127_025911 | L 230 / F 47 / R 439 | L 223 / F 36 / R 348 |
| mio_gemini-2.5-flash-nothinking_20251127_012706 | L 45 / F 7 / R 98 | L 100 / F 23 / R 166 |
| rand_gemini-2.5-flash-nothinking_20251127_025504 | L 1242 / F 235 / R 2595 | L 722 / F 137 / R 1450 |
| rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808 | L 161 / F 33 / R 255 | L 23 / F 5 / R 48 |
| rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | L 172 / F 32 / R 270 | L 17 / F 5 / R 26 |
| ryu_gemini-2.5-flash-nothinking_20251127_013142 | L 373 / F 32 / R 816 | L 17 / F 4 / R 30 |
| semver_gemini-2.5-flash-nothinking_20251127_013324 | L 557 / F 72 / R 1323 | L 62 / F 13 / R 119 |

## Stabilized Pass-Gated Conservative

| run_id | crate | model | gate status | original generation total | original success assets | failed generation attempts | normalized repaired | compile-executable repaired | runtime included repaired | runtime quarantined original | runtime quarantined repaired | coverage effective |
|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| humantime_claude-3-5-haiku-20241022_20251109_133407 | humantime | claude-3-5-haiku-20241022 | stabilized_pass_gated | 139 | 16 | 123 | 22 | 22 | 21 | 0 | 1 | yes |
| humantime_deepseek-v3_20251109_140004 | humantime | deepseek-v3 | stabilized_pass_gated | 101 | 21 | 80 | 20 | 20 | 19 | 1 | 1 | yes |
| humantime_gemini-2.5-flash-nothinking_20251109_134926 | humantime | gemini-2.5-flash-nothinking | stabilized_pass_gated | 110 | 25 | 85 | 20 | 20 | 16 | 4 | 4 | yes |
| humantime_gemini-2.5-flash-thinking_20251109_141506 | humantime | gemini-2.5-flash-thinking | stabilized_pass_gated | 95 | 25 | 70 | 17 | 17 | 17 | 7 | 0 | yes |
| humantime_gpt-3.5-turbo_20251109_000031 | humantime | gpt-3.5-turbo | stabilized_pass_gated | 132 | 18 | 114 | 24 | 24 | 22 | 0 | 2 | yes |
| humantime_gpt-4.1-mini_20251109_001052 | humantime | gpt-4.1-mini | stabilized_pass_gated | 122 | 21 | 101 | 31 | 4 | 4 | 1 | 0 | yes |
| humantime_gpt-4.1-nano_20251109_132117 | humantime | gpt-4.1-nano | stabilized_pass_gated | 145 | 17 | 128 | 31 | 31 | 30 | 0 | 1 | yes |
| humantime_gpt-4o-mini_20251108_234236 | humantime | gpt-4o-mini | stabilized_pass_gated | 137 | 16 | 121 | 32 | 32 | 24 | 2 | 8 | yes |
| humantime_gpt-5-nano_20251109_004846 | humantime | gpt-5-nano | stabilized_pass_gated | 138 | 16 | 122 | 33 | 33 | 33 | 0 | 0 | yes |
| itoa_gemini-2.5-flash-nothinking_20251127_010109 | itoa | gemini-2.5-flash-nothinking | stabilized_pass_gated | 25 | 14 | 11 | 4 | 4 | 4 | 0 | 0 | yes |
| log_gemini-2.5-flash-nothinking_20251127_025911 | log | gemini-2.5-flash-nothinking | stabilized_pass_gated | 178 | 33 | 145 | 27 | 27 | 27 | 3 | 0 | yes |
| mio_gemini-2.5-flash-nothinking_20251127_012706 | mio | gemini-2.5-flash-nothinking | stabilized_pass_gated | 179 | 4 | 175 | 26 | 21 | 15 | 0 | 6 | yes |
| rand_gemini-2.5-flash-nothinking_20251127_025504 | rand | gemini-2.5-flash-nothinking | stabilized_pass_gated | 496 | 141 | 355 | 95 | 95 | 93 | 1 | 2 | yes |
| rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808 | rust-crc32fast | gemini-2.5-flash-nothinking | stabilized_pass_gated | 35 | 26 | 9 | 5 | 5 | 5 | 2 | 0 | yes |
| rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | rustc-demangle | gemini-2.5-flash-nothinking | stabilized_pass_gated | 161 | 11 | 150 | 6 | 6 | 5 | 1 | 1 | yes |
| ryu_gemini-2.5-flash-nothinking_20251127_013142 | ryu | gemini-2.5-flash-nothinking | stabilized_pass_gated | 48 | 34 | 14 | 4 | 4 | 3 | 9 | 1 | yes |
| semver_gemini-2.5-flash-nothinking_20251127_013324 | semver | gemini-2.5-flash-nothinking | stabilized_pass_gated | 94 | 39 | 55 | 10 | 10 | 10 | 7 | 0 | yes |

| run_id | original success delta (L/F/R) | repaired delta (L/F/R) |
|---|---|---|
| humantime_claude-3-5-haiku-20241022_20251109_133407 | L 103 / F 26 / R 192 | L 119 / F 26 / R 192 |
| humantime_deepseek-v3_20251109_140004 | L 164 / F 41 / R 298 | L 103 / F 20 / R 174 |
| humantime_gemini-2.5-flash-nothinking_20251109_134926 | L 266 / F 44 / R 570 | L 67 / F 15 / R 147 |
| humantime_gemini-2.5-flash-thinking_20251109_141506 | L 313 / F 52 / R 714 | L 174 / F 20 / R 385 |
| humantime_gpt-3.5-turbo_20251109_000031 | L 95 / F 27 / R 173 | L 108 / F 26 / R 222 |
| humantime_gpt-4.1-mini_20251109_001052 | L 169 / F 36 / R 348 | L 47 / F 5 / R 93 |
| humantime_gpt-4.1-nano_20251109_132117 | L 100 / F 23 / R 190 | L 199 / F 35 / R 340 |
| humantime_gpt-4o-mini_20251108_234236 | L 90 / F 20 / R 169 | L 126 / F 34 / R 288 |
| humantime_gpt-5-nano_20251109_004846 | L 108 / F 23 / R 199 | L 141 / F 38 / R 278 |
| itoa_gemini-2.5-flash-nothinking_20251127_010109 | L 222 / F 29 / R 513 | L 59 / F 10 / R 128 |
| log_gemini-2.5-flash-nothinking_20251127_025911 | L 230 / F 47 / R 439 | L 223 / F 36 / R 348 |
| mio_gemini-2.5-flash-nothinking_20251127_012706 | L 45 / F 7 / R 98 | L 100 / F 23 / R 166 |
| rand_gemini-2.5-flash-nothinking_20251127_025504 | L 1242 / F 235 / R 2595 | L 722 / F 137 / R 1450 |
| rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808 | L 161 / F 33 / R 255 | L 23 / F 5 / R 48 |
| rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | L 172 / F 32 / R 270 | L 17 / F 5 / R 26 |
| ryu_gemini-2.5-flash-nothinking_20251127_013142 | L 373 / F 32 / R 816 | L 17 / F 4 / R 30 |
| semver_gemini-2.5-flash-nothinking_20251127_013324 | L 557 / F 72 / R 1323 | L 62 / F 13 / R 119 |


## Validity Buckets

- raw_count: 17
- pass_gated_count: 17
- stabilized_pass_gated_count: 17
