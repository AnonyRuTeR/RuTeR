# RuTeR Test Help

## 一、目标与范围

1. 本文给出 RuTeR 的可执行测试手册，覆盖：
   1) 本地回归（mock online + replay + fixture）；
   2) 真实 online LLM 测试（使用真实 API URL/Model/Key）；
   3) 结果验收（退出码 + artifacts + 回归测试）。
2. 本文默认目标 fixture：`tests/humantime-e0433-gemini-fixture`。
3. 目标问题位置：`tests/humantime-e0433-gemini-fixture/src/duration.rs` 的 E0433 场景。

---

## 二、前置条件

1. 在仓库根目录执行命令。
2. Rust 工具链可用：

```bash
cargo --version
```

3. 在线模式必需环境变量：
   1) `RUTER_LLM_API_URL`
   2) `RUTER_LLM_MODEL`
   3) `RUTER_LLM_API_KEY`

说明：程序启动时会默认尝试读取仓库根 `.env`。若你希望显式覆盖，直接在 shell `export` 即可（显式环境变量优先）。

---

## 三、快速命令索引

1. 本地回归（不访问真实外网）：

```bash
bash scripts/run_regression_m3.sh
```

2. 真实 fixture 基线（确认当前有待修复错误）：

```bash
cargo test --manifest-path tests/humantime-e0433-gemini-fixture/Cargo.toml
```

3. 真实 online dry-run（不写回源文件）：

```bash
cargo run -- fix tests/humantime-e0433-gemini-fixture \
  --enable-llm \
  --llm-mode online \
  --llm-debug-dump-full-io
```

4. 真实 online apply（写回源文件）：

```bash
cargo run -- --apply fix tests/humantime-e0433-gemini-fixture \
  --enable-llm \
  --llm-mode online \
  --llm-debug-dump-full-io
```

---

## 四、推荐测试流程（真实 online）

1. 阶段 A：确认问题可复现
   1) 跑基线：

```bash
cargo test --manifest-path tests/humantime-e0433-gemini-fixture/Cargo.toml
```

   2) 预期：出现 E0433，且主要集中在 `src/duration.rs`。

2. 阶段 B：先 dry-run 再决定 apply
   1) 执行 dry-run online fix（见第三节命令 3）。
   2) 关注退出码：
      1) `0`：本轮已闭环；
      2) `7`：部分未闭环（但流程正常，需看 artifacts 归因）。

3. 阶段 C：检查 artifacts 证据
   1) 必看文件：
      1) `/tmp/rug_m3_real_llm/6_summary.json`
      2) `/tmp/rug_m3_real_llm/4_llm_attempts.json`
      3) `/tmp/rug_m3_real_llm/verify/4_report.json`
      4) `/tmp/rug_m3_real_llm/4_llm_contexts.json`
      5) `/tmp/rug_m3_real_llm/4_llm_io_debug.json`（开启 `--llm-debug-dump-full-io` 时）
   2) 快速检查示例：

```bash
jq '.mode, (.attempts | length)' /tmp/rug_m3_real_llm/4_llm_attempts.json
jq '.exit_code, .final_status, .unresolved_functions' /tmp/rug_m3_real_llm/6_summary.json
jq '.[0] | {function_id, round, request_user_prompt, response_content, request_error}' /tmp/rug_m3_real_llm/4_llm_io_debug.json
```

   3) 协议与 Prompt 验收点（`4_llm_io_debug.json`）：
      1) `request_user_prompt` 不包含 `Round:` / `Target:` / `Output contract:` / `Acceptance:`；
      2) `request_user_prompt` 的 `Location` 路径应为 `<CRATE_ROOT>/src/...`（不应出现绝对路径）；
      3) `response_content` 优先应包含 ` ```rust ` 代码块（JSON 仅兼容回退）。

4. 阶段 D：确认后 apply
   1) 执行 apply online fix（见第三节命令 4）。
   2) 回归测试：

```bash
cargo test --manifest-path tests/humantime-e0433-gemini-fixture/Cargo.toml
```

---

## 五、退出码与判定

1. `0`：流程成功闭环。
2. `7`：partial pending LLM（有未闭环函数，需看 handoff/attempts/report 继续定位）。
3. 非 `0/7`：通常是参数/工件/执行错误，需要先修复运行条件。

---

## 六、在线失败排查（高频）

1. `llm online mode requires api key via RUTER_LLM_API_KEY`
   - 原因：在线模式缺 key。
   - 处理：检查 `.env` 与当前 shell 是否生效，或显式 `export`。
2. `LLM_REQUEST_FAILED`
   - 常见证据：401/429/5xx/timeout。
   - 处理：看 `4_llm_attempts.json` 的 `phase=request`、`failure_detail`。
3. `CONTEXT_TOO_LARGE`
   - 原因：函数/上下文超过预算。
   - 处理：调大 `--llm-context-max-chars` 或缩小目标范围。

---

## 七、本地稳定回归矩阵（推荐每次改动后执行）

1. replay 与历史 llm 流程回归：

```bash
cargo test --test cli_e0433_flow llm_
```

2. online 异常矩阵（mock）：

```bash
cargo test --test cli_llm_online_flow
```

3. fixture 回归：

```bash
cargo test --test fixture_e0433_regression
```

4. R1 可达性回归：

```bash
cargo test unresolved_head_
```

5. 全量回归：

```bash
cargo test
```

---

## 八、附加说明

1. 配置优先级：`CLI > ENV > TOML > 默认值`。
2. 关键默认值：
   1) `topk=3`
   2) `llm.timeout_secs=60`
   3) `llm.max_rounds=3`
   4) `llm.context.max_chars=12000`
   5) `llm.max_candidates_per_round=3`
3. 安全红线不变：仅测试代码、仅目标函数域、签名/属性不变、必须通过验证闭环。
