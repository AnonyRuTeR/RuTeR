# Coverage Recovery Fixed-Model Cross-Crate (gemini-2.5-flash-nothinking)

This is the only cross-crate slice intended for main-text effectiveness arguments.

## Raw Observed

| crate | run_id | gate status | original generation total | original success assets | failed generation attempts | normalized repaired | compile-executable repaired | original success delta (L/F/R) | repaired delta (L/F/R) |
|---|---|---|---:|---:|---:|---:|---:|---|---|
| humantime | humantime_gemini-2.5-flash-nothinking_20251109_134926 | stabilized_pass_gated | 110 | 25 | 85 | 19 | 19 | L 266 / F 44 / R 571 | L 57 / F 13 / R 126 |
| itoa | itoa_gemini-2.5-flash-nothinking_20251127_010109 | stabilized_pass_gated | 25 | 14 | 11 | 4 | 4 | L 222 / F 29 / R 513 | L 37 / F 4 / R 96 |
| log | log_gemini-2.5-flash-nothinking_20251127_025911 | stabilized_pass_gated | 178 | 33 | 145 | 13 | 13 | L 230 / F 47 / R 439 | L 126 / F 15 / R 148 |
| mio | mio_gemini-2.5-flash-nothinking_20251127_012706 | stabilized_pass_gated | 179 | 4 | 175 | 35 | 5 | L 45 / F 7 / R 98 | L 17 / F 3 / R 37 |
| rand | rand_gemini-2.5-flash-nothinking_20251127_025504 | stabilized_pass_gated | 496 | 141 | 355 | 64 | 64 | L 1242 / F 235 / R 2595 | L 425 / F 82 / R 816 |
| rust-crc32fast | rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808 | stabilized_pass_gated | 35 | 26 | 9 | 9 | 9 | L 161 / F 33 / R 255 | L 42 / F 9 / R 88 |
| rustc-demangle | rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | stabilized_pass_gated | 161 | 11 | 150 | 4 | 4 | L 172 / F 32 / R 270 | L 8 / F 2 / R 13 |
| ryu | ryu_gemini-2.5-flash-nothinking_20251127_013142 | stabilized_pass_gated | 48 | 34 | 14 | 3 | 3 | L 373 / F 32 / R 816 | L 10 / F 2 / R 20 |
| semver | semver_gemini-2.5-flash-nothinking_20251127_013324 | stabilized_pass_gated | 94 | 39 | 55 | 2 | 2 | L 557 / F 72 / R 1323 | L 8 / F 2 / R 13 |

## Stabilized Pass-Gated Conservative

| crate | run_id | original generation total | original success assets | failed generation attempts | normalized repaired | compile-executable repaired | original success delta (L/F/R) | repaired delta (L/F/R) |
|---|---|---:|---:|---:|---:|---:|---|---|
| humantime | humantime_gemini-2.5-flash-nothinking_20251109_134926 | 110 | 25 | 85 | 19 | 19 | L 266 / F 44 / R 571 | L 57 / F 13 / R 126 |
| itoa | itoa_gemini-2.5-flash-nothinking_20251127_010109 | 25 | 14 | 11 | 4 | 4 | L 222 / F 29 / R 513 | L 37 / F 4 / R 96 |
| log | log_gemini-2.5-flash-nothinking_20251127_025911 | 178 | 33 | 145 | 13 | 13 | L 230 / F 47 / R 439 | L 126 / F 15 / R 148 |
| mio | mio_gemini-2.5-flash-nothinking_20251127_012706 | 179 | 4 | 175 | 35 | 5 | L 45 / F 7 / R 98 | L 17 / F 3 / R 37 |
| rand | rand_gemini-2.5-flash-nothinking_20251127_025504 | 496 | 141 | 355 | 64 | 64 | L 1242 / F 235 / R 2595 | L 425 / F 82 / R 816 |
| rust-crc32fast | rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808 | 35 | 26 | 9 | 9 | 9 | L 161 / F 33 / R 255 | L 42 / F 9 / R 88 |
| rustc-demangle | rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | 161 | 11 | 150 | 4 | 4 | L 172 / F 32 / R 270 | L 8 / F 2 / R 13 |
| ryu | ryu_gemini-2.5-flash-nothinking_20251127_013142 | 48 | 34 | 14 | 3 | 3 | L 373 / F 32 / R 816 | L 10 / F 2 / R 20 |
| semver | semver_gemini-2.5-flash-nothinking_20251127_013324 | 94 | 39 | 55 | 2 | 2 | L 557 / F 72 / R 1323 | L 8 / F 2 / R 13 |
