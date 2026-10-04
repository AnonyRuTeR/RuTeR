"""Offline tests for complete sweeps followed by deferred API-only retries."""

import json
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import run_full_paired_token_experiment as pipeline
from run_claude_code_experiment import paired_ruter_values
from run_token_cost_experiment import record_infrastructure_failure, write_json


class DeferredPipelineTests(unittest.TestCase):
    def simulate(self, recover=True, passes=2):
        calls = []
        pending = {"ruter": [{"case_id": "r1"}], "claude": [{"case_id": "c1"}]}
        roots = [Path("/tmp") / name for name in ("ruter", "claude", "control")]
        args = SimpleNamespace(deferred_retry_passes=passes)

        def fake_run(command, env, log_path):
            is_retry = "--retry-infrastructure-failures" in command
            self.assertNotIn("--stop-on-infrastructure-failure", command)
            stage = command[0]
            calls.append((stage, is_retry))
            if recover and is_retry:
                pending[stage] = []
            return 0

        with patch.object(pipeline, "run", side_effect=fake_run), \
             patch.object(pipeline, "write_status"), \
             patch.object(pipeline, "write_pending_api_failures", side_effect=lambda *a: pending), \
             patch.object(pipeline, "assert_ruter_complete") as ruter_audit, \
             patch.object(pipeline, "assert_claude_complete") as claude_audit:
            result = pipeline.run_deferred_pipeline(
                args, 1858,
                ["ruter", "--resume", "--stop-on-infrastructure-failure", "--retry-infrastructure-failures"],
                ["claude", "--resume", "--stop-on-infrastructure-failure", "--retry-infrastructure-failures"],
                ["summary"], *roots, {}, {}, {},
            )
            if result:
                ruter_audit.assert_called_once()
                claude_audit.assert_called_once()
            else:
                ruter_audit.assert_not_called()
                claude_audit.assert_not_called()
        return result, calls

    def test_both_full_sweeps_precede_retries(self):
        result, calls = self.simulate()
        self.assertTrue(result)
        self.assertEqual(calls, [
            ("ruter", False), ("summary", False), ("claude", False),
            ("ruter", True), ("summary", False), ("claude", True),
        ])

    def test_persistent_failures_have_bounded_deferred_passes(self):
        result, calls = self.simulate(recover=False)
        self.assertFalse(result)
        self.assertEqual(sum(stage == "ruter" and retry for stage, retry in calls), 2)
        self.assertEqual(sum(stage == "claude" and retry for stage, retry in calls), 2)

    def test_zero_retry_passes_still_runs_both_full_sweeps(self):
        result, calls = self.simulate(passes=0)
        self.assertFalse(result)
        self.assertEqual(calls, [("ruter", False), ("summary", False), ("claude", False)])

    def test_incomplete_ruter_is_not_zero_or_failure_in_pair(self):
        self.assertEqual(paired_ruter_values({
            "analysis_included": "False", "total_tokens": "100", "strict_success": "False",
        }), {"ruter_total_tokens": None, "ruter_strict_success": None})

    def test_complete_rule_only_pair_keeps_zero(self):
        self.assertEqual(paired_ruter_values({
            "analysis_included": "True", "total_tokens": "0", "strict_success": "True",
        }), {"ruter_total_tokens": 0, "ruter_strict_success": True})

    def test_failure_log_contains_402_metadata_without_secrets(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            result_path = root / "cases/case_1/case_result.json"
            write_json(result_path, {
                "case_id": "case_1", "attempt_uid": "u1", "crate": "crate",
                "artifact_dir": "ruter_artifacts", "status": "not_repaired",
                "command": ["secret-must-not-be-logged"],
            })
            write_json(result_path.parent / "ruter_artifacts/4_llm_usage.json", {
                "requests": [{"http_status": 200}, {"http_status": 402}],
                "summary": {"request_count": 2},
            })
            record_infrastructure_failure(root, "ruter", result_path)
            raw = (root / "api_failures.jsonl").read_text()
            data = json.loads(raw)
            self.assertEqual(data["http_402_count"], 1)
            self.assertEqual(data["http_statuses"], [200, 402])
            self.assertFalse(data["included_in_primary_analysis"])
            self.assertNotIn("secret-must-not-be-logged", raw)


if __name__ == "__main__":
    unittest.main()
