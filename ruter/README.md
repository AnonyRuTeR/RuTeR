# ruter

**ruter** 是一款针对 Rust 单元测试代码的编译错误自动化修复框架，主要面向自动化工具生成的 Rust 测试函数。通过解析 `rustc` JSON 诊断输出，以**规则修复优先、LLM 辅助兜底**的分层策略，自动将编译错误收敛至零。

---

## 核心原则

| 原则 | 说明 |
|---|---|
| 测试代码专修 | 仅修改 `#[test]` / `#[cfg(test)]` / `tests/` 下的代码 |
| 函数级粒度 | 分析、修复、验证均以单个测试函数为最小操作单元 |
| 验证门禁 | 任何修复必须通过编译验证：目标函数错误清零 + 非目标函数无回归 |
| 预算收敛 | LLM 每个函数独立轮次预算（默认 3 轮），耗尽即停止 |
| 干运行安全 | 默认不写回源文件，需显式 `--apply` 才执行写入 |

---

## 架构概览

```
Compile → Analyze → Plan → Verify → [LLM] → Apply / Summarize
```

- **Rule Patchers**：静态规则直接输出 `FixAction`，通过编译验证后写入计划；或者输出分析结论，注入 LLM Patcher 上下文。
- **Preflight Interceptors**：五维静态分析（LowValue / E0599 / E0308 / E0432 / E0560），决定函数路由（`ContinueToLlm` / `SkipLlmCommentOut`）。
- **LLM Patcher**：基于 `FunctionContextBundleV1` 构建上下文（L0 元数据 + L1 诊断 + L2 语义），支持 Replay 和 Online 两种执行模式。
- **Coordinator / Transformer**：全局候选排序、冲突检测、字节级代码替换。
- **Artifacts**：每阶段产出结构化 JSON，支持离线分析与回放。

详细设计见 [PatcherDesign.md](PatcherDesign.md)。

---

## 环境要求

- Rust 工具链：**Edition 2024**（`rustup` 安装，stable channel）
- `cargo` 在 `PATH` 中可用（用于编译目标 crate 以及 ruter 自身构建）

---

## 安装

```bash
git clone https://github.com/i4c9s3e6/RuTeR.git
cd ruter
cargo build --release
```

编译产物位于 `target/release/ruter`，可将其复制到 `PATH` 中任意目录：

```bash
cp target/release/ruter ~/.local/bin/
```

---

## 配置

配置优先级：**CLI 参数 > 环境变量 > TOML 配置文件 > 内置默认值**。

### TOML 配置文件

项目内提供 `ruter.toml` 作为默认配置：

```toml
[topk]
size = 3

[llm]
enabled = false
mode = "replay"       # "replay" | "online"
timeout_secs = 60
max_rounds = 3
max_candidates_per_round = 3
debug_dump_full_io = false

[llm.context]
max_chars = 12000
target_fn_hard_limit_chars = 8000
```

通过 `--config <path>` 指定自定义配置文件路径。

### LLM Online 模式环境变量

使用 Online 模式时，通过环境变量（或 `.env` 文件）提供凭据：

```bash
LLM_API_KEY=sk-...
LLM_API_URL=https://api.openai.com/v1/chat/completions
LLM_MODEL=gpt-4o
```

---

## 使用

### Fix 模式（一键修复）

```bash
# 默认干运行（不写回文件），输出 Diff 和 Summary
ruter fix <crate_path> --artifacts-dir <artifacts_dir>

# 确认后写回源文件
ruter --apply fix <crate_path> --artifacts-dir <artifacts_dir>

# 启用 LLM（Online 模式）
ruter --apply fix <crate_path> \
  --enable-llm --llm-mode online \
  --llm-api-url $LLM_API_URL \
  --llm-model $LLM_MODEL \
  --artifacts-dir <artifacts_dir>
```

### Step 模式（逐阶段调试）

```bash
# 可用的 stage: compile | analyze | plan | verify | apply
ruter step compile <crate_path> --artifacts-dir <dir>
ruter step analyze <crate_path> --artifacts-dir <dir>
ruter step plan    <crate_path> --artifacts-dir <dir>
ruter step verify  <crate_path> --artifacts-dir <dir>
ruter step apply   <crate_path> --artifacts-dir <dir>
```

### 常用选项

| 选项 | 默认值 | 说明 |
|---|---|---|
| `--apply` | false | 写回修复后的源文件（否则仅输出 Diff） |
| `--topk <n>` | 3 | 每个诊断保留的候选修复数量 |
| `--enable-llm` | false | 启用 LLM 修复路径 |
| `--llm-mode` | replay | `replay`（本地 JSON 文件）或 `online` |
| `--llm-replay-file <path>` | — | Replay 模式下的响应文件路径 |
| `--llm-max-rounds <n>` | 3 | 每函数最大 LLM 轮次 |
| `--llm-context-max-chars <n>` | 12000 | 上下文字符预算上限 |
| `--llm-debug-dump-full-io` | false | 将完整 Prompt/Response 写入 Artifacts |
| `--artifacts-dir <path>` | — | Artifact JSON 输出目录 |
| `--diff-file <path>` | — | 将统一 Diff 输出到指定文件 |
| `--no-backup` | false | 写回时不生成 `.bak` 备份文件 |
| `-v` / `-vv` | — | 增加日志输出详细级别 |

---

## Artifacts

每次运行在 `--artifacts-dir` 下产出结构化 JSON 文件，可用于离线分析与回归验证：

| 文件 | 内容 |
|---|---|
| `compile_output.json` | 原始 rustc NDJSON 诊断输出 |
| `analyze_result.json` | 函数级聚合诊断与分发决策 |
| `plan_result.json` | 全局修复候选计划 |
| `verify_result.json` | 验证结果与接纳/拒绝记录 |
| `llm_context_*.json` | 每函数的 `FunctionContextBundleV1` |
| `llm_io_*.json` | （`--llm-debug-dump-full-io`）完整 Prompt 与原始响应 |
| `run_summary.json` | 全流程汇总（接纳数 / 失败数 / 跳过数） |

---

