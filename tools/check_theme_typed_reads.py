#!/usr/bin/env python3
"""
Disallow typed theme reads (typed field access) in selected source trees.

Cross-platform replacement for `tools/check_theme_typed_reads.ps1`.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _to_repo_rel(repo_root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except Exception:
        return str(path)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--path",
        action="append",
        default=[
            "ecosystem/fret-ui-shadcn/src",
            "ecosystem/fret-ui-kit/src",
            "apps/fret-examples/src",
            "apps/fret-demo/src",
        ],
        help="Search root (repeatable).",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    pattern = re.compile(r"theme\\.colors\\.|theme\\.metrics\\.")

    matches: list[str] = []
    for raw in args.path:
        root = (repo_root / raw).resolve()
        if not root.exists():
            continue
        for path in root.rglob("*.rs"):
            if not path.is_file():
                continue
            rel = _to_repo_rel(repo_root, path)
            try:
                with path.open("r", encoding="utf-8", errors="replace") as f:
                    for line_no, line in enumerate(f, start=1):
                        if pattern.search(line) is not None:
                            matches.append(f"{rel}:{line_no}:{line.strip()}")
            except OSError as exc:
                matches.append(f"{rel}:0:{exc}")

    if matches:
        print("Found forbidden typed theme reads:", file=sys.stderr)
        for m in matches:
            print(m, file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
