# Coverage Recovery Fixed-Model Cross-Crate (gemini-2.5-flash-nothinking)

This is the only cross-crate slice intended for main-text effectiveness arguments.

## Raw Observed

| crate | run_id | gate status | original generation total | original success assets | failed generation attempts | normalized repaired | compile-executable repaired | original success delta (L/F/R) | repaired delta (L/F/R) |
|---|---|---|---:|---:|---:|---:|---:|---|---|
| humantime | humantime_gemini-2.5-flash-nothinking_20251109_134926 | stabilized_pass_gated | 110 | 25 | 85 | 48 | 0 | L 266 / F 44 / R 570 | L 0 / F 0 / R 0 |
| itoa | itoa_gemini-2.5-flash-nothinking_20251127_010109 | stabilized_pass_gated | 25 | 14 | 11 | 8 | 8 | L 222 / F 29 / R 513 | L 212 / F 29 / R 505 |
| log | log_gemini-2.5-flash-nothinking_20251127_025911 | stabilized_pass_gated | 178 | 33 | 145 | 41 | 10 | L 230 / F 47 / R 439 | L 121 / F 10 / R 141 |
| mio | mio_gemini-2.5-flash-nothinking_20251127_012706 | stabilized_pass_gated | 179 | 4 | 175 | 6 | 6 | L 45 / F 7 / R 98 | L 24 / F 4 / R 59 |
| rand | rand_gemini-2.5-flash-nothinking_20251127_025504 | stabilized_pass_gated | 496 | 141 | 355 | 165 | 2 | L 1242 / F 235 / R 2595 | L 14 / F 3 / R 26 |
| rust-crc32fast | rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808 | stabilized_pass_gated | 35 | 26 | 9 | 9 | 0 | L 161 / F 33 / R 255 | L 0 / F 0 / R 0 |
| rustc-demangle | rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | stabilized_pass_gated | 161 | 11 | 150 | 19 | 2 | L 172 / F 32 / R 270 | L 17 / F 2 / R 18 |
| ryu | ryu_gemini-2.5-flash-nothinking_20251127_013142 | stabilized_pass_gated | 48 | 34 | 14 | 7 | 0 | L 373 / F 32 / R 816 | L 0 / F 0 / R 0 |
| semver | semver_gemini-2.5-flash-nothinking_20251127_013324 | stabilized_pass_gated | 94 | 39 | 55 | 10 | 0 | L 557 / F 72 / R 1323 | L 0 / F 0 / R 0 |

## Stabilized Pass-Gated Conservative

| crate | run_id | original generation total | original success assets | failed generation attempts | normalized repaired | compile-executable repaired | original success delta (L/F/R) | repaired delta (L/F/R) |
|---|---|---:|---:|---:|---:|---:|---|---|
| humantime | humantime_gemini-2.5-flash-nothinking_20251109_134926 | 110 | 25 | 85 | 48 | 0 | L 266 / F 44 / R 570 | L 0 / F 0 / R 0 |
| itoa | itoa_gemini-2.5-flash-nothinking_20251127_010109 | 25 | 14 | 11 | 8 | 8 | L 222 / F 29 / R 513 | L 212 / F 29 / R 505 |
| log | log_gemini-2.5-flash-nothinking_20251127_025911 | 178 | 33 | 145 | 41 | 10 | L 230 / F 47 / R 439 | L 121 / F 10 / R 141 |
| mio | mio_gemini-2.5-flash-nothinking_20251127_012706 | 179 | 4 | 175 | 6 | 6 | L 45 / F 7 / R 98 | L 24 / F 4 / R 59 |
| rand | rand_gemini-2.5-flash-nothinking_20251127_025504 | 496 | 141 | 355 | 165 | 2 | L 1242 / F 235 / R 2595 | L 14 / F 3 / R 26 |
| rust-crc32fast | rust-crc32fast_gemini-2.5-flash-nothinking_20251109_235808 | 35 | 26 | 9 | 9 | 0 | L 161 / F 33 / R 255 | L 0 / F 0 / R 0 |
| rustc-demangle | rustc-demangle_gemini-2.5-flash-nothinking_20251109_233158 | 161 | 11 | 150 | 19 | 2 | L 172 / F 32 / R 270 | L 17 / F 2 / R 18 |
| ryu | ryu_gemini-2.5-flash-nothinking_20251127_013142 | 48 | 34 | 14 | 7 | 0 | L 373 / F 32 / R 816 | L 0 / F 0 / R 0 |
| semver | semver_gemini-2.5-flash-nothinking_20251127_013324 | 94 | 39 | 55 | 10 | 0 | L 557 / F 72 / R 1323 | L 0 / F 0 / R 0 |
