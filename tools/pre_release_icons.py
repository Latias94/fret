#!/usr/bin/env python3
"""
Icon pre-release checks (cross-platform).

Ported from `tools/pre_release_icons.ps1`.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _run_checked(name: str, argv: list[str]) -> None:
    print(f"[pre-release/icons] {name}")
    proc = subprocess.run(argv, check=False)
    if proc.returncode != 0:
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-diff-check", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    py = sys.executable

    _run_checked(
        "check all icon packs generation",
        [py, str(repo_root / "tools/check_icons_generation.py"), "--pack", "all"],
    )

    if not args.skip_diff_check:
        _run_checked(
            "diff check icon-related paths",
            [
                "git",
                "diff",
                "--exit-code",
                "--",
                "ecosystem/fret-icons-lucide",
                "ecosystem/fret-icons-radix",
                "tools",
                ".gitmodules",
                "third_party/lucide",
                "third_party/radix-icons",
            ],
        )

    print("[pre-release/icons] done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

