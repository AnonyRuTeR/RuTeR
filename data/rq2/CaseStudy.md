# Phase 1A 典型案例详解

本文档整理了 3 个来自 Phase 1A 主评估语料的典型案例，分别覆盖：

1. 规则驱动成功修复：E0433
2. 分析提示 + LLM handoff 成功修复：E0308
3. 多错误耦合下的未收敛失败：E0599 主导场景

所有结论都只基于本地 artifact，不引用 `artifact_stats_artifacts*/attempt_results.csv` 这类旧快照统计作为个案证据。

## 证据来源说明

### 案例 1：E0433 成功修复
- `evaluation/phase_1A/artifacts/humantime_claude-3-5-haiku-20241022_20251109_133407/batch_0001/batch_result.json`
- `evaluation/phase_1A/artifacts/humantime_claude-3-5-haiku-20241022_20251109_133407/batch_0001/repro/cargo_check.json`
- `evaluation/phase_1A/artifacts/humantime_claude-3-5-haiku-20241022_20251109_133407/batch_0001/repro/step_compile/1_compile_diagnostics.json`
- `evaluation/phase_1A/artifacts/humantime_claude-3-5-haiku-20241022_20251109_133407/batch_0001/rule/3_function_rule_candidates.json`
- `evaluation/phase_1A/artifacts/humantime_claude-3-5-haiku-20241022_20251109_133407/batch_0001/rule/3_plan.json`
- `evaluation/phase_1A/artifacts/humantime_claude-3-5-haiku-20241022_20251109_133407/batch_0001/rule/5_changes.diff`

### 案例 2：E0308 成功修复
- `evaluation/phase_1A/artifacts/itoa_gemini-2.5-flash-nothinking_20251127_010109/batch_0001/batch_result.json`
- `evaluation/phase_1A/artifacts/itoa_gemini-2.5-flash-nothinking_20251127_010109/batch_0001/batch_llm.json`
- `evaluation/phase_1A/artifacts/itoa_gemini-2.5-flash-nothinking_20251127_010109/batch_0001/rule/stage_result.json`
- `evaluation/phase_1A/artifacts/itoa_gemini-2.5-flash-nothinking_20251127_010109/batch_0001/rule/verify/4_function_verify_rounds.json`

注：该批次的 `repro/step_compile/1_compile_diagnostics.json` 未在当前保留集内落盘保留，因此“代表性编译反馈”来自 `batch_llm.json` 中逐函数 handoff prompt，与 `batch_result.json` 的总量统计交叉核对。

### 案例 3：E0599 主导的失败案例
- `evaluation/phase_1A/artifacts/semver_gemini-2.5-flash-nothinking_20251127_013324/batch_0009/batch_result.json`
- `evaluation/phase_1A/artifacts/semver_gemini-2.5-flash-nothinking_20251127_013324/batch_0009/batch_llm.json`
- `evaluation/phase_1A/artifacts/semver_gemini-2.5-flash-nothinking_20251127_013324/batch_0009/rule/stage_result.json`
- `evaluation/phase_1A/artifacts/semver_gemini-2.5-flash-nothinking_20251127_013324/batch_0009/rule/verify/4_function_verify_rounds.json`

注：和案例 2 一样，当前保留集未保存独立的 `repro/step_compile/1_compile_diagnostics.json`，因此代表性编译反馈取自 `batch_llm.json` 中的原始 handoff prompt，并与 `batch_result.json` 的批次级错误统计交叉核对。

---

## 案例一：E0433 成功修复

### 1. 基本信息
- `batch_id`: `humantime_claude-3-5-haiku-20241022_20251109_133407::batch_0001`
- crate: `humantime`
- model: `claude-3-5-haiku-20241022`
- 目标位置：`src/wrapper.rs` 中自动注入测试模块 `tests_rug_23`
- 最终状态：`resolved_by_rule`

### 2. 原始错误代码
- 主错误码：`E0433`
- 冻结口径：`frozen_e_codes = ["E0433"]`
- 复现编译统计：`error_total = 2`，`error_by_code = {"E0433": 2}`

### 3. 编译反馈
`cargo check --tests --message-format=json --locked` 的复现结果如下：
- `exit_code = 101`
- `duration_sec = 5.202`
- `check_passed = false`
- `error_total = 2`
- `error_by_code = {"E0433": 2}`

两条主诊断都来自同一个测试函数 `src/wrapper.rs::tests_rug_23::test_rug`：

1. 第 120 行：
   - message: `failed to resolve: use of unresolved module or unlinked crate 'wrapper'`
   - label: `use of unresolved module or unlinked crate 'wrapper'`
   - rustc help: 可以声明 `mod wrapper;`，也给出了 `use crate::Duration;` 等导入方向
2. 第 122 行：
   - message: `failed to resolve: use of unresolved module or unlinked crate 'wrapper'`
   - label: `use of unresolved module or unlinked crate 'wrapper'`
   - rustc help: 给出了 `use crate::wrapper;` 的导入方向

对应的原始测试代码片段是：

```rust
let p0 = wrapper::Duration::from_str("12h 5min 2ns").unwrap();
<wrapper::Duration>::as_ref(&p0);
```

### 4. 修复分析
这是一个非常典型的“测试模块内路径限定错误”。问题不在于业务逻辑，也不在于类型系统，而在于自动生成的测试把 `wrapper::Duration` 当成了当前作用域可直接解析的模块路径；但测试实际位于 `src/wrapper.rs` 内部的 `#[cfg(test)]` 模块，正确路径应显式回到 crate 根作用域。

这里有两个关键信号让规则修复变得可靠：
- 编译器已经把错误范围精确定位到两个 token 级别的路径片段
- rustc 的帮助信息和本地候选分析都指向同一类解法：将未限定路径改为 crate 内可解析路径

### 5. RuTeR 的修复动作
`E0433Patcher` 为该函数生成了 3 组候选，其中 rank 1 候选得分最高（`score = 292`），并被 verify 选中。选中的动作一共 2 条：

1. 将第 120 行中的
   - `wrapper::Duration::from_str`
   - 替换为 `crate::wrapper::Duration::from_str`
2. 将第 122 行中的
   - `wrapper::Duration`
   - 替换为 `crate::wrapper::Duration`

`3_plan.json` 和 `5_changes.diff` 中记录的最终补丁如下：

```diff
- let p0 = wrapper::Duration::from_str("12h 5min 2ns").unwrap();
+ let p0 = crate::wrapper::Duration::from_str("12h 5min 2ns").unwrap();

- <wrapper::Duration>::as_ref(&p0);
+ <crate::wrapper::Duration>::as_ref(&p0);
```

### 6. 修复是否成功
成功。

rule 阶段结果：
- `exit_code = 0`
- `duration_sec = 4.633`
- `resolved_test_function_count = 1`
- `unresolved_test_function_count = 0`
- `remaining_error_total = 0`
- `patch_candidate_file_count = 1`
- `llm_attempt_count = 0`
- `verify_round_count = 1`
- `verify_report.check_passed = true`

`4_function_verify_rounds.json` 也显示：
- 第 1 轮 verify 即选择了 rank 1 候选
- `check_error_total = 0`
- 目标函数 `src/wrapper.rs::tests_rug_23::test_rug:118:123` 成功 resolved

### 7. 修复原理分析
这个案例体现了 RuTeR 在浅层结构化错误上的优势：
- 错误是局部、确定性的路径解析失败
- 修补动作只涉及同一测试函数内的 token 替换，不需要引入新语句或推断业务语义
- verify gate 可以立即判定补丁是否将目标错误清零，且不会污染其他函数

因此，Rule-first 在这类错误上的本质不是“猜修复”，而是利用编译器定位信息执行一个高置信度、低搜索成本的局部重写。

---

## 案例二：E0308 成功修复

### 1. 基本信息
- `batch_id`: `itoa_gemini-2.5-flash-nothinking_20251127_010109::batch_0001`
- crate: `itoa`
- model: `gemini-2.5-flash-nothinking`
- 目标位置：`src/udiv128.rs` 中自动注入测试模块 `tests_rug_17`
- 最终状态：`resolved_by_full`

### 2. 原始错误代码
- 主错误码：`E0308`
- 冻结口径：`frozen_e_codes = ["E0308"]`
- 复现编译统计：`error_total = 8`，`error_by_code = {"E0308": 8}`

这 8 个 E0308 落在同一测试模块内的 8 个测试函数上：
- `test_i128_max`
- `test_i128_min`
- `test_large_number_with_middle_zeros`
- `test_large_positive_number`
- `test_negative_single_digit`
- `test_positive_single_digit`
- `test_rug`
- `test_zero`

### 3. 编译反馈
代表性编译反馈来自 `batch_llm.json` 中的 handoff prompt。以 `test_i128_max` 为例：

- 位置：`src/udiv128.rs:107:35-107:42`
- message: `mismatched types`
- label: `expected an array with a size of 40, found one with a size of 42`

原始测试函数的核心问题是：

```rust
const U64_MAX_LEN: usize = 20;
const I128_MAX_LEN: usize = U64_MAX_LEN * 2 + 1 + 1;
let mut p1: [MaybeUninit<u8>; I128_MAX_LEN] = unsafe { MaybeUninit::uninit().assume_init() };
let s = <i128>::write(p0, &mut p1);
```

而 preflight 对每个函数都给出了同一组静态约束：
- `E0308_EXPECTED = &mut [MaybeUninit<u8>; 40]`
- `E0308_FOUND = &mut [MaybeUninit<u8>; 42]`
- `E0308_EXPECTED_ARRAY_LEN = 40`
- `E0308_FOUND_ARRAY_LEN = 42`
- `E0308_LEN_DELTA = 2`

也就是说，问题并不是调用目标不明，而是测试缓冲区长度常量系统性偏大。

### 4. 修复分析
rule 阶段完全没有产出可接纳的本地补丁：
- `resolved_test_function_count = 0`
- `unresolved_test_function_count = 8`
- `remaining_error_by_code = {"E0308": 8}`
- `patch_candidate_file_count = 0`
- `llm_handoff_count = 8`

这符合 RuTeR 的职责分工：当前 rule patcher 擅长处理路径、导入和少量结构化替换，不负责直接推导数组长度常量。于是系统把 8 个 unresolved 函数全部移交给 full 阶段，并把 `expected/found` 这组长度约束写入 handoff prompt。

### 5. RuTeR 的修复动作
full 阶段的修复动作可以概括为：
- 保留原有测试语义和断言
- 不修改生产代码
- 仅调整测试缓冲区长度常量，使其从 42 收敛到 40

#### 5.1 一次命中的代表补丁
`test_i128_max` 在 round 1 就给出了被 verify 接纳的补丁：

```rust
#[test]
fn test_i128_max() {
    let p0: i128 = i128::MAX;
    const U64_MAX_LEN: usize = 20;
    const I128_MAX_LEN: usize = U64_MAX_LEN * 2;
    let mut p1: [MaybeUninit<u8>; I128_MAX_LEN] = unsafe { MaybeUninit::uninit().assume_init() };

    let s = <i128>::write(p0, &mut p1);
    assert_eq!(s, "170141183460469231731687303715884105727");
}
```

#### 5.2 先失败、后收敛的代表补丁
`test_large_positive_number` 在 round 1 先尝试把长度改成 41：

```rust
const I128_MAX_LEN: usize = U64_MAX_LEN * 2 + 1;
```

该候选被 verify 拒绝，失败记录为：
- `failure_kind = LLM_VERIFY_FAILED`
- `unresolved_error_by_code = E0308=5`
- `introduced_error_by_code = none`

在 round 2 中，模型根据上一轮失败记录把长度进一步收缩为 40，随后通过 verify：

```rust
const I128_MAX_LEN: usize = U64_MAX_LEN * 2;
```

`test_positive_single_digit` 更极端：
- round 1：41，失败
- round 2：仍然 41，失败
- round 3：改为 40，成功

#### 5.3 批次级结果
从 `attempt_index` 统计可以看出 8 个函数的收敛分布：
- 6 个函数在 round 1 直接成功
- 1 个函数在 round 2 成功
- 1 个函数在 round 3 成功

full 阶段总结果：
- `duration_sec = 22.629`
- `resolved_test_function_count = 8`
- `unresolved_test_function_count = 0`
- `remaining_error_total = 0`
- `patch_candidate_file_count = 1`
- `llm_attempt_count = 27`
- `verify_report.check_passed = true`

### 6. 修复是否成功
成功。

最终批次状态：
- `final_status = resolved_by_full`
- `strict_success = true`
- `normalized_success = true`
- `frozen_status = frozen_cleared`

### 7. 修复原理分析
这个案例说明 RuTeR 在 E0308 场景下的关键能力，不是“让 LLM 盲猜类型”，而是先用静态分析把问题压缩成一个受约束的常量修正任务：

1. preflight 已经指出“期望 40，实际 42”
2. handoff prompt 保留了函数源码、错误位置和 no-std 约束
3. verify gate 连续拒绝 `41` 这类看似接近但仍不满足签名的补丁
4. 最终只有长度精确收敛到 `40` 的候选才能被接纳

因此，这里的成功并不是 LLM 的自由发挥，而是“静态约束缩小搜索空间 + 验证门禁剔除近似错误”的结果。

---

## 案例三：E0599 主导的多错误耦合失败

### 1. 基本信息
- `batch_id`: `semver_gemini-2.5-flash-nothinking_20251127_013324::batch_0009`
- crate: `semver`
- model: `gemini-2.5-flash-nothinking`
- 目标位置：`src/display.rs` 中自动注入测试模块 `tests_rug_6`
- 最终状态：`unresolved_after_full`

### 2. 原始错误代码
- 冻结口径：`frozen_e_codes = ["E0599"]`
- 但复现阶段的实际错误分布已经扩展为：
  - `E0425 = 11`
  - `E0599 = 3`
  - `E0603 = 1`
- 总错误数：`15`
- 涉及 unresolved 测试函数数：`12`

这说明该批次虽然在冻结语料标签上以 E0599 为主，但在真实重放时已经演化成一个多错误耦合场景，而不是纯单码失败。

### 3. 编译反馈
#### 3.1 代表反馈一：`test_exact_match_with_prerelease`
该测试的原始代码为：

```rust
#[test]
fn test_exact_match_with_prerelease() {
    let cmp = Comparator::parse("=1.0.0-alpha.1").unwrap();
    let ver = Version::new_with_prerelease(1, 0, 0, "alpha.1");
    assert!(matches_exact(&cmp, &ver));
}
```

对应编译反馈：
- `E0425`: `cannot find function 'matches_exact' in this scope`
- `E0599`: `no function or associated item named 'new_with_prerelease' found for struct 'Version' in the current scope`

#### 3.2 代表反馈二：`test_wildcard_minor_match`
该测试的原始断言为：

```rust
assert!(matches_exact(&test_cmp, &ver));
```

对应编译反馈：
- `E0425`: `cannot find function 'matches_exact' in this scope`

也就是说，这一批次至少同时存在三类问题：
- 未解析的自由函数调用（E0425）
- 不存在的关联函数或方法调用（E0599）
- 可见性错误（E0603）

### 4. 修复分析
rule 阶段没有产生任何可用修复：
- `resolved_test_function_count = 0`
- `unresolved_test_function_count = 12`
- `remaining_error_total = 15`
- `remaining_error_by_code = {"E0425": 11, "E0599": 3, "E0603": 1}`
- `patch_candidate_file_count = 0`
- `llm_handoff_count = 12`

进入 full 阶段后，系统确实修掉了一部分问题：
- `resolved_test_function_count = 8`
- `unresolved_test_function_count = 4`
- `remaining_error_total = 7`
- `remaining_error_by_code = {"E0425": 4, "E0599": 3}`
- `llm_attempt_count = 39`

也就是说，RuTeR 并非完全无效，而是发生了“局部有效但整体未收敛”的情况：它救回了 8 个测试函数，但最后 4 个函数仍拖住了整个批次，导致 verify gate 不接纳最终补丁集。

### 5. RuTeR 的修复动作
#### 5.1 能成功的局部修复
有些函数可以通过简单地改写断言 API 取得成功，例如 `test_wildcard_patch_match`：

原始断言：

```rust
assert!(matches_exact(&test_cmp, &ver));
```

LLM round 1 给出的补丁：

```rust
assert!(cmp.matches(&ver));
```

该函数对应的 verify 记录是成功接纳的，因此它属于“局部 API 替换后即可收敛”的子问题。

#### 5.2 未能收敛的错误修补轨迹
真正拖垮批次的是 4 个函数：
- `test_exact_match_with_prerelease`
- `test_prerelease_mismatch`
- `test_prerelease_missing_in_comparator`
- `test_wildcard_minor_match`

它们的共同点是：
- 每个函数都经历了 3 轮 verify 失败
- 之后触发 `LLM_BUDGET_EXHAUSTED`
- 没有任何一个候选在 full 阶段被接纳

最典型的是 `test_wildcard_minor_match`。模型连续三轮都把原本不存在的自由函数 `matches_exact(...)` 改写成不存在的方法调用：

round 1：
```rust
assert!(cmp.matches_exact(&test_cmp, &ver));
```

round 2：
```rust
assert!(cmp.matches_exact(&ver));
```

round 3：
```rust
assert!(cmp.matches_exact(&ver));
```

verify 对 round 1 和 round 2 的失败记录都明确指出：
- `failure_kind = LLM_VERIFY_FAILED`
- `introduced_error_codes = E0599=1`
- `unresolved_error_by_code = E0425=4, E0599=4`

也就是说，模型虽然试图把“找不到自由函数”改成“调用现有对象方法”，但实际上引入了新的“方法不存在”错误，导致 E0425 没有真正消失，E0599 反而被放大。

`test_exact_match_with_prerelease` 的轨迹也类似。模型连续三轮尝试把：
- `Version::new_with_prerelease(...)`
- `matches_exact(&cmp, &ver)`

改写成：
- `Version::new(...).pre(...)`
- `Version::new(...).with_prerelease(...)`
- `cmp.matches(&ver)`

这些候选说明模型已经识别出“应该找替代构造器/替代匹配接口”，但它对当前 crate 中真实可用 API 的把握并不稳定，导致三个 round 全部被 verify 拒绝，最后预算耗尽。

### 6. 修复是否成功
失败。

最终批次状态：
- `final_status = unresolved_after_full`
- `strict_success = false`
- `normalized_success = false`
- `final_remaining_error_total = 7`
- `final_unresolved_function_ids` 共 4 个

full 阶段虽然相比 rule 阶段明显缩小了错误面，但没有达到 verify gate 所要求的“目标错误清零且无残留 unresolved”的接纳标准。

### 7. 修复原理分析
这个案例反映的不是“单个 E0599 太难”，而是多错误耦合下的系统性收敛困难：

1. 冻结标签是 E0599，但实际复现同时包含 E0425、E0599、E0603，说明问题已经跨越了函数调用解析、关联项解析和可见性边界
2. 一部分函数可以靠局部 API 改写快速成功，说明 full 阶段不是完全失效
3. 失败的 4 个函数都呈现出相同模式：模型知道应当“换 API”，但无法稳定对齐当前 crate 中真实存在、且可在测试模块访问的符号集合
4. 一旦将 `matches_exact(...)` 误改为 `cmp.matches_exact(...)` 这类错误方法调用，系统就会把 E0425 转化为新的 E0599，形成局部修补反复震荡
5. 在当前预算配置下，verify gate 只能不断拒绝这些候选，而不会进行跨函数的全局重排或 staged rollback search

因此，这个案例的失败本质是：局部修补可以缩小错误面，但在“未解析符号 + 不存在的关联项 + 可见性约束”耦合时，RuTeR 现阶段缺少更强的全局协调机制，最终无法把剩余冲突整体收敛到零。

---

## 总结对比

| 案例 | 主错误码 | 原始规模 | 主要修复方式 | 最终结果 | 关键结论 |
|---|---|---:|---|---|---|
| 案例一 | E0433 | 2 个错误 / 1 个测试函数 | 纯规则 patcher | 成功 | 路径限定类错误适合确定性局部替换 |
| 案例二 | E0308 | 8 个错误 / 8 个测试函数 | preflight 约束 + LLM handoff | 成功 | 静态约束能把类型修复压缩成窄搜索空间 |
| 案例三 | E0599 主导，多码耦合 | 15 个错误 / 12 个测试函数 | full 阶段局部修补 | 失败 | 多错误耦合下局部补丁可能反复把一种错误转化成另一种错误 |

这三个案例共同说明：
- RuTeR 在浅层结构化错误上最强，尤其是 E0433 这类路径/名称解析问题
- 当静态分析能把 E0308 这类问题压缩为明确的 `expected/found` 差异时，LLM 阶段也能稳定工作
- 真正困难的是多错误耦合场景：即使系统能局部修掉一部分错误，也未必能在有限预算内完成全局收敛
