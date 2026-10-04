# Paired Claude Code token-cost experiment

This supplementary experiment reruns the complete frozen population: **1,858
failed generation attempts, 566 generation-run/function units, nine crates**.
It does not replace the submitted-paper RQ1/RQ2/RQ3 aggregates.

## Protocol

Both methods receive the same failed test in independent copies of the same
clean crate. RuTeR runs first to establish the paired case manifest; Claude Code
does not receive RuTeR's patches or outcomes.

- Same gateway (`https://4sapi.org`), API account and repair-model alias
  `gemini-2.5-flash-nothinking`; the gateway returns `gemini-2.5-flash`.
  This is Claude Code as an agent harness with Gemini, not a Claude-model baseline.
- RuTeR: topk 3, at most three online LLM rounds per unresolved function,
  900-second case timeout and 120-second LLM-request timeout.
- Claude Code 2.1.281: noninteractive `claude -p` in each reconstructed crate,
  at most 20 turns, 900-second case timeout and 600-second outer Cargo timeout.
  The full run used two isolated workers, each with two Cargo build jobs.
- The checked-in prompt includes the exact injected test and current compiler
  errors, requests compilation repair, permits context inspection when needed,
  and forbids production/dependency changes and removal or weakening of tests.
- Tools: Read, Edit, Glob, Grep and restricted Cargo Bash commands. No web tools,
  external MCP, persistent sessions or slash commands. A Stop hook checks
  `cargo check --tests`; the outer runner independently checks the final result.
- RuTeR strict success requires passing patch verification, zero remaining
  compiler errors and zero unresolved test functions. Claude strict success
  requires a passing final compile check, zero errors, valid edit scope and
  structural test-integrity guards. These guards are not a proof of semantic
  equivalence and compilation success does not establish runtime correctness.

The runners support resumable first sweeps and deferred retries of infrastructure
failures (including HTTP 402). Archived invalid trials are retained locally.
Ordinary repair failures are not retried to search for a favorable outcome.

## Token accounting and budget definition

RuTeR records each online request in `4_llm_usage.json`. Claude Code uses a
loopback Anthropic-format proxy that forwards requests to the gateway and logs
usage metadata, not prompt/response bodies or authorization headers. Parallel
workers have distinct proxy ports, workspaces and Cargo target directories.

The comparable primary cost is provider OpenAI-style input plus output tokens.
For Claude, this is `billing_usage.openai_usage.total_tokens`, not the sum of
Anthropic cache/read/write counters. Do not add cached input a second time.
All requests in each final provider-complete trial are included, including
agent overhead and ordinary repair failures. Valid rule-only RuTeR cases have
zero tokens and remain in the mean. Missing usage is never imputed as zero.

Preflights and archived infrastructure retries are excluded from this primary
method-cost comparison; their operational spend is a different quantity.
The released cohort has complete comparable provider usage for every paired
case. Gateway-reported tokens are not an independently verified monetary bill.

For multiplier `m`, a Claude attempt succeeds in the budgeted analysis only if
it is a strict repair and `Claude_tokens <= m * mean_RuTeR_tokens`. The reference
mean uses all 1,858 final RuTeR trials, including failures and valid zeros.
Use the unrounded mean; the exporter compares integers exactly:
`Claude_tokens * 1858 <= m * sum_RuTeR_tokens`.

**These are post-hoc cutoffs on completed runs, not online early stopping.**
They cannot establish the success rate of a different agent stopped mid-run.
The per-case cap is not a cap on the sum of all attempts belonging to a function.
All cases and functions remain in the denominator.

A function is salvaged if any of its attempts satisfies the applicable success
criterion. Group by `(run_id, node_id)`; do not merge different generation runs.
The public table replaces these keys with stable pseudonymous function/run IDs.

## Released results

| Method / post-hoc cap | Repaired attempts | Repair rate | Salvaged functions | Salvage rate |
| --- | ---: | ---: | ---: | ---: |
| RuTeR (no cap) | 872 / 1,858 | 46.93% | 375 / 566 | 66.25% |
| Claude Code, 5× RuTeR mean | 251 / 1,858 | 13.51% | 171 / 566 | 30.21% |
| Claude Code, 10× RuTeR mean | 600 / 1,858 | 32.29% | 320 / 566 | 56.54% |
| Claude Code (no cap) | 1,297 / 1,858 | 69.81% | 495 / 566 | 87.46% |

RuTeR consumes 8,663,677 tokens (mean 4,662.90); Claude Code consumes 218,149,284
(mean 117,410.81), a **25.18×** full-run ratio. The exact 5× and 10× caps are
23,314.523681... and 46,629.047362... tokens (integer cutoffs 23,314 and 46,629).
Unrestricted Claude Code has higher repair effectiveness; budgeted rows should
not be presented as unrestricted agent performance.

The earlier submitted-paper aggregates report 869 repaired attempts and 366
salvaged functions for Full RuTeR. The new paired rerun reports 872 and 375.
These are separate runs; the original tables are deliberately unchanged.

## Recompute without APIs

From the repository root:

```bash
python3 experiments/scripts/export_claude_code_results.py \
  --cases-csv data/claude_code/paired_cases.csv
```

The workbook `data/claude_code/claude_code_results.xlsx` contains the same
1,858 per-case input/output/total counts and success flags, a verified summary,
566 function groups and cached Excel formulas. Its Calculations sheet exposes
the multipliers, means, token ratio and attempt/function rates. Changing the
multipliers recomputes the Cases and Functions sheets; Summary is a fixed
verified snapshot. No large logs are needed to reproduce these statistics.

To regenerate the workbook (optional dependency):

```bash
python3 -m pip install -r experiments/requirements-token-cost.txt
python3 experiments/scripts/export_claude_code_results.py \
  --cases-csv data/claude_code/paired_cases.csv \
  --out experiments/artifacts/recomputed
```

## Run new repairs (requires separate inputs and paid API access)

Requirements: Python 3.10+, the RuTeR release binary, Cargo and the experiment's
crate toolchains/dependencies, and Claude Code with the CLI options used by the
runner. Pin the reported Claude Code version when comparing against this run.

This compact extension **does not bundle raw failed tests, clean crate snapshots
or execution logs**. The existing aggregate-only data and workbook are sufficient
to recompute reported statistics, but not to reconstruct those inputs. Supply
the exact frozen failures and matching crate snapshots separately to reproduce
repairs; different inputs, dependency versions or current model behavior yield
a new experiment. No claim of bit-for-bit repair reproducibility is made.

Required manifest structure:

```json
{
  "attempts": [{
    "attempt_uid": "unique-attempt-identifier",
    "run_id": "generation-run-identifier",
    "node_id": "function-identifier",
    "crate": "crate-directory-name",
    "model": "generation-model-name",
    "src_path": "src/lib.rs",
    "attempt_seq": 1,
    "error_codes": ["E0433"],
    "injected_code": "#[cfg(test)] mod generated_test { /* exact failed test */ }"
  }]
}
```

Crate snapshots are stored as `<clean-crates-root>/<crate>/Cargo.toml` with their
source files. Source paths must be relative and must not contain `..`.
The cohort is frozen before either method runs; never select cases by outcomes.

```bash
cargo build --manifest-path ruter/Cargo.toml --release --locked --bin ruter

# Enter a secret interactively; do not put it in commands, configs or Git.
read -rs FULL_PAIRED_API_KEY
export FULL_PAIRED_API_KEY

# A preflight alone makes small paid calls to both API protocols.
python3 experiments/scripts/run_full_paired_token_experiment.py --preflight-only

# First complete both sweeps, then retry only infrastructure-invalid cases.
python3 experiments/scripts/run_full_paired_token_experiment.py \
  --manifest /path/to/frozen_attempt_manifest.json \
  --clean-crates-root /path/to/clean-crates \
  --claude-workers 2 --defer-infrastructure-retries
unset FULL_PAIRED_API_KEY
```

The default output is `experiments/artifacts/token_cost/` (Git-ignored).
Rerun the same command to resume; protocol hashes reject incompatible reuse.
`--reuse-ruter-results-from` and `--reuse-claude-results-from` are opt-in and
accept only compatible completed trials, never arbitrary favorable results.
Two or three workers can also be selected using the parallel Claude runner.

Export only the small allowlisted data from new completed results:

```bash
python3 experiments/scripts/export_claude_code_results.py \
  --ruter-root experiments/artifacts/token_cost/ruter \
  --claude-root experiments/artifacts/token_cost/claude \
  --out experiments/artifacts/public-export
```

Audit new exports before publishing. Do not commit raw manifests, compiler
diagnostics, stream logs, workspaces, local settings or account credentials.

## Offline tests

```bash
python3 -m unittest discover -s experiments/tests -p 'test_*.py'
```
