"""No paid API calls: assignment, publishing, isolation and retry invariants."""

import json
import socket
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import run_parallel_claude_code_experiment as parallel
import run_claude_code_experiment as serial
from run_token_cost_experiment import write_json


def loopback_available():
    """Restricted execution environments may disallow all socket creation."""
    try:
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
        return True
    except PermissionError:
        return False


def case(index):
    return {"case_id": f"case_{index:04}", "attempt_uid": f"uid_{index}",
            "crate": "crate", "sampling_weight": 1.0}


def result(index=1, status="not_repaired", valid=True, finished="2026-10-01T01:00:00+00:00"):
    return {**case(index), "status": status, "strict_success": status == "repaired",
            "eligible": True, "model_valid": valid, "finished_at_utc": finished,
            "usage": {"provider_usage_complete": valid, "usage_complete": valid,
                      "request_count": 1, "successful_request_count": int(valid),
                      "total_tokens": 100}}


class ParallelClaudeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.out = self.root / "out"
        self.source = self.root / "worker/cases/case_0001"
        self.expected = parallel.validate_cases([case(1)])

    def source_result(self, value=None, marker=True):
        write_json(self.source / "case_result.json", value or result())
        if marker:
            serial.mark_case_complete(self.source, True)

    def test_two_shards_are_disjoint_and_cover_every_case(self):
        cases = [case(i) for i in range(1858)]
        shards = parallel.shard_cases(cases, 2)
        self.assertEqual([len(s) for s in shards], [929, 929])
        self.assertEqual({r["attempt_uid"] for s in shards for r in s}, {r["attempt_uid"] for r in cases})
        self.assertFalse({r["case_id"] for r in shards[0]} & {r["case_id"] for r in shards[1]})

    def test_three_shards_have_balanced_counts(self):
        shards = parallel.shard_cases([case(i) for i in range(1858)], 3)
        self.assertEqual([len(s) for s in shards], [620, 619, 619])

    def test_duplicate_uid_rejected(self):
        with self.assertRaises(ValueError):
            parallel.shard_cases([case(1), {**case(2), "attempt_uid": "uid_1"}], 2)

    def test_path_traversal_case_rejected(self):
        with self.assertRaises(ValueError):
            parallel.validate_cases([{**case(1), "case_id": "../case_1"}])

    def test_no_marker_means_not_publishable(self):
        self.source_result(marker=False)
        self.assertFalse(parallel.publish_result(self.source, self.out, self.expected, 0, 2))
        self.assertFalse((self.out / "cases/case_0001").exists())

    def test_complete_case_is_published_and_not_double_counted(self):
        self.source_result()
        self.assertTrue(parallel.publish_result(self.source, self.out, self.expected, 0, 2))
        self.assertFalse(parallel.publish_result(self.source, self.out, self.expected, 0, 2))
        paths = list(self.out.glob("cases/*/case_result.json"))
        self.assertEqual(len(paths), 1)
        saved = json.loads(paths[0].read_text())
        self.assertEqual(saved["usage"]["total_tokens"], 100)
        self.assertEqual(saved["execution_worker_count"], 2)

    def test_ordinary_failure_is_not_replaced_by_a_later_success(self):
        path = self.out / "cases/case_0001/case_result.json"
        write_json(path, result())
        original = path.read_bytes()
        self.source_result(result(status="repaired"))
        self.assertFalse(parallel.publish_result(self.source, self.out, self.expected, 1, 2))
        self.assertEqual(path.read_bytes(), original)

    def test_api_failure_is_archived_before_replacement(self):
        path = self.out / "cases/case_0001/case_result.json"
        write_json(path, result(status="infrastructure_failed", valid=False))
        self.source_result(result(status="repaired", finished="2026-10-01T02:00:00+00:00"))
        self.assertTrue(parallel.publish_result(self.source, self.out, self.expected, 1, 2))
        self.assertEqual(len(list(self.out.glob("_aborted_cases/*/case_result.json"))), 1)
        self.assertTrue(json.loads(path.read_text())["strict_success"])

    def test_older_api_failure_cannot_replace_newer_api_failure(self):
        write_json(self.out / "cases/case_0001/case_result.json",
                   result(status="infrastructure_failed", valid=False, finished="2026-10-01T03:00:00+00:00"))
        self.source_result(result(status="infrastructure_failed", valid=False))
        self.assertFalse(parallel.publish_result(self.source, self.out, self.expected, 0, 2))

    def test_worker_cannot_publish_a_different_uid(self):
        self.source_result({**result(), "attempt_uid": "other_uid"})
        with self.assertRaises(ValueError):
            parallel.publish_result(self.source, self.out, self.expected, 0, 2)

    def test_deferred_mode_does_not_immediately_retry_api_failures(self):
        write_json(self.out / "cases/case_0001/case_result.json", result(status="infrastructure_failed", valid=False))
        self.assertEqual(parallel.pending_cases([case(1)], self.out, False), [])
        self.assertEqual(parallel.pending_cases([case(1)], self.out, True), [case(1)])

    def test_retry_pass_does_not_retry_ordinary_failures(self):
        write_json(self.out / "cases/case_0001/case_result.json", result(status="claude_failed"))
        self.assertEqual(parallel.pending_cases([case(1)], self.out, True), [])

    def test_worker_commands_isolate_ports_and_cargo_targets(self):
        args = serial.build_parser().parse_args(["--out", str(self.out)])
        first = parallel.serial_command(args, self.root / "worker0", self.root / "ids0.json", 18767, self.root / "target0")
        second = parallel.serial_command(args, self.root / "worker1", self.root / "ids1.json", 18768, self.root / "target1")
        self.assertEqual(first[first.index("--proxy-port") + 1], "18767")
        self.assertEqual(second[second.index("--proxy-port") + 1], "18768")
        self.assertNotEqual(first[first.index("--cargo-target-root") + 1], second[second.index("--cargo-target-root") + 1])
        self.assertNotIn("--stop-on-infrastructure-failure", first)

    def test_settings_override_changes_only_the_loopback_port(self):
        template = self.root / "template.json"
        original = {"env": {"ANTHROPIC_BASE_URL": "http://127.0.0.1:18766", "ANTHROPIC_MODEL": "same-model"},
                    "permissions": {"allow": ["Read"]}, "hooks": {"Stop": []}}
        write_json(template, original)
        path = parallel.worker_settings(template, self.root / "worker0_settings.json", 18767)
        modified = json.loads(path.read_text())
        self.assertEqual(modified["env"]["ANTHROPIC_BASE_URL"], "http://127.0.0.1:18767")
        modified["env"]["ANTHROPIC_BASE_URL"] = original["env"]["ANTHROPIC_BASE_URL"]
        self.assertEqual(modified, original)
        self.assertEqual(json.loads(template.read_text()), original)

    @unittest.skipUnless(loopback_available(), "loopback sockets prohibited by execution environment")
    def test_free_proxy_port_is_available(self):
        with socket.socket() as reservation:
            reservation.bind(("127.0.0.1", 0))
            port = reservation.getsockname()[1]
        parallel.check_proxy_port(port)

    @unittest.skipUnless(loopback_available(), "loopback sockets prohibited by execution environment")
    def test_live_listener_is_rejected_and_not_disrupted(self):
        with socket.socket() as listener:
            listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            listener.bind(("127.0.0.1", 0))
            listener.listen(1)
            port = listener.getsockname()[1]
            with self.assertRaisesRegex(RuntimeError, "existing services will not be stopped"):
                parallel.check_proxy_port(port)
            with socket.create_connection(("127.0.0.1", port), timeout=1):
                accepted, _ = listener.accept()
                accepted.close()

    @unittest.skipUnless(loopback_available(), "loopback sockets prohibited by execution environment")
    def test_completed_proxy_connection_allows_immediate_restart(self):
        listener = socket.socket()
        listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
        listener.listen(1)
        client = socket.create_connection(("127.0.0.1", port), timeout=1)
        accepted, _ = listener.accept()
        try:
            # The server actively closes first, leaving its port in TIME_WAIT.
            accepted.shutdown(socket.SHUT_WR)
            self.assertEqual(client.recv(1), b"")
        finally:
            client.close()
            accepted.close()
            listener.close()
        parallel.check_proxy_port(port)

    def test_interrupt_cleans_up_child_process_group(self):
        with patch.object(serial.subprocess, "Popen") as spawn, patch.object(serial.os, "killpg") as kill:
            proc = spawn.return_value
            proc.pid = 12345
            proc.poll.return_value = None
            proc.communicate.side_effect = [KeyboardInterrupt(), ("", "")]
            with self.assertRaises(KeyboardInterrupt):
                serial.run_command(["mock"], cwd=self.root, env={}, timeout=5)
            kill.assert_called_once_with(12345, serial.signal.SIGTERM)


if __name__ == "__main__":
    unittest.main()
