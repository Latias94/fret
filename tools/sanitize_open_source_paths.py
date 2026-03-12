#!/usr/bin/env python3
"""
Sanitize developer-machine absolute paths in tracked docs/artifacts before open-source releases.

This script is intentionally conservative and only rewrites a small set of known, non-semantic
fields/areas (e.g. perf baseline `out_path`) where absolute paths add noise for readers.

Usage:
  python tools/sanitize_open_source_paths.py

Exit codes:
  0: no changes needed (or changes applied successfully)
  1: changes were applied (useful for CI gating if desired)
  2: error
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


def _repo_root() -> Path:
    try:
        out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"])
    except Exception as e:
        raise RuntimeError("failed to locate repo root via git") from e
    return Path(out.decode("utf-8").strip()).resolve()


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _write_text_if_changed(path: Path, text: str) -> bool:
    old = _read_text(path)
    if old == text:
        return False
    path.write_text(text, encoding="utf-8")
    return True


def _sanitize_perf_baseline_json(path: Path, repo_root: Path) -> bool:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as e:
        raise RuntimeError(f"failed to parse json: {path}") from e

    if not isinstance(data, dict):
        return False

    out_path = data.get("out_path")
    if not isinstance(out_path, str) or not out_path:
        return False

    # Perf baseline `out_path` is just a generation artifact pointer; keep it repo-relative.
    rel = path.resolve().relative_to(repo_root.resolve()).as_posix()
    if out_path != rel:
        data["out_path"] = rel
        path.write_text(json.dumps(data, indent=2, sort_keys=False) + "\n", encoding="utf-8")
        return True
    return False


_ABS_WIN_PREFIX_RE = re.compile(r"^[A-Za-z]:\\\\[^\\s`]+\\\\")


def _sanitize_markdown_paths(text: str) -> str:
    # Prefer keeping repo-relative paths when possible; otherwise replace with a placeholder.
    def sanitize_path(p: str) -> str:
        p = p.strip()
        if "/tools/" in p:
            return "tools/" + p.split("/tools/", 1)[1]
        if "/target/" in p:
            return "target/" + p.split("/target/", 1)[1]
        if "/docs/" in p:
            return "docs/" + p.split("/docs/", 1)[1]
        # Fall back to a stable placeholder.
        return "<local path>"

    # Rewrite backticked absolute paths.
    def repl_backtick(m: re.Match[str]) -> str:
        inner = m.group(1)
        if inner.startswith("/Users/"):
            return f"`{sanitize_path(inner)}`"
        if _ABS_WIN_PREFIX_RE.match(inner):
            # Try to keep common repo-relative subtrees.
            for marker in ("tools\\", "target\\", "docs\\"):
                idx = inner.find(marker)
                if idx >= 0:
                    return f"`{inner[idx:].replace('\\\\', '/')}`"
            return "`<local path>`"
        return m.group(0)

    text = re.sub(r"`([^`]+)`", repl_backtick, text)

    # Rewrite non-backticked absolute POSIX paths (common in markdown tables).
    def repl_posix(m: re.Match[str]) -> str:
        full = m.group(0)
        if "/tools/" in full:
            return "tools/" + full.split("/tools/", 1)[1]
        if "/target/" in full:
            return "target/" + full.split("/target/", 1)[1]
        if "/docs/" in full:
            return "docs/" + full.split("/docs/", 1)[1]
        return "<local path>"

    text = re.sub(r"/Users/[^\s|]+", repl_posix, text)

    # If a previous sanitization pass produced partially-rewritten placeholders (e.g.
    # "<local path>.../tools/..."), canonicalize them to repo-relative paths.
    text = re.sub(r"<local path>[^\s`|]*/tools/", "tools/", text)
    text = re.sub(r"<local path>[^\s`|]*/target/", "target/", text)
    text = re.sub(r"<local path>[^\s`|]*/docs/", "docs/", text)
    text = re.sub(r"<local path>[^\s`|]+", "<local path>", text)

    return text


def main() -> int:
    try:
        repo_root = _repo_root()
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 2

    changed = False

    # 1) Perf baseline JSON: normalize `out_path` to the file's repo-relative path.
    baselines_dir = repo_root / "docs/workstreams/perf-baselines"
    if baselines_dir.is_dir():
        for path in sorted(baselines_dir.glob("*.json")):
            if _sanitize_perf_baseline_json(path, repo_root):
                changed = True

    # 2) Known markdown log file(s) with developer-machine absolute paths.
    md_paths = [
        repo_root / "docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md",
    ]
    for path in md_paths:
        if not path.is_file():
            continue
        new_text = _sanitize_markdown_paths(_read_text(path))
        if _write_text_if_changed(path, new_text):
            changed = True

    return 1 if changed else 0


if __name__ == "__main__":
    raise SystemExit(main())
