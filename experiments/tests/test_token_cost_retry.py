"""Regression tests separating repair failures from infrastructure retries."""

import copy
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
from run_token_cost_experiment import is_retryable_infrastructure_failure, write_json


class InfrastructureRetryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.result_path = self.root / "case_result.json"
        self.result = {
            "repair_model": "gemini-2.5-flash-nothinking",
            "status": "not_repaired",
            "timed_out": False,
            "artifact_dir": "ruter_artifacts",
        }
        self.usage = {
            "configured_model": "gemini-2.5-flash-nothinking",
            "requests": [{
                "returned_model": "gemini-2.5-flash",
                "http_status": 200,
                "outcome": "success",
            }],
            "summary": {
                "request_count": 1,
                "usage_reported_request_count": 1,
                "usage_missing_request_count": 0,
                "failed_request_count": 0,
            },
        }

    def retryable(self, include_usage=True):
        write_json(self.result_path, self.result)
        if include_usage:
            write_json(self.root / "ruter_artifacts/4_llm_usage.json", self.usage)
        return is_retryable_infrastructure_failure(self.result_path)

    def test_valid_request_does_not_retry_not_repaired_case(self):
        self.assertFalse(self.retryable())

    def test_invalid_candidate_output_is_method_failure(self):
        self.usage["requests"][0]["outcome"] = "invalid_candidate_output"
        self.usage["summary"]["failed_request_count"] = 1
        self.assertFalse(self.retryable())

    def test_three_invalid_candidates_do_not_get_extra_attempts(self):
        request = self.usage["requests"][0]
        request["outcome"] = "invalid_candidate_output"
        self.usage["requests"] = [copy.deepcopy(request) for _ in range(3)]
        self.usage["summary"].update(
            request_count=3, usage_reported_request_count=3, failed_request_count=3
        )
        self.assertFalse(self.retryable())

    def test_http_402_is_infrastructure_failure(self):
        self.usage["requests"][0].update(http_status=402, outcome="http_error")
        self.assertTrue(self.retryable())

    def test_missing_usage_is_infrastructure_failure(self):
        self.usage["summary"].update(
            usage_reported_request_count=0, usage_missing_request_count=1
        )
        self.assertTrue(self.retryable())

    def test_request_count_mismatch_is_infrastructure_failure(self):
        self.usage["summary"].update(request_count=2, usage_reported_request_count=2)
        self.assertTrue(self.retryable())

    def test_wrong_model_is_infrastructure_failure(self):
        self.usage["requests"][0]["returned_model"] = "other-model"
        self.assertTrue(self.retryable())

    def test_rule_only_case_does_not_require_model_response(self):
        self.usage["requests"] = []
        self.usage["summary"].update(request_count=0, usage_reported_request_count=0)
        self.assertFalse(self.retryable())

    def test_timeout_is_method_outcome(self):
        self.result["timed_out"] = True
        self.assertFalse(self.retryable(include_usage=False))

    def test_runtime_failure_without_usage_is_retryable(self):
        self.result["status"] = "runtime_failed"
        self.assertTrue(self.retryable(include_usage=False))


if __name__ == "__main__":
    unittest.main()
