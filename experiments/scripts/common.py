from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import tomllib

ROOT = Path(__file__).resolve().parents[2]
RUTER_ROOT = ROOT / "ruter"


@dataclass
class Subject:
    raw: dict[str, Any]
    subject_id: str
    crate_name: str
    source_type: str
    rust_toolchain: str
    path: str | None
    repo_url: str | None
    commit_hash: str | None
    target_functions: list[str]
    expected_error_codes: list[str]
    notes: str

    def resolved_path(self) -> Path | None:
        if self.path:
            candidate = Path(self.path)
            if not candidate.is_absolute():
                candidate = ROOT / candidate
            return candidate.resolve()
        return None


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def ensure_dir(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    return path


def write_json(path: Path, data: Any) -> None:
    ensure_dir(path.parent)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False))


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    ensure_dir(path.parent)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(row, ensure_ascii=False) + "\n")


def load_manifest(path: Path) -> list[Subject]:
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    subjects = []
    for entry in data.get("subject", []):
        subjects.append(
            Subject(
                raw=entry,
                subject_id=entry["subject_id"],
                crate_name=entry["crate_name"],
                source_type=entry["source_type"],
                rust_toolchain=entry.get("rust_toolchain", "stable"),
                path=entry.get("path"),
                repo_url=entry.get("repo_url"),
                commit_hash=entry.get("commit_hash"),
                target_functions=list(entry.get("target_functions", [])),
                expected_error_codes=list(entry.get("expected_error_codes", [])),
                notes=entry.get("notes", ""),
            )
        )
    return subjects


def default_subject_command(subject_dir: Path) -> list[str]:
    return ["cargo", "check", "--tests", "--message-format=json"]


def run_command(command: list[str], cwd: Path, env: dict[str, str] | None = None, timeout: int | None = None) -> subprocess.CompletedProcess[str]:
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    return subprocess.run(
        command,
        cwd=str(cwd),
        env=merged_env,
        text=True,
        capture_output=True,
        timeout=timeout,
    )


def iter_cargo_json(stdout: str) -> list[dict[str, Any]]:
    items: list[dict[str, Any]] = []
    for line in stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict):
            items.append(value)
    return items


def _collect_span_suggestions(spans: list[dict[str, Any]], subject_root: Path) -> list[dict[str, Any]]:
    suggestions: list[dict[str, Any]] = []
    for span in spans or []:
        replacement = span.get("suggested_replacement")
        if replacement is None:
            continue
        file_name = span.get("file_name")
        abs_path: str | None = None
        if isinstance(file_name, str):
            file_path = Path(file_name)
            if not file_path.is_absolute():
                file_path = (subject_root / file_path).resolve()
            abs_path = str(file_path)
        suggestions.append(
            {
                "file": file_name,
                "abs_path": abs_path,
                "byte_start": span.get("byte_start"),
                "byte_end": span.get("byte_end"),
                "line_start": span.get("line_start"),
                "line_end": span.get("line_end"),
                "column_start": span.get("column_start"),
                "column_end": span.get("column_end"),
                "replacement": replacement,
                "applicability": span.get("suggestion_applicability"),
                "is_primary": span.get("is_primary"),
                "label": span.get("label"),
            }
        )
    return suggestions


def extract_diagnostics_from_cargo_stdout(subject_id: str, subject_root: Path, stdout: str) -> list[dict[str, Any]]:
    diagnostics: list[dict[str, Any]] = []
    for item in iter_cargo_json(stdout):
        message = None
        if item.get("reason") == "compiler-message" and isinstance(item.get("message"), dict):
            message = item["message"]
        elif item.get("$message_type") == "diagnostic":
            message = item
        if not isinstance(message, dict):
            continue

        spans = message.get("spans") or []
        primary = next((span for span in spans if span.get("is_primary")), spans[0] if spans else None)
        suggestions = _collect_span_suggestions(spans, subject_root)
        for child in message.get("children") or []:
            if isinstance(child, dict):
                suggestions.extend(_collect_span_suggestions(child.get("spans") or [], subject_root))

        diagnostics.append(
            {
                "subject_id": subject_id,
                "diagnostic_level": message.get("level"),
                "error_code": (message.get("code") or {}).get("code"),
                "message": message.get("message"),
                "rendered": message.get("rendered"),
                "file": primary.get("file_name") if isinstance(primary, dict) else None,
                "line_start": primary.get("line_start") if isinstance(primary, dict) else None,
                "line_end": primary.get("line_end") if isinstance(primary, dict) else None,
                "column_start": primary.get("column_start") if isinstance(primary, dict) else None,
                "column_end": primary.get("column_end") if isinstance(primary, dict) else None,
                "byte_start": primary.get("byte_start") if isinstance(primary, dict) else None,
                "byte_end": primary.get("byte_end") if isinstance(primary, dict) else None,
                "suggestions": suggestions,
                "raw": message,
            }
        )
    return diagnostics


def error_code_counts(diagnostics: list[dict[str, Any]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for diag in diagnostics:
        code = diag.get("error_code") or "NO_CODE"
        counts[code] = counts.get(code, 0) + 1
    return counts


def copy_subject_to_temp(subject_root: Path) -> tuple[Path, tempfile.TemporaryDirectory[str]]:
    temp = tempfile.TemporaryDirectory(prefix="rq_baseline_")
    temp_root = Path(temp.name) / subject_root.name
    shutil.copytree(subject_root, temp_root)
    return temp_root, temp


def machine_applicable_suggestions(diagnostics: list[dict[str, Any]], subject_root: Path) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    subject_root = subject_root.resolve()
    for diag in diagnostics:
        for suggestion in diag.get("suggestions", []):
            if suggestion.get("applicability") != "MachineApplicable":
                continue
            abs_path = suggestion.get("abs_path")
            if not abs_path:
                continue
            path = Path(abs_path).resolve()
            try:
                path.relative_to(subject_root)
            except ValueError:
                continue
            if suggestion.get("byte_start") is None or suggestion.get("byte_end") is None:
                continue
            out.append(
                {
                    "abs_path": str(path),
                    "byte_start": int(suggestion["byte_start"]),
                    "byte_end": int(suggestion["byte_end"]),
                    "replacement": suggestion.get("replacement", ""),
                    "file": suggestion.get("file"),
                    "line_start": suggestion.get("line_start"),
                    "line_end": suggestion.get("line_end"),
                }
            )
    out.sort(key=lambda item: (item["abs_path"], item["byte_start"], item["byte_end"]))

    filtered: list[dict[str, Any]] = []
    last_end_by_file: dict[str, int] = {}
    seen: set[tuple[str, int, int, str]] = set()
    for item in out:
        key = (item["abs_path"], item["byte_start"], item["byte_end"], item["replacement"])
        if key in seen:
            continue
        seen.add(key)
        last_end = last_end_by_file.get(item["abs_path"])
        if last_end is not None and item["byte_start"] < last_end:
            continue
        last_end_by_file[item["abs_path"]] = item["byte_end"]
        filtered.append(item)
    return filtered


def apply_suggestions_in_place(workspace_root: Path, suggestions: list[dict[str, Any]]) -> list[str]:
    by_file: dict[Path, list[dict[str, Any]]] = {}
    modified_files: list[str] = []
    for item in suggestions:
        path = Path(item["abs_path"])
        try:
            rel = path.relative_to(workspace_root)
        except ValueError:
            rel = path.name
        by_file.setdefault(path, []).append(item)
        if str(rel) not in modified_files:
            modified_files.append(str(rel))

    for path, items in by_file.items():
        content = path.read_bytes()
        for item in sorted(items, key=lambda entry: entry["byte_start"], reverse=True):
            start = item["byte_start"]
            end = item["byte_end"]
            replacement = str(item["replacement"]).encode("utf-8")
            if start < 0 or end < start or end > len(content):
                continue
            content = content[:start] + replacement + content[end:]
        path.write_bytes(content)
    return modified_files
