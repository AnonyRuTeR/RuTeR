#!/usr/bin/env python3
"""Loopback Anthropic API proxy that records usage without recording content."""

from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import os
import threading
import time
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit


USAGE_FIELDS = (
    "input_tokens",
    "cache_creation_input_tokens",
    "cache_read_input_tokens",
    "output_tokens",
)
HOP_BY_HOP = {
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
}


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def as_nonnegative_int(value: Any) -> int | None:
    if isinstance(value, bool):
        return None
    try:
        number = int(value)
    except (TypeError, ValueError):
        return None
    return number if number >= 0 else None


def merge_usage(target: dict[str, Any], candidate: Any) -> None:
    if not isinstance(candidate, dict):
        return
    for field in USAGE_FIELDS:
        value = as_nonnegative_int(candidate.get(field))
        if value is not None:
            # Anthropic streaming usage is cumulative within a response. Taking the
            # maximum avoids double-counting message_start and message_delta events.
            target[field] = max(int(target.get(field, 0)), value)
    billing = candidate.get("billing_usage")
    if isinstance(billing, dict):
        target["billing_usage"] = billing


def inspect_event(event: Any, usage: dict[str, Any], metadata: dict[str, Any]) -> None:
    if not isinstance(event, dict):
        return
    merge_usage(usage, event.get("usage"))
    message = event.get("message")
    if isinstance(message, dict):
        merge_usage(usage, message.get("usage"))
        if message.get("model"):
            metadata["returned_model"] = str(message["model"])
        if message.get("id"):
            metadata["message_id"] = str(message["id"])
    if event.get("model"):
        metadata["returned_model"] = str(event["model"])
    if event.get("id"):
        metadata["message_id"] = str(event["id"])
    if event.get("stop_reason"):
        metadata["stop_reason"] = str(event["stop_reason"])


def parse_response_usage(body: bytes, content_type: str) -> tuple[dict[str, Any], dict[str, Any]]:
    usage: dict[str, Any] = {}
    metadata: dict[str, Any] = {}
    text = body.decode("utf-8", errors="replace")
    if "text/event-stream" in content_type.lower() or text.lstrip().startswith("event:"):
        for line in text.splitlines():
            if not line.startswith("data:"):
                continue
            payload = line[5:].strip()
            if not payload or payload == "[DONE]":
                continue
            try:
                inspect_event(json.loads(payload), usage, metadata)
            except json.JSONDecodeError:
                continue
    else:
        try:
            inspect_event(json.loads(text), usage, metadata)
        except json.JSONDecodeError:
            pass
    for field in USAGE_FIELDS:
        usage.setdefault(field, 0)
    usage["gross_tokens"] = sum(int(usage[field]) for field in USAGE_FIELDS)
    billing = usage.get("billing_usage")
    if isinstance(billing, dict):
        original = billing.get("openai_usage")
        if isinstance(original, dict):
            metadata["upstream_openai_usage"] = {
                key: original.get(key)
                for key in ("prompt_tokens", "completion_tokens", "total_tokens")
                if original.get(key) is not None
            }
        if billing.get("source") is not None:
            metadata["billing_usage_source"] = billing.get("source")
        if billing.get("semantic") is not None:
            metadata["billing_usage_semantic"] = billing.get("semantic")
    return usage, metadata


class UsageProxy(ThreadingHTTPServer):
    daemon_threads = True

    def __init__(self, address: tuple[str, int], handler: type[BaseHTTPRequestHandler], *, upstream: str, api_key: str, log_path: Path):
        super().__init__(address, handler)
        parsed = urlsplit(upstream.rstrip("/"))
        if parsed.scheme not in {"http", "https"} or not parsed.hostname:
            raise ValueError("upstream must be an absolute HTTP(S) URL")
        self.upstream = parsed
        self.api_key = api_key
        self.log_path = log_path
        self.log_lock = threading.Lock()

    def append_record(self, record: dict[str, Any]) -> None:
        self.log_path.parent.mkdir(parents=True, exist_ok=True)
        line = json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n"
        with self.log_lock:
            with self.log_path.open("a", encoding="utf-8") as handle:
                handle.write(line)
                handle.flush()


class ProxyHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    server: UsageProxy

    def log_message(self, _format: str, *_args: Any) -> None:
        return

    def do_GET(self) -> None:  # noqa: N802
        if self.path == "/healthz":
            payload = b'{"status":"ok"}\n'
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
            return
        self.send_error(404)

    def do_POST(self) -> None:  # noqa: N802
        started = time.monotonic()
        content_length = as_nonnegative_int(self.headers.get("Content-Length"))
        if content_length is None:
            self.send_error(411, "Content-Length required")
            return
        request_body = self.rfile.read(content_length)
        request_meta: dict[str, Any] = {}
        try:
            parsed_request = json.loads(request_body)
            if isinstance(parsed_request, dict):
                for key in ("model", "stream", "max_tokens"):
                    if key in parsed_request:
                        request_meta[key] = parsed_request[key]
        except json.JSONDecodeError:
            pass

        base_path = self.server.upstream.path.rstrip("/")
        upstream_path = f"{base_path}{self.path}" if base_path else self.path
        headers: dict[str, str] = {}
        for key, value in self.headers.items():
            lowered = key.lower()
            if lowered in HOP_BY_HOP or lowered in {
                "host",
                "content-length",
                "authorization",
                "x-api-key",
                "accept-encoding",
            }:
                continue
            headers[key] = value
        headers["Authorization"] = f"Bearer {self.server.api_key}"
        headers["x-api-key"] = self.server.api_key
        headers["Accept-Encoding"] = "identity"
        headers["Connection"] = "close"

        connection_type = (
            http.client.HTTPSConnection
            if self.server.upstream.scheme == "https"
            else http.client.HTTPConnection
        )
        port = self.server.upstream.port
        connection = connection_type(
            self.server.upstream.hostname,
            port=port,
            timeout=600,
        )
        status = 502
        response_headers: list[tuple[str, str]] = []
        response_body = bytearray()
        error: str | None = None
        try:
            connection.request("POST", upstream_path, body=request_body, headers=headers)
            upstream_response = connection.getresponse()
            status = upstream_response.status
            response_headers = upstream_response.getheaders()
            self.send_response(status, upstream_response.reason)
            for key, value in response_headers:
                lowered = key.lower()
                if lowered in HOP_BY_HOP or lowered == "content-length":
                    continue
                self.send_header(key, value)
            self.send_header("Connection", "close")
            self.end_headers()
            while True:
                chunk = upstream_response.read(64 * 1024)
                if not chunk:
                    break
                response_body.extend(chunk)
                self.wfile.write(chunk)
                self.wfile.flush()
        except Exception as exc:  # pragma: no cover - network failure path
            error = f"{type(exc).__name__}: {exc}"
            if not self.wfile.closed:
                try:
                    self.send_error(502, "upstream request failed")
                except (BrokenPipeError, ConnectionError):
                    pass
        finally:
            connection.close()
            self.close_connection = True

        content_type = next(
            (value for key, value in response_headers if key.lower() == "content-type"),
            "",
        )
        usage, response_meta = parse_response_usage(bytes(response_body), content_type)
        request_id = next(
            (
                value
                for key, value in response_headers
                if key.lower() in {"request-id", "x-request-id"}
            ),
            None,
        )
        record = {
            "schema_version": "1",
            "recorded_at_utc": utc_now(),
            "method": "POST",
            "path": self.path,
            "request_kind": (
                "messages_count_tokens"
                if "count_tokens" in self.path
                else "messages"
                if "/messages" in self.path
                else "other"
            ),
            "request": request_meta,
            "request_body_bytes": len(request_body),
            "request_body_sha256": hashlib.sha256(request_body).hexdigest(),
            "status": status,
            "duration_sec": round(time.monotonic() - started, 3),
            "request_id": request_id,
            "response_body_bytes": len(response_body),
            "usage": usage,
            "usage_complete": status < 400 and all(
                isinstance(usage.get(field), int) for field in USAGE_FIELDS
            ),
            **response_meta,
            "error": error,
        }
        self.server.append_record(record)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--listen", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=18766)
    parser.add_argument("--upstream", default="https://4sapi.org")
    parser.add_argument("--log", required=True)
    parser.add_argument(
        "--api-key-env",
        default="CLAUDE_EXPERIMENT_API_KEY",
        help="environment variable containing the upstream API key",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()
    api_key = os.environ.get(args.api_key_env)
    if not api_key:
        raise SystemExit(f"{args.api_key_env} is required")
    server = UsageProxy(
        (args.listen, args.port),
        ProxyHandler,
        upstream=args.upstream,
        api_key=api_key,
        log_path=Path(args.log).resolve(),
    )
    print(f"READY http://{args.listen}:{args.port}", flush=True)
    try:
        server.serve_forever(poll_interval=0.2)
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
