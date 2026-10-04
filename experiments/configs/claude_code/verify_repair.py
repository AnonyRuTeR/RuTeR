#!/usr/bin/env python3
"""Claude Stop hook: require `cargo check --tests` to exit successfully."""

from __future__ import annotations

import json
import subprocess
import sys


def main() -> int:
    try:
        hook_input = json.load(sys.stdin)
    except (json.JSONDecodeError, OSError):
        hook_input = {}

    # Claude Code sets this after a Stop hook has already continued the turn.
    # Blocking recursively can create an infinite loop, so give the model one
    # verified continuation per attempted stop. The outer runner still performs
    # the authoritative final check.
    if hook_input.get("stop_hook_active") is True:
        return 0

    try:
        result = subprocess.run(
            ["cargo", "check", "--tests", "--message-format=json"],
            text=True,
            capture_output=True,
            check=False,
            timeout=240,
        )
    except subprocess.TimeoutExpired:
        print(
            "Verification timed out. Continue repairing and run cargo check --tests.",
            file=sys.stderr,
        )
        return 2

    errors: list[str] = []
    for line in result.stdout.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        message = event.get("message") if isinstance(event, dict) else None
        if (
            event.get("reason") == "compiler-message"
            and isinstance(message, dict)
            and message.get("level") == "error"
        ):
            rendered = message.get("rendered") or message.get("message")
            if rendered:
                errors.append(str(rendered).strip())

    if result.returncode == 0 and not errors:
        return 0

    detail = "\n\n".join(errors[:8])
    if not detail:
        detail = (result.stderr or "cargo check --tests failed").strip()[-8000:]
    print(
        "Verification failed (cargo check --tests did not exit 0). "
        "Do not stop; fix the remaining error(s), run cargo check --tests, and "
        f"continue until it exits 0.\n\n{detail}",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
