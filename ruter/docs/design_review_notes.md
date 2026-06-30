# RuTeR 设计审查备忘

> **日期**: 2026-03-08
>
> 本文档记录在核实代码实现与重写设计文档过程中发现的潜在设计问题和改进建议。
> 这些不影响 PatcherDesign.md 的忠实记录——设计文档严格反映当前实现，本文件独立存放建议。

---

## 1. FixAction 枚举存在未使用变体

**现状**：`FixAction` 定义了 `Insert`、`Replace`、`Delete` 三个变体，但 `CodeTransformer` 仅实现了 `Replace`。遇到 `Insert`/`Delete` 会返回 `UnsupportedFixAction` 错误。

**风险**：API 表面误导调用方认为三种操作均可用。

**建议**：
- 短期：在 `FixAction` 的 `Insert`/`Delete` 上添加 `#[deprecated]` 或文档注释说明未实现
- 长期：如果确认不需要，简化为仅 `Replace`

---

## 2. E0599/E0308 Analysis Patcher 的 analyze() 总是返回空 Vec

**现状**：E0599Patcher 和 E0308Patcher 的 `analyze()` 方法实现为 `Ok(Vec::new())`，其分析逻辑在 Preflight 阶段独立调用，不经过 Patcher trait 接口。

**问题**：`Patcher` trait 的 `analyze()` 签名暗示它应返回修复动作。返回空 Vec 虽然合法，但语义模糊——调用方无法区分"无法修复"和"分析型 Patcher 设计如此"。

**建议**：
- 考虑在 `Patcher` trait 中明确区分 `ActionPatcher` 和 `AnalysisPatcher` 子分类（例如通过一个 `fn patcher_kind(&self) -> PatcherKind` 方法）
- 或在 `description()` 返回值 / 文档中明确标注

---

## 3. SymbolReachabilityIndex 使用全局 OnceLock + Mutex 缓存

**现状**：`e0433::reachability_index` 使用 `OnceLock<Mutex<HashMap<PathBuf, ...>>>` 作为全局缓存。

**问题**：
- 在并发场景下 Mutex 锁争用
- `OnceLock` 意味着进程内第一次扫描的结果将永久缓存，无法感知文件变更
- 全局可变状态不利于测试隔离

**建议**：将缓存生命周期绑定到 `PatchCoordinator` 或 `RuntimeContext`，而非全局静态。

---

## 4. 上下文字符计数基于 JSON 序列化

**现状**：`context_char_count()` 通过 `serde_json::to_string()` 序列化整个 bundle 后计算 `.chars().count()`。

**问题**：
- 序列化开销在多轮裁剪循环中被反复触发
- JSON 序列化长度包含键名、引号、转义符等，与实际 Prompt 文本长度有差异

**建议**：
- 可选：维护增量字符计数器，避免每次全量序列化
- 或：预计算各 section 的近似字符数，裁剪时直接减

---

## 5. E0432 R2 的注释化策略较激进

**现状**：当 P1、R1 轨道未命中时，R2 直接将 `use` 语句注释化。虽然有级联阻断验证（检测 E0425/E0433），但注释化本身修改了代码结构。

**问题**：注释化的 `use` 语句可能在后续 LLM 轮次中造成混淆——LLM 看到被注释的行，可能不理解其已被工具处理。

**建议**：在 LLM 上下文中标注哪些 `use` 已被 R2 处理，或在 Preflight digest 中注入相关信息。

---

## 6. 裁剪顺序中 preflight_interceptor_digest 在较后位置

**现状**：上下文预算裁剪顺序中，`preflight_interceptor_digest` 排在 `diagnostics.primary_items` 和 `local_rule_failure_digest` 之后。

**问题**：Preflight digest 包含错误码分类和修复策略提示（如 E0599 的可用方法签名），信息密度较高。当预算紧张时，它可能比部分 primary_items 更有价值。

**建议**：考虑将 `preflight_interceptor_digest` 的裁剪优先级提高（更晚裁剪），或按信息价值做自适应裁剪。

---

## 7. Replay 模式与 Online 模式的候选格式不完全统一

**现状**：`OnlineLlmClient` 支持 4 种 JSON 形状（Shape A/B/C/D）+ fenced code block + plain fn item。Replay 文件使用结构化的 `LlmReplayFile` 格式。

**问题**：两种模式的候选解析路径不完全相同，增加了维护复杂度和边际 Bug 风险。

**建议**：长期统一 Online 和 Replay 的候选解析管线，Online 响应先转换为内部统一格式再进入候选解析流程。
