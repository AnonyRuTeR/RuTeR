# Coverage Recovery Crate Table

This table is descriptive only. It aggregates selected runs by crate, so it must not be used for cross-crate claims when a crate has multiple model runs.

## Raw Observed

| crate | run total | positive repaired runs | original success delta (L/F/R) | repaired delta (L/F/R) | models |
|---|---:|---:|---|---|---|
| humantime | 9 | 9 | L 1408 / F 292 / R 2853 | L 1084 / F 219 / R 2119 | claude-3-5-haiku-20241022, deepseek-v3, gemini-2.5-flash-nothinking, gemini-2.5-flash-thinking, gpt-3.5-turbo, gpt-4.1-mini, gpt-4.1-nano, gpt-4o-mini, gpt-5-nano |
| itoa | 1 | 1 | L 222 / F 29 / R 513 | L 59 / F 10 / R 128 | gemini-2.5-flash-nothinking |
| log | 1 | 1 | L 230 / F 47 / R 439 | L 223 / F 36 / R 348 | gemini-2.5-flash-nothinking |
| mio | 1 | 1 | L 45 / F 7 / R 98 | L 100 / F 23 / R 166 | gemini-2.5-flash-nothinking |
| rand | 1 | 1 | L 1242 / F 235 / R 2595 | L 722 / F 137 / R 1450 | gemini-2.5-flash-nothinking |
| rust-crc32fast | 1 | 1 | L 161 / F 33 / R 255 | L 23 / F 5 / R 48 | gemini-2.5-flash-nothinking |
| rustc-demangle | 1 | 1 | L 172 / F 32 / R 270 | L 17 / F 5 / R 26 | gemini-2.5-flash-nothinking |
| ryu | 1 | 1 | L 373 / F 32 / R 816 | L 17 / F 4 / R 30 | gemini-2.5-flash-nothinking |
| semver | 1 | 1 | L 557 / F 72 / R 1323 | L 62 / F 13 / R 119 | gemini-2.5-flash-nothinking |

## Stabilized Pass-Gated Conservative

| crate | run total | positive repaired runs | original success delta (L/F/R) | repaired delta (L/F/R) | models |
|---|---:|---:|---|---|---|
| humantime | 9 | 9 | L 1408 / F 292 / R 2853 | L 1084 / F 219 / R 2119 | claude-3-5-haiku-20241022, deepseek-v3, gemini-2.5-flash-nothinking, gemini-2.5-flash-thinking, gpt-3.5-turbo, gpt-4.1-mini, gpt-4.1-nano, gpt-4o-mini, gpt-5-nano |
| itoa | 1 | 1 | L 222 / F 29 / R 513 | L 59 / F 10 / R 128 | gemini-2.5-flash-nothinking |
| log | 1 | 1 | L 230 / F 47 / R 439 | L 223 / F 36 / R 348 | gemini-2.5-flash-nothinking |
| mio | 1 | 1 | L 45 / F 7 / R 98 | L 100 / F 23 / R 166 | gemini-2.5-flash-nothinking |
| rand | 1 | 1 | L 1242 / F 235 / R 2595 | L 722 / F 137 / R 1450 | gemini-2.5-flash-nothinking |
| rust-crc32fast | 1 | 1 | L 161 / F 33 / R 255 | L 23 / F 5 / R 48 | gemini-2.5-flash-nothinking |
| rustc-demangle | 1 | 1 | L 172 / F 32 / R 270 | L 17 / F 5 / R 26 | gemini-2.5-flash-nothinking |
| ryu | 1 | 1 | L 373 / F 32 / R 816 | L 17 / F 4 / R 30 | gemini-2.5-flash-nothinking |
| semver | 1 | 1 | L 557 / F 72 / R 1323 | L 62 / F 13 / R 119 | gemini-2.5-flash-nothinking |
