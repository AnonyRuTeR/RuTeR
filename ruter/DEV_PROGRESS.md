# RuTeR 开发实践进度

## 一、设计决策（已锁定）

1. 仅允许修改测试代码（`#[cfg(test)]` / `#[test]` / `tests/`）。
2. 规则 patcher 优先，LLM patcher 作为函数级双入口中的兜底路径。
3. 修复与验收必须通过编译验证闭环，禁止“仅凭文本建议”直接落盘。
4. 当前默认每函数 3 轮预算（replay/online 统一上限）；网络请求失败计入该函数预算。
5. `Patcher` trait 本阶段不改签名；分析型 patcher 允许返回空动作（`Ok(vec![])`）。

---

## 二、当前开发状态

### 2.1 已完成（代码已落地）

13. E0433 R1 静态替换增强：支持 `unresolved_head::tail` 到 `crate::tail` 候选生成；新增依赖门禁（含 `package` rename）与轻量唯一可达性扫描（`src/**/*.rs`）；不确定场景安全降级为“无候选”。✅
15. 单函数多错误码聚合规则已落地：场景 A（span 不重叠）Append 聚合；场景 B（span 重叠/包含）主错误优先并产出 suppressed 证据。✅
21. 配置体系已落地：新增 `ruter.toml` + `.env.example`，支持 `CLI > ENV > TOML > 默认值` 优先级；新增 M3 配置参数（mode/api/model/timeout/max_rounds/context budget）。✅
22. LLM 执行器已升级：`replay/online` 双模式并存、每函数最多 3 轮预算、解析优先级 `rust fenced` > `plain fn` > `JSON fallback`、网络失败计入预算。✅
24. `4_llm_attempts.json` 已升级到 schema v2，包含 phase/prompt_excerpt 等 M3 证据字段；旧 replay 输入 schema 仍保持兼容读取。✅
29. E0433 R1 可达性最小共享索引已落地：新增 `src/patchers/e0433/reachability_index.rs`，`tail_is_uniquely_reachable` 改为共享索引查询，复用单次构建结果。✅
31. LLM 失败专项修复（Prompt/协议/上下文最小增强）已落地：路径脱敏修复为 `<CRATE_ROOT>/相对路径`；User Prompt 强裁剪（移除 `Round/Target/Output contract/Acceptance`）；在线主协议改为单个 Rust 代码块并保留 JSON 回退；新增同文件 `related_fn_defs`（诊断反引号标识符驱动）与 `4_llm_io_debug.json` 调试证据。✅
32. dead_code 清理已完成：移除 `FunctionRulePlanResult`、`FunctionOptimisticVerifyResult`、`VerifyTopKCandidatesResult`、`PartialUnionPlanResult` 中未使用字段，`cargo test --no-run` 无警告通过。✅
33. 结构优化（低风险）已完成：抽出 `src/llm/io_debug.rs` 承载 IO 调试记录模型与构造器，`src/llm/executor.rs` 仅保留执行编排逻辑；并清理 `verify_flow` 中历史无用 map。✅
34. Prompt retry memory loop 已落地：Round>=2 强制注入同函数上一轮失败摘要（修复摘要 + 失败分类 + 错误码计数），并升级 Prompt 协议到 v2，补齐单元/在线集成测试证据。✅
35. Local rule failure summary 注入已落地：移交 LLM 的函数在 Round=1 注入本地规则修复失败摘要（优先 `verify/4_function_verify_rounds.json`，缺失时回退 diagnostics 聚合），并补齐 prompt/执行器/在线集成测试。✅
36. Pre-flight Interceptors（LowValue + E0599）已落地：新增 `src/runtime/function/low_value.rs`、`src/patchers/e0599`、LLM 前置拦截执行链、`preflight_interceptor_digest` 上下文注入与 `phase=preflight_*` 运行证据。✅
37. M5-1 编译器建议提取公共层已落地：新增 `src/patchers/common/compiler_suggestion.rs`，E0433 已直接切换到 common API；旧 `e0433/suggestion_extractor.rs` 已移除，并补齐 common + e0433 回归测试。✅
38. M5-2 E0599 注册型 patcher 已落地：新增 `src/patchers/e0599/patcher.rs`（analysis-only，`analyze()->Ok(vec![])`），并在 `plan_top_k_stage` 与 `build_function_planning_stage` 统一默认 registry 中完成 `E0433+E0599` 接线。✅
39. M5-3 E0308 注册 + preflight 四层漏斗已落地：新增 `src/patchers/e0308`（analysis-only patcher + analyzer），默认 registry 已纳入 E0308；`FunctionDiagnostic` 已扩展 child note/help/suggestion 多源证据并注入 E0308 提取与分类。✅
40. M5-4 LowValue 错误码感知策略已落地：新增 `src/llm/preflight` 子模块，`PreflightDecision` 已统一接管拦截；low-value 全局一刀切注释已移除，改为 E0599/E0308 条件触发，且保留 verify 门禁与 E0308 高风险预算兜底。✅
41. M5-5 artifact 与指标对齐已落地：`preflight_interceptor_digest.notes` 已注入 `E0308_*` 键，`RunSummary` 已扩展 `preflight_skipped_llm_by_code`，`4_llm_attempts.json` 保持 schema v2 并补齐 `phase=preflight_*` 证据。✅
42. M6-1 LLM 执行器结构拆分已落地：`src/llm/executor.rs` 已降级为门面，主逻辑拆分到 `executor/round_runner.rs`、`executor/preflight_flow.rs`、`executor/candidate_resolution.rs`、`executor/workspace_ops.rs`、`executor/attempt_history.rs`，并抽离 `executor/tests.rs`。✅
43. M6-2 共享函数域类型已落地：新增 `src/core/function.rs` 承载 `TestFunction`/`FunctionDiagnostic`，`runtime::function::index` 改为复用并提供兼容 re-export，LLM 相关模块改为优先依赖 `core` 类型。✅
44. M6-3 LLM 验证端口抽象已落地：新增 `llm::executor::verify_port::LlmVerifyPort` 与 `runtime::llm_verify_port::RuntimeLlmVerifyPort`，`round_runner/preflight_flow` 不再直接调用 `runtime::stages::verify_partial_union_plan_with_tag`。✅
45. M6-4 E0433 patcher 文件瘦身已落地：`src/patchers/e0433/patcher.rs` 仅保留实现主链，测试迁移到 `src/patchers/e0433/tests.rs`。✅
46. M6-5 runtime 分层与治理已落地：`src/runtime/stages.rs` 下沉 `compile/analyze/top-k` 到 `stages/run_flow.rs`、`verify_flow` 拆出 `verify_flow/candidate_verify.rs`；新增架构守门脚本 `scripts/check_architecture_guard.sh`；`runtime/workflow.rs` 的 LLM 参数校验与 attempts bootstrap 已迁移到 `llm/runtime_entry.rs`。✅
47. M7-1 E0432 注册型 patcher：`ErrorCode::E0432` + 默认 registry 接线 + analysis-only 空动作语义 + 基线测试。(已完成)
48. M7-2 E0432 实修轨道 P1/R1 已落地：递归提取主/子诊断 `MachineApplicable` 建议并执行“非重叠全收、重叠取第一”直通；在 P1 未命中时启用 `head==package.name` 且仅 `use` 行的 `crate::` 精确重写。✅
49. M7-3 E0432 R2 已落地：仅在 P1/R1 均未命中且严格测试属性语境（`#[test]/#[tokio::test]/#[rstest]/#[cfg(test)]`）下，对单行 `use` 执行 `FixAction::Replace` 注释化替换（保留原 use 文本）。✅
50. M7-4 E0432 preflight 已落地（digest-only）：新增 `E0432_COUNT / E0432_SUMMARY / E0432_HINTS` 摘要注入，不改 `decide_preflight` 分支优先级与 SkipLlm 决策行为。✅
51. M8-1 E0560 设计修订与注册骨架已落地：`PatcherDesign.md`/`docs/E0560PatcherDesign.md` 已补“L2 ExprStruct 缺口 + digest-only 注入”约束；并完成 E0560 error_code + patcher 注册链路接线。✅
52. M8-2 E0560 analyzer + preflight digest-only + L2 最小补强已落地：实现 P1（Machine/Maybe）+ R1（编辑距离=1唯一候选）规则修复、`E0560_COUNT/UNKNOWN_FIELDS/AVAILABLE_FIELDS/HINTS` 注入、`ExprStruct.path` 符号采集增强。✅
53. M8-3 模块+回归组合验收已完成：`cargo test patchers::registry::tests::implemented_error_codes_includes_all_registered_patchers`、`cargo test e0560`、`cargo test llm::preflight`、`cargo test llm::context_builder`、`cargo test runtime::function::rule_plan`、`cargo test --no-run` 与代表性 E0560 fixture `cargo check --tests --message-format=json` 全部通过。✅
54. M8 测试覆盖补强已落地：新增 R1 “编辑距离>1 拒绝”测试、`preflight` E0560 键名契约测试（仅四键）、以及 verify 层 `E0063/E0451` unresolved 门禁测试。✅
55. M8-4 L2 `ExprStruct.path` 规则增强已落地：改为“末段类型名优先 + 首段回退”（`foo::Parser` 优先注入 `Parser`，并保留 `foo` 回退）；并同步两份设计文档说明。✅
56. 结构小优化（skill 扫描驱动）已落地：将 `src/patchers/e0560/analyzer.rs` 的单测拆分到 `src/patchers/e0560/analyzer/tests.rs`，该热点文件从扫描 Top-10 中移除（不再触发 `FILE_LARGE`）。✅
57. 交付前中小型架构优化已落地：`context_builder` 拆为 `budget/symbols/tests` 子模块并清零 `FILE_TOO_LARGE`；`patch_coordinator`、`e0432/analyzer`、`runtime::function::{index,dispatch,rule_plan}` 单测全部外移为 `tests.rs` 子模块；`round_runner` 增加 preflight/mapping 轻量 helper；新增并提交 `docs/arch_scan_baseline.json` 与刷新后的 `docs/arch_scan_latest.json`。✅
58. M9-1 LLM 候选归一化升级已落地：`patched_function_text` 从“整函数字符串替换”升级为 AST 级函数体合并（仅覆盖 `block`），并新增 `NormalizedCandidateEvidence.merge_strategy` 证据字段。✅
59. M9-2 LLM verify 失败证据增强已落地：`round_runner` 记录 `introduced_error_codes` 差分摘要，`attempt_history` 提炼 `introduced_error_by_code` 注入到 `previous_round_failure_digest`，Prompt 可显式展示“上一轮新增错误码”。✅
60. M9-3 no_std 环境感知已落地：preflight 新增 `CRATE_ENV_NO_STD=true/false` 键（扫描 `src/lib.rs` / `src/main.rs`），Prompt 增加 no_std 强约束提示，E0599 正则扩展支持 `no method named ... found for type ...`。✅
61. M9-4 E0308 数值提示增强已落地：preflight 新增 `E0308_EXPECTED_ARRAY_LEN`、`E0308_FOUND_ARRAY_LEN`、`E0308_LEN_DELTA`（可解析时）并补齐单测。✅
63. M9-6 闭环验收门禁去噪已落地：`round_runner` 对 `non-target regression` 增加“无新增错误码且总错误不升/非目标计数不升”的等价波动放行逻辑，并以稳定 identity（去行号后缀）比较非目标 unresolved 集，显著降低误拒绝。✅
64. M9-7 no_std 幻觉硬守卫与提示增强已落地：当命中 `no_std + primitive E0599(to_string)` 场景，候选中若再次调用 `.to_string()` 将在 normalize 阶段直接拒绝；preflight 新增 `E0599_NO_STD_PRIMITIVE_TOSTRING` 键，prompt 强化禁止项；同配置 `phase_1A pilot` 复测中 `LLM_VERIFY_FAILED` 与 `non-target regression` 明显下降。✅
65. M9-8 验收门禁二次收敛已落地：`non-target` 波动放行从“非目标计数不升”进一步收敛为“无新增错误码且总错误不升”即可通过，减少 `introduced_error_codes=none` 的误阻塞；对应单测新增“计数上升但错误总量不升可放行”场景。✅
67. M9-10 Online 输出预算控制已落地：新增 `cap_tokens = floor(input_tokens_est * ratio)`（默认 `ratio=2.0`），在线请求体优先发送 `max_tokens=cap_tokens`；配置层新增 `llm.output_token_ratio` / `RUTER_LLM_OUTPUT_TOKEN_RATIO` / `--llm-output-token-ratio`，并补齐 client/config/cli 测试。✅
68. M9-11 Prompt 目标函数源码漂移修复已落地：`current_source_for_function` 改为返回“baseline 源码 + 重映射后的函数范围”，`round_runner` 在构建 context/prompt 时改用重映射范围，避免“旧 byte range + 新源码”导致 `Target function source` 截断；并新增 `candidate_resolution` 单测覆盖“有位移/无位移”两类场景。✅
69. M9-12 规则错误码漂移识别与提示注入已落地：`round_runner` 新增基于 `3_function_dispatch_report.json` / `3_function_rule_candidates.json` / `verify/4_function_verify_rounds.json` 的 `RuleDriftSnapshot` 回放，识别“原始规则码消失 + 新错误码出现”漂移并注入 `rule_error_drift_digest`；`context_builder` 新增 `RuleErrorDriftDigestV1/RuleErrorDriftPairV1` 及 `rule_drift` 预算裁剪链；`prompt_builder` 新增 `Rule patch drift hints` 段（原始函数源码、失败规则方案、漂移对与避免重复失败提示）；补齐 `round_runner/context_builder/prompt_builder` 单测与在线集成测试 `online_prompt_injects_rule_error_drift_digest`。✅
70. E0433 中等重构（`crate::run_id::...` 失败场景）已落地：补齐 `could not find ... in the crate root` 句型提取；`R2` 新增“crate 根模块可见性”门禁（支持非 `pub mod`）；`FunctionIndex` 增加“测试模块内函数外诊断”回落映射以消除 `__UNMAPPED_ERRORS__` 漏路由；`FixGenerator` 增加前导 `crate::` 去重以修复 `crate::crate::...` 双前缀；并补齐 `e0433 / runtime::function::{index,dispatch}` 回归测试。✅
71. E0433 模块上下文增强（`src/v0.rs` 内 `v0::Type` 误写场景）已落地：当 unresolved `head` 本身是 crate 根模块时，R1 改为保留 `head` 生成 `crate::head::...`（不再错误降格为 `crate::tail...`）；新增 `v0::Parser` 与 `<v0::ParseError>` 双回归单测；本地复测显示 `batch_0025` 的 E0433 已清零，`batch_0026` 的 E0433 亦清零并暴露后继 `E0277`（非路径修复引入）。✅
72. E0433 `crate::<extern_or_dep>::...` 泛化修复已落地：新增 `missing crate` 句型提取与 R3 候选（测试语境下对 `crate::head::...` 误前缀执行“去 `crate::`”路径重写），并引入 `package.name` 冲突规避 + 依赖声明感知（支持 `core/std/alloc/...` 与普通依赖如 `serde`）；`FixGenerator` 新增“no-op 但前置 `crate::`”的回退扩 span 替换；`ryu batch_0002` 本地复测 `E0433:1 -> 0`。✅
73. E0308 同模块边界增强已落地：`runtime::function::index` 新增 `ScopeRange` + “同测试模块函数集合”查询；LLM `candidate_resolution` legacy action 校验由“函数范围”放宽为“同测试模块范围”；`preflight comment-out` 优先注释同测试模块并回退到函数范围；`round_runner/preflight_flow` 的 non-target 回归判定已改为“排除同模块 identity”；`E0308Patcher` 从占位升级为 Nominal Drift 确定性修复（同名尾类型 + 路径漂移，歧义 fail-closed）；`batch_0008/0009` 复测 `final_status` 均为 `resolved_by_full`。✅
74. E0599 命名空间错位增强已落地：`analyzer` 新增 `MisplacedFreeFunction` 分类与“同文件优先 + crate 回退”自由函数补扫（含“编辑距离=1 且唯一候选”保守兜底）；`E0599Analysis` 新增 `related_free_function_signatures/recommended_call_forms`；preflight 新增 `E0599_RELATED_FREE_FN_SIGNATURES` 与 `E0599_NAMESPACE_HINT` 并在 `decide_preflight` 下发 `ContinueToLlmWithHints`；`round_runner` 会将该自由函数签名并入 prompt 的 `related free function defs`；已补齐 e0599/preflight/decision/round_runner 相关单测。✅

### 2.2 待开发

（暂无）

---

## 三、里程碑计划

### M8：E0560Patcher 三阶段实现

1. M8-1（已完成）
   1. 设计文档修订先行（5.5/6.1/6.2 对齐现实现状）。
   2. 注册型骨架接线（`ErrorCode` + patcher + default registry）。
2. M8-2（已完成）
   1. 规则层：P1（Machine/Maybe）+ R1（编辑距离=1 唯一候选）最小字段键替换。
   2. 可观测层：digest-only 注入 4 键，不修改 preflight 决策枚举。
   3. 上下文层：L2 补采 `ExprStruct.path`，仅同文件范围。
3. M8-3（已完成）
   1. 执行模块+回归组合验收命令集。
   2. 验收标准：E0560 下降、E0063/E0451 不新增、非目标 unresolved 不回归、prompt 含 E0560 4 键。
4. M8-4（已完成）
   1. L2 `ExprStruct.path` 改为“末段类型名优先 + 首段回退”。
   2. 基于架构扫描执行低风险拆分：`e0560/analyzer` 测试外移，降低单文件复杂度。

### M9：LLM 闭环可靠性增强

1. M9-1（已完成）
   1. `patched_function_text` 采用 AST 级函数体合并，签名/属性由原函数强制保留。
   2. 候选证据新增 `merge_strategy`，便于回放与归因。
2. M9-2（已完成）
   1. verify 失败详情追加 `introduced_error_codes` 差分。
   2. 历史摘要新增 `introduced_error_by_code`，Round>=2 Prompt 显式注入。
3. M9-3（已完成）
   1. preflight 注入 `CRATE_ENV_NO_STD` 环境键。
   2. Prompt 增加 no_std 强约束提示，抑制 `.to_string()` 幻觉路径。
4. M9-4（已完成）
   1. E0308 digest 新增数组长度提示与长度差值提示（可解析时）。
   2. 暂未引入常量表达式求值器，保留为后续增强项。


---

## 四、测试与验收标准

### 4.1 功能验收

1. 函数级分发决策正确（规则优先 + LLM 兜底）。
2. LLM 输出在线主协议为单个 Rust 函数代码块（兼容 JSON 回退），并统一映射到 `FixAction::Replace`。
3. 验收门槛为“函数级错误清零 + 全量 `cargo check --tests` 通过”。
4. M2 副作用门禁：不得引入非目标函数 unresolved 回归。

### 4.2 稳定性验收

1. LLM 重试预算（3 轮）生效且可预测。
2. 失败路径不破坏已通过函数的补丁结果。
3. 不引入对生产代码的越权修改。

### 4.3 可观测性验收

1. 必须可追踪每函数分发决策、每轮候选、失败分类与验证结果。
2. 保持与现有 `6_summary.json`、`verify/4_report.json`、`4_llm_handoff.json` 兼容。

---

## 五、风险与缓解（更新）

1. 风险：模型输出不稳定或格式漂移。
   - 缓解：严格 schema 校验；非法输出直接拒绝。
2. 风险：LLM 成本与时延上升。
   - 缓解：每函数 3 轮预算、失败早停、按函数并行度可配置。
3. 风险：上下文不足导致错误修复。
   - 缓解：保持“目标函数最小编辑”，并逐步扩展只读上下文窗口。
4. 风险：函数映射偏差（宏/复杂语法）。
   - 缓解：`syn` + 行列/字节双回退；落入 `__UNMAPPED_ERRORS__`。
5. 风险：动作冲突或越界编辑。
   - 缓解：复用冲突检测 + span 门禁，验证失败不落盘。
6. 风险：Prompt 膨胀导致上下文预算紧张。
   - 缓解：历史摘要结构化短文本、固定截断、超预算仅裁剪 `candidate_failures` 尾部。
7. 风险：失败摘要失真导致模型过度规避。
   - 缓解：优先保留验证相位失败、稳定排序、仅注入同函数 round-1 证据。
8. 风险：E0308 分类误判导致过度注释。
   - 缓解：采用保守注释策略，仅在 Setup Hell/低价值名义幻觉场景允许注释。
9. 风险：公共 suggestion 抽取重构引入行为漂移。
   - 缓解：先抽象后迁移，保留 E0433 原逻辑旁路以便快速回滚。
10. 风险：注册型分析 patcher “空动作”被误解为失败。
   - 缓解：在 artifacts 与文档中明确“空动作是预期语义”，并用 preflight 证据补全可观测性。
11. 风险：函数外 E0432（文件头/模块头 import）被忽略导致覆盖盲区。
   - 缓解：明确函数级边界，并在设计文档中声明“函数外 E0432 不进入本地规则修复”。
12. 风险：E0560 场景缺失 struct 字段上下文，导致 LLM 重复无效尝试。
   - 缓解：`context_builder` 补采 `ExprStruct.path`，并通过 digest-only 注入补齐字段集证据。

---

## 六、真实 Fixture + 在线 LLM 测试流程（humantime-e0433-gemini-fixture）

1. 详细测试手册已抽取到根目录文档：`TestHelp.md`。
2. 建议执行顺序：
   1. 先跑基线失败复现；
   2. 再跑 online dry-run；
   3. 验证 artifacts 后再决定 apply；
   4. 最后跑 fixture 回归与全量回归。
3. 一键本地回归入口（不访问真实 online API）：`bash scripts/run_regression_m3.sh`。

## 七、备注

1. `CandidateFix.bare_identifier` 仍可进一步纳入跨错误码评分与语义匹配。
2. 现阶段不引入 `ManifestEdit`；待 LLM patcher 主链稳定后再评估动作模型升级。
3. E0599/E0308 的“注册 patcher + preflight analyzer”组合是 M5 的统一模板。
4. E0432 当前强制函数级处理；函数外 import 诊断不进入本地规则修复。
5. E0560 本轮采用“注册 patcher + 窄口径规则修复 + digest-only preflight + L2 最小补强”模板，不扩展到 builder/宏重写。
