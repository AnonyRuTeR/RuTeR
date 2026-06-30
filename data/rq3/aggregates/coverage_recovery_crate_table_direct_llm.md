# Coverage Recovery Crate Table

This table is descriptive only. It aggregates selected runs by crate, so it must not be used for cross-crate claims when a crate has multiple model runs.

## Raw Observed

| crate | run total | positive repaired runs | original success delta (L/F/R) | repaired delta (L/F/R) | models |
|---|---:|---:|---|---|---|
| humantime | 9 | 9 | L 1408 / F 292 / R 2854 | L 542 / F 110 / R 1016 | claude-3-5-haiku-20241022, deepseek-v3, gemini-2.5-flash-nothinking, gemini-2.5-flash-thinking, gpt-3.5-turbo, gpt-4.1-mini, gpt-4.1-nano, gpt-4o-mini, gpt-5-nano |
| itoa | 1 | 1 | L 222 / F 29 / R 513 | L 37 / F 4 / R 96 | gemini-2.5-flash-nothinking |
| log | 1 | 1 | L 230 / F 47 / R 439 | L 126 / F 15 / R 148 | gemini-2.5-flash-nothinking |
| mio | 1 | 1 | L 45 / F 7 / R 98 | L 17 / F 3 / R 37 | gemini-2.5-flash-nothinking |
| rand | 1 | 1 | L 1242 / F 235 / R 2595 | L 425 / F 82 / R 816 | gemini-2.5-flash-nothinking |
| rust-crc32fast | 1 | 1 | L 161 / F 33 / R 255 | L 42 / F 9 / R 88 | gemini-2.5-flash-nothinking |
| rustc-demangle | 1 | 1 | L 172 / F 32 / R 270 | L 8 / F 2 / R 13 | gemini-2.5-flash-nothinking |
| ryu | 1 | 1 | L 373 / F 32 / R 816 | L 10 / F 2 / R 20 | gemini-2.5-flash-nothinking |
| semver | 1 | 1 | L 557 / F 72 / R 1323 | L 8 / F 2 / R 13 | gemini-2.5-flash-nothinking |

## Stabilized Pass-Gated Conservative

| crate | run total | positive repaired runs | original success delta (L/F/R) | repaired delta (L/F/R) | models |
|---|---:|---:|---|---|---|
| humantime | 9 | 9 | L 1408 / F 292 / R 2854 | L 542 / F 110 / R 1016 | claude-3-5-haiku-20241022, deepseek-v3, gemini-2.5-flash-nothinking, gemini-2.5-flash-thinking, gpt-3.5-turbo, gpt-4.1-mini, gpt-4.1-nano, gpt-4o-mini, gpt-5-nano |
| itoa | 1 | 1 | L 222 / F 29 / R 513 | L 37 / F 4 / R 96 | gemini-2.5-flash-nothinking |
| log | 1 | 1 | L 230 / F 47 / R 439 | L 126 / F 15 / R 148 | gemini-2.5-flash-nothinking |
| mio | 1 | 1 | L 45 / F 7 / R 98 | L 17 / F 3 / R 37 | gemini-2.5-flash-nothinking |
| rand | 1 | 1 | L 1242 / F 235 / R 2595 | L 425 / F 82 / R 816 | gemini-2.5-flash-nothinking |
| rust-crc32fast | 1 | 1 | L 161 / F 33 / R 255 | L 42 / F 9 / R 88 | gemini-2.5-flash-nothinking |
| rustc-demangle | 1 | 1 | L 172 / F 32 / R 270 | L 8 / F 2 / R 13 | gemini-2.5-flash-nothinking |
| ryu | 1 | 1 | L 373 / F 32 / R 816 | L 10 / F 2 / R 20 | gemini-2.5-flash-nothinking |
| semver | 1 | 1 | L 557 / F 72 / R 1323 | L 8 / F 2 / R 13 | gemini-2.5-flash-nothinking |
