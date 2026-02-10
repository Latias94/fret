#!/usr/bin/env python3
"""
Execution surface checks (cross-platform).

Ported from `tools/check_execution_surface.ps1`.
See `docs/dependency-policy.md` and related ADR/workstream notes.
"""

from __future__ import annotations

import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Rule:
    name: str
    pattern: re.Pattern[str]
    scope_prefixes: tuple[str, ...]
    allowlist: tuple[str, ...]


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _normalize_repo_path(repo_root: Path, path: Path) -> str:
    try:
        rel = path.resolve().relative_to(repo_root)
        return rel.as_posix()
    except Exception:
        return path.as_posix()


def _is_allowed_path(rel: str, allowlist: tuple[str, ...]) -> bool:
    for allowed in allowlist:
        if allowed.endswith("/"):
            if rel.startswith(allowed):
                return True
            continue
        if rel == allowed:
            return True
    return False


def main(argv: list[str]) -> int:
    _ = argv

    repo_root = _repo_root()
    roots = ("crates", "ecosystem", "apps")

    rules = [
        Rule(
            name="no-raw-thread-spawn",
            pattern=re.compile(r"\b(std::thread::spawn|thread::spawn)\b"),
            scope_prefixes=("crates/", "ecosystem/", "apps/"),
            allowlist=(
                # Runner-owned concurrency wiring.
                "crates/fret-launch/src/runner/desktop/dispatcher.rs",
                "crates/fret-launch/src/runner/desktop/hotpatch.rs",
                # Diagnostics and devtools are allowed to use raw threads.
                "crates/fret-diag-ws/src/client.rs",
                "crates/fret-diag-ws/src/server.rs",
                "apps/fret-devtools/src/main.rs",
                "apps/fret-devtools/src/pack.rs",
                "apps/fret-devtools-mcp/src/main.rs",
            ),
        ),
        Rule(
            name="no-raw-thread-sleep",
            pattern=re.compile(r"\b(std::thread::sleep|thread::sleep)\b"),
            scope_prefixes=("crates/", "ecosystem/", "apps/"),
            allowlist=(
                # Tooling is allowed to poll/wait.
                "apps/fretboard/",
                "crates/fret-launch/src/runner/desktop/hotpatch.rs",
                "crates/fret-diag/src/lib.rs",
                "crates/fret-diag/src/compare.rs",
                "crates/fret-diag/src/stats.rs",
                "crates/fret-diag-ws/src/client.rs",
                "apps/fret-examples/src/todo_demo.rs",
            ),
        ),
        Rule(
            name="no-bespoke-channels",
            pattern=re.compile(r"\b(std::sync::mpsc|crossbeam_channel|async_channel|flume)\b"),
            scope_prefixes=("ecosystem/", "apps/"),
            allowlist=(
                # Devtools uses a local mpsc worker for packing artifacts.
                "apps/fret-devtools/src/main.rs",
                "apps/fret-devtools/src/pack.rs",
            ),
        ),
        Rule(
            name="no-bespoke-futures-channels",
            pattern=re.compile(r"\b(futures::channel::(mpsc|oneshot)|futures_channel)\b"),
            scope_prefixes=("ecosystem/", "apps/"),
            allowlist=(),
        ),
        Rule(
            name="no-split-brain-timers",
            pattern=re.compile(
                r"\b(gloo_timers|futures_timer|wasm_timer|tokio::time::sleep|async_std::task::sleep)\b"
            ),
            scope_prefixes=("ecosystem/", "apps/"),
            allowlist=(
                # MCP server uses Tokio internally; it is not part of the UI runtime scheduling contract.
                "apps/fret-devtools-mcp/src/main.rs",
            ),
        ),
    ]

    had_errors = False

    for root in roots:
        base = repo_root / root
        if not base.exists():
            continue
        for path in base.rglob("*.rs"):
            if not path.is_file():
                continue
            rel = _normalize_repo_path(repo_root, path)

            # Read once per file.
            try:
                text = path.read_text(encoding="utf-8", errors="replace").splitlines()
            except Exception as e:
                print(f"error: failed to read {rel}: {e}", file=sys.stderr)
                had_errors = True
                continue

            for rule in rules:
                if not any(rel.startswith(prefix) for prefix in rule.scope_prefixes):
                    continue
                if _is_allowed_path(rel, rule.allowlist):
                    continue

                for idx, line in enumerate(text, start=1):
                    if rule.pattern.search(line) is None:
                        continue
                    print(
                        f"Execution surface violation ({rule.name}): {rel}:{idx}: {line.strip()}",
                        file=sys.stderr,
                    )
                    had_errors = True

    if had_errors:
        return 1
    print("Execution surface check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
