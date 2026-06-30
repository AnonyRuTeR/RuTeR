# Coverage Recovery Fixed-Model Cross-Crate (gemini-2.5-flash-nothinking)

This is the only cross-crate slice intended for main-text effectiveness arguments.

## Raw Observed

| crate | run_id | gate status | original generation total | original success assets | failed generation attempts | normalized repaired | compile-executable repaired | original success delta (L/F/R) | repaired delta (L/F/R) |
|---|---|---|---:|---:|---:|---:|---:|---|---|
| humantime | humantime_gemini-2.5-flash-nothinking_20251109_134926 | stabilized_pass_gated | 110 | 25 | 85 | 20 | 20 | L 266 / F 44 / R 570 | L 67 / F 15 / R 147 |
| itoa | itoa_gemini-2.5-flash-nothinking_20251127_010109 | stabilized_pass_gated | 25 | 14 | 11 | 4 | 4 | L 222 / F 29 / R 513 | L 59 / F 10 / R 128 |
| log | log_gemini-2.5-flash-nothinking_20251127_025911 | stabilized_pass_gated | 178 | 33 | 145 | 27 | 27 | L 230 / F 47 / R 439 | L 223 / F 36 / R 348 |
| mio | mio_gemini-2.5-flash-nothinking_20251127_012706 | stabilized_pass_gated | 179 | 4 | 175 | 26 | 21 | L 45 / F 7 / R 98 | L 100 / F 23 / R 166 |
| rand | rand_gemini-2.5-flash-nothinking_20251127_025504 | stabilized_pass_gated | 496 | 141 | 355 | 95 | 95 | L 1242 / F 235 / R 2595 | L 722 / F 137 / R 1450 |
| rust-crc32fast | rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808 | stabilized_pass_gated | 35 | 26 | 9 | 5 | 5 | L 161 / F 33 / R 255 | L 23 / F 5 / R 48 |
| rustc-demangle | rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | stabilized_pass_gated | 161 | 11 | 150 | 6 | 6 | L 172 / F 32 / R 270 | L 17 / F 5 / R 26 |
| ryu | ryu_gemini-2.5-flash-nothinking_20251127_013142 | stabilized_pass_gated | 48 | 34 | 14 | 4 | 4 | L 373 / F 32 / R 816 | L 17 / F 4 / R 30 |
| semver | semver_gemini-2.5-flash-nothinking_20251127_013324 | stabilized_pass_gated | 94 | 39 | 55 | 10 | 10 | L 557 / F 72 / R 1323 | L 62 / F 13 / R 119 |

## Stabilized Pass-Gated Conservative

| crate | run_id | original generation total | original success assets | failed generation attempts | normalized repaired | compile-executable repaired | original success delta (L/F/R) | repaired delta (L/F/R) |
|---|---|---:|---:|---:|---:|---:|---|---|
| humantime | humantime_gemini-2.5-flash-nothinking_20251109_134926 | 110 | 25 | 85 | 20 | 20 | L 266 / F 44 / R 570 | L 67 / F 15 / R 147 |
| itoa | itoa_gemini-2.5-flash-nothinking_20251127_010109 | 25 | 14 | 11 | 4 | 4 | L 222 / F 29 / R 513 | L 59 / F 10 / R 128 |
| log | log_gemini-2.5-flash-nothinking_20251127_025911 | 178 | 33 | 145 | 27 | 27 | L 230 / F 47 / R 439 | L 223 / F 36 / R 348 |
| mio | mio_gemini-2.5-flash-nothinking_20251127_012706 | 179 | 4 | 175 | 26 | 21 | L 45 / F 7 / R 98 | L 100 / F 23 / R 166 |
| rand | rand_gemini-2.5-flash-nothinking_20251127_025504 | 496 | 141 | 355 | 95 | 95 | L 1242 / F 235 / R 2595 | L 722 / F 137 / R 1450 |
| rust-crc32fast | rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808 | 35 | 26 | 9 | 5 | 5 | L 161 / F 33 / R 255 | L 23 / F 5 / R 48 |
| rustc-demangle | rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | 161 | 11 | 150 | 6 | 6 | L 172 / F 32 / R 270 | L 17 / F 5 / R 26 |
| ryu | ryu_gemini-2.5-flash-nothinking_20251127_013142 | 48 | 34 | 14 | 4 | 4 | L 373 / F 32 / R 816 | L 17 / F 4 / R 30 |
| semver | semver_gemini-2.5-flash-nothinking_20251127_013324 | 94 | 39 | 55 | 10 | 10 | L 557 / F 72 / R 1323 | L 62 / F 13 / R 119 |
