# Phase 1A+ RQ Draft

- Generated at (UTC): 2026-04-15T14:24:56+00:00
- Source snapshot: evaluation/phase_1A/logs/run_summary_refreshed_all_20260316_031806.json
- Attempt total: 1858
- Strict success: 866/1858 = 0.4661
- Normalized success: 906/1858 = 0.4876
- Replay eligible: 738/1858 = 0.3972
- Coverage metric scope: lines, functions, regions
- Branch coverage available: False
- Branch coverage note: Current coverage replay exports only provide usable lines/functions/regions totals; branch coverage is excluded from the Phase1A+ main summary.

## Patch Recovery
- full_patch_recovery_failed: 103
- not_repaired_success: 952
- recovered_from_full_batch_llm: 349
- recovered_from_rule_artifacts: 389
- rule_patch_not_recoverable: 25
- unresolved_after_repair: 40

## Cost Utility
- attempt_total: 1858
- strict_success_total: 866
- normalized_success_total: 906
- replay_eligible_total: 738

## Quality Guards
- oracle::panic_or_unwrap_driven: 227
- oracle::smoke_or_no_assertion: 824
- oracle::strong_assertion: 807
- coverage_effective_rate: 0.529412
- flaky_rate_pilot: 0/6 = 0.0

## Coverage Replay Plan
- Crates: humantime, itoa, log, mio, rand, rust-crc32fast, rustc-demangle, ryu, semver
- Tracks: canonical
- Canonical scenarios: baseline, successful_only, successful_plus_repaired, repaired_only
- Shadow scenarios: rug_generated_union

## Canonical Recovery
- run_total: 17
- pass_gated_run_total: 17
- stabilized_pass_gated_run_total: 17
- failure::success: 68
- validity::raw_count: 17
- validity::pass_gated_count: 17
- validity::stabilized_pass_gated_count: 17
- canonical::original_generation_total: 2335
- canonical::failed_generation_attempt_total: 1858
- canonical::original_success_asset_total: 477
- canonical::original_success_added_coverage_lines_total: 4410.0
- canonical::original_success_added_coverage_functions_total: 779.0
- canonical::original_success_added_coverage_regions_total: 9166.0
- canonical::repaired_added_coverage_lines_total: 679.0
- canonical::repaired_added_coverage_functions_total: 77.0
- canonical::repaired_added_coverage_regions_total: 1174.0
- canonical::replay_eligible_repaired_total: 738
- canonical::compile_executable_repaired_total: 42
- canonical::coverage_contributing_repaired_total: 42
- canonical::runtime_quarantined_original_total: 38
- canonical::runtime_quarantined_repaired_total: 7
- canonical::source_mismatch_pending_total: 0
- canonical::positive_repaired_run_total: 9
- canonical::positive_repaired_crate_total: 6
- canonical::positive_repaired_crates: ['humantime', 'itoa', 'log', 'mio', 'rand', 'rustc-demangle']
- canonical::max_positive_repaired_crate_share_lines: 0.428571

## Fixed-Model Cross-Crate
- model: gemini-2.5-flash-nothinking
- run_total: 9
- positive_run_total: 5
- run::humantime::gemini-2.5-flash-nothinking: failed=85, repaired_pool=48, compile_executable=0, delta(L/F/R)=0.0/0.0/0.0
- run::itoa::gemini-2.5-flash-nothinking: failed=11, repaired_pool=8, compile_executable=8, delta(L/F/R)=212.0/29.0/505.0
- run::log::gemini-2.5-flash-nothinking: failed=145, repaired_pool=41, compile_executable=10, delta(L/F/R)=121.0/10.0/141.0
- run::mio::gemini-2.5-flash-nothinking: failed=175, repaired_pool=6, compile_executable=6, delta(L/F/R)=24.0/4.0/59.0
- run::rand::gemini-2.5-flash-nothinking: failed=355, repaired_pool=165, compile_executable=2, delta(L/F/R)=14.0/3.0/26.0
- run::rust-crc32fast::gemini-2.5-flash-nothinking: failed=9, repaired_pool=9, compile_executable=0, delta(L/F/R)=0.0/0.0/0.0
- run::rustc-demangle::gemini-2.5-flash-nothinking: failed=150, repaired_pool=19, compile_executable=2, delta(L/F/R)=17.0/2.0/18.0
- run::ryu::gemini-2.5-flash-nothinking: failed=14, repaired_pool=7, compile_executable=0, delta(L/F/R)=0.0/0.0/0.0
- run::semver::gemini-2.5-flash-nothinking: failed=55, repaired_pool=10, compile_executable=0, delta(L/F/R)=0.0/0.0/0.0

## Humantime Cross-Model
- crate: humantime
- run_total: 9
- positive_run_total: 4
- run::claude-3-5-haiku-20241022: failed=123, repaired_pool=68, compile_executable=4, delta(L/F/R)=123.0/12.0/157.0
- run::deepseek-v3: failed=80, repaired_pool=47, compile_executable=6, delta(L/F/R)=78.0/8.0/147.0
- run::gemini-2.5-flash-nothinking: failed=85, repaired_pool=48, compile_executable=0, delta(L/F/R)=0.0/0.0/0.0
- run::gemini-2.5-flash-thinking: failed=70, repaired_pool=28, compile_executable=3, delta(L/F/R)=86.0/8.0/116.0
- run::gpt-3.5-turbo: failed=114, repaired_pool=36, compile_executable=0, delta(L/F/R)=0.0/0.0/0.0
- run::gpt-4.1-mini: failed=101, repaired_pool=65, compile_executable=0, delta(L/F/R)=0.0/0.0/0.0
- run::gpt-4.1-nano: failed=128, repaired_pool=66, compile_executable=0, delta(L/F/R)=0.0/0.0/-1.0
- run::gpt-4o-mini: failed=121, repaired_pool=57, compile_executable=0, delta(L/F/R)=0.0/0.0/0.0
- run::gpt-5-nano: failed=122, repaired_pool=58, compile_executable=1, delta(L/F/R)=4.0/1.0/6.0

## Shadow Observation
- run_total: 17
- failure::success: 6
- failure::test_failed: 11
- shadow::ready_run_total: 6
- shadow::matched_test_total: 858

## Freshness Gate
- passed: True
- coverage_metrics_generated_at_utc: 2026-04-15T14:22:55+00:00
- run_results_generated_at_utc: 2026-04-15T14:22:48+00:00

## Notes
- Canonical replay is the primary LLM-only recovery result on pristine crates.
- Coverage is reported with lines/functions/regions; branch coverage is currently not part of the exported canonical summary.
- Fixed-model cross-crate uses only gemini-2.5-flash-nothinking.
- Humantime is reported only as a fixed-crate cross-model slice and should not be collapsed into a cross-crate aggregate.
- Shadow observation is appendix-only and reports current rug_runs generated-test assets.
- Coverage is reported as raw observed plus pass-gated conservative for canonical replay.
