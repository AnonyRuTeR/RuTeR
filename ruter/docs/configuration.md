# RuTeR Configuration

This document describes runtime configuration for M3 LLM pipeline.

## 1. Priority

1. CLI arguments
2. Environment variables (`RUTER_*`)
3. `ruter.toml`
4. Built-in defaults

## 2. Core Defaults

- `topk.size=3`
- `llm.enabled=false`
- `llm.mode=replay`
- `llm.timeout_secs=60`
- `llm.max_rounds=3`
- `llm.max_candidates_per_round=3`
- `llm.context.max_chars=12000`
- `llm.context.target_fn_hard_limit_chars=8000`
- `llm.raw_response_max_chars=4096`
- `llm.debug_dump_full_io=false`

## 3. CLI Arguments

- `--config <PATH>`
- `--topk <N>`
- `--enable-llm`
- `--llm-mode <replay|online>`
- `--llm-replay-file <PATH>`
- `--llm-api-url <URL>`
- `--llm-model <MODEL>`
- `--llm-timeout-secs <SECS>`
- `--llm-max-rounds <N>`
- `--llm-max-candidates <N>`
- `--llm-context-max-chars <N>`
- `--llm-target-fn-hard-limit-chars <N>`
- `--llm-raw-excerpt-max-chars <N>`
- `--llm-debug-dump-full-io`

## 4. Environment Variables

- `RUTER_TOPK_SIZE`
- `RUTER_LLM_ENABLED`
- `RUTER_LLM_MODE`
- `RUTER_LLM_REPLAY_FILE`
- `RUTER_LLM_API_URL`
- `RUTER_LLM_MODEL`
- `RUTER_LLM_API_KEY`
- `RUTER_LLM_TIMEOUT_SECS`
- `RUTER_LLM_MAX_ROUNDS`
- `RUTER_LLM_MAX_CANDIDATES_PER_ROUND`
- `RUTER_LLM_CONTEXT_MAX_CHARS`
- `RUTER_LLM_TARGET_FN_HARD_LIMIT_CHARS`
- `RUTER_LLM_RAW_EXCERPT_MAX_CHARS`
- `RUTER_LLM_DEBUG_DUMP_FULL_IO`

## 5. Artifacts (M3)

- `4_llm_contexts.json`: context builder records per function-round.
- `4_llm_attempts.json`: attempt-level evidence and failure attribution.
- `4_llm_io_debug.json`: full prompt/response dump (only when debug option is enabled).
