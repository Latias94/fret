#!/usr/bin/env python3
"""
Reports the largest Rust source files in the repository by line count.

Cross-platform replacement for `tools/report_largest_files.ps1`.

This is a lightweight drift detector for "god files" that tend to appear during rapid refactors.
It intentionally does not depend on Cargo metadata so it can be used before the workspace builds.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path


def _split_csv(values: list[str]) -> list[str]:
    out: list[str] = []
    for v in values:
        for part in v.split(","):
            p = part.strip().strip("/\\")
            if p:
                out.append(p)
    return out


def _repo_relative(base: Path, full: Path) -> str:
    try:
        rel = full.resolve().relative_to(base.resolve())
        return rel.as_posix()
    except Exception:
        return str(full.resolve())


def _count_lines(path: Path) -> int:
    count = 0
    with path.open("r", encoding="utf-8", errors="replace", newline="") as f:
        for _ in f:
            count += 1
    return count


def _is_included(rel: str, include_prefixes: list[str]) -> bool:
    rel_lower = rel.lower()
    for prefix in include_prefixes:
        p = prefix.lower().rstrip("/")
        if rel_lower == p:
            return True
        if rel_lower.startswith(p + "/"):
            return True
    return False


def _is_excluded(rel: str, exclude_segments: list[str]) -> bool:
    rel_lower = rel.lower()
    parts = [p for p in rel_lower.split("/") if p]
    for seg in exclude_segments:
        s = seg.lower().strip("/").strip("\\")
        if not s:
            continue
        if s in rel_lower:
            # Fallback: segment may not align with separators (e.g. ".git").
            return True
        if any(p == s for p in parts):
            return True
    return False


def _print_table(rows: list[dict[str, object]]) -> None:
    if not rows:
        print("(no matching files)")
        return
    path_width = max(len(str(r["path"])) for r in rows)
    lines_width = max(len(str(r["lines"])) for r in rows)
    path_width = max(path_width, len("path"))
    lines_width = max(lines_width, len("lines"))

    print(f"{'path'.ljust(path_width)}  {'lines'.rjust(lines_width)}")
    print(f"{'-' * path_width}  {'-' * lines_width}")
    for r in rows:
        print(f"{str(r['path']).ljust(path_width)}  {str(r['lines']).rjust(lines_width)}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".", help="Repository root (defaults to current directory).")
    parser.add_argument("--top", type=int, default=50, help="Number of entries to print.")
    parser.add_argument("--min-lines", type=int, default=0, help="Only include files with at least this many lines.")
    parser.add_argument(
        "--include",
        action="append",
        default=[],
        help='Only include paths that start with one of these prefixes (repeatable or comma-separated). Default: "crates,ecosystem,apps,tools,themes,docs".',
    )
    parser.add_argument(
        "--exclude",
        action="append",
        default=[],
        help='Exclude any file whose path contains one of these segments (repeatable or comma-separated). Default: "target,.git,repo-ref".',
    )
    parser.add_argument("--json", action="store_true", help="Emit JSON instead of a table.")
    args = parser.parse_args(argv)

    root = Path(args.root).resolve()
    if not root.exists():
        raise RuntimeError(f"Root not found: {root}")

    include_prefixes = _split_csv(args.include) if args.include else []
    if not include_prefixes:
        include_prefixes = ["crates", "ecosystem", "apps", "tools", "themes", "docs"]

    exclude_segments = _split_csv(args.exclude) if args.exclude else []
    if not exclude_segments:
        exclude_segments = ["target", ".git", "repo-ref"]

    rows: list[dict[str, object]] = []
    for path in root.rglob("*.rs"):
        if not path.is_file():
            continue
        rel = _repo_relative(root, path)

        if not _is_included(rel, include_prefixes):
            continue
        if _is_excluded(rel, exclude_segments):
            continue

        lines = _count_lines(path)
        if lines < args.min_lines:
            continue

        rows.append({"path": rel, "lines": lines})

    rows.sort(key=lambda r: int(r["lines"]), reverse=True)
    rows = rows[: max(args.top, 0)]

    if args.json:
        print(json.dumps(rows, indent=2))
    else:
        _print_table(rows)

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
