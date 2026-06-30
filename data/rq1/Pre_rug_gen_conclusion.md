# RUG Generation Study Summary

## Purpose

This study characterizes failures produced by RUG-based Rust unit-test generation before applying RuTeR. It provides the empirical motivation for a post-generation repair stage: many generated tests fail at compilation time, and the dominant failures are structured compiler diagnostics rather than arbitrary runtime behavior.

## Experimental Objects

The study uses 18 RUG runs:

- Fixed-model cross-crate runs: `gemini-2.5-flash-nothinking` across 10 crates.
- Fixed-crate cross-model runs: `humantime` across 9 models.
- The shared `humantime_gemini-2.5-flash-nothinking` run appears in both views.

Crates:

| Crate | Role |
|---|---|
| `humantime` | Fixed-crate cross-model target and small API-heavy crate. |
| `itoa`, `ryu`, `rust-crc32fast` | Small pure-function or low-level utility crates. |
| `log`, `semver`, `rustc-demangle` | Small to medium crates with traits, parsing or state-machine behavior. |
| `mio`, `rand`, `chrono` | Larger or more complex crates. |

Models in the `humantime` cross-model slice:

`claude-3-5-haiku-20241022`, `deepseek-v3`, `gemini-2.5-flash-nothinking`, `gemini-2.5-flash-thinking`, `gpt-3.5-turbo`, `gpt-4.1-mini`, `gpt-4.1-nano`, `gpt-4o-mini`, `gpt-5-nano`.

## Method

Each RUG run records generation attempts, injected test code, compile outcome and compiler output. The analysis groups results along two axes:

- Cross-crate: one model over multiple crates, to estimate crate-complexity effects.
- Cross-model: one crate over multiple models, to estimate model-choice effects.

Failures are summarized by failure category and by Rust compiler error code. These error-code distributions are the main input for deciding which failure modes RuTeR should target first.

## Data

Primary files:

| Path | Contents |
|---|---|
| `rug_runs/*/detailed_log.json` | Attempt-level RUG generation records, including generated code and compiler output. |
| `local_analysis_output/*/stats_raw.json` | Parsed error-code and failure-category statistics. |
| `rug_reports/` | Human-readable run summaries. |

Scale:

- Fixed-model cross-crate view: 3,183 generated samples.
- Fixed-crate cross-model view: 1,277 generated `humantime` samples.
- Total RUG run directories retained in `eval/rug_runs`: 18.

## Results

### Fixed-Model Cross-Crate View

For `gemini-2.5-flash-nothinking` across 10 crates:

- Generated samples: 3,183.
- Compile successes: 731.
- Compile success rate: 22.97%.
- Compile failure rate: 77.03%.

Compile success varies strongly by crate:

| Crate | Compile success rate |
|---|---:|
| `rust-crc32fast` | 74.3% |
| `ryu` | 70.8% |
| `itoa` | 56.0% |
| `semver` | 41.5% |
| `chrono` | 33.5% |
| `rand` | 28.4% |
| `humantime` | 22.7% |
| `log` | 18.5% |
| `rustc-demangle` | 6.8% |
| `mio` | 2.2% |

The spread between `rust-crc32fast` and `mio` is 72.1 percentage points, indicating that crate/API structure dominates much of the observed difficulty.

### Fixed-Crate Cross-Model View

For `humantime` across 9 models, all models remain below 30% compile success:

| Model | Compile success rate |
|---|---:|
| `gemini-2.5-flash-thinking` | 26.3% |
| `gemini-2.5-flash-nothinking` | 22.7% |
| `deepseek-v3` | 20.8% |
| `gpt-4.1-mini` | 17.2% |
| `gpt-3.5-turbo` | 13.5% |
| `gpt-4.1-nano` | 11.7% |
| `gpt-4o-mini` | 11.7% |
| `gpt-5-nano` | 11.6% |
| `claude-3-5-haiku-20241022` | 11.5% |

The model spread is smaller than the crate spread, so changing the model alone does not remove the Rust compilation bottleneck.

### Failure Structure

In the cross-crate view, 93.27% of failures are Rust compiler errors. The most frequent error codes are:

| Rank | Error code | Count | Main meaning |
|---:|---|---:|---|
| 1 | `E0433` | 1,210 | unresolved crate/module/type path |
| 2 | `E0432` | 880 | unresolved import |
| 3 | `E0599` | 549 | missing method or field |
| 4 | `E0308` | 207 | type mismatch |
| 5 | `E0603` | 185 | private item access |
| 6 | `E0560` | 120 | unknown struct field |
| 7 | `E0061` | 110 | wrong number of function arguments |
| 8 | `E0277` | 81 | trait bound not satisfied |
| 9 | `E0412` | 56 | type not in scope |
| 10 | `E0658` | 46 | unstable feature |

Path/import errors (`E0433`, `E0432`, `E0412`) account for a large fraction of failures and are well-suited to structured repair because compiler diagnostics often contain candidate paths or scope hints.

## Takeaways

- RUG-generated Rust tests fail compilation frequently; this is observable across crates and models.
- Failure modes are concentrated in compiler-diagnostic categories, not random output noise.
- Path/import errors are the strongest initial target for automated repair.
- Crate complexity is a stronger driver of failure than model choice in this dataset.

