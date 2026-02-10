#!/usr/bin/env python3
"""
Pre-release check runner (cross-platform).

Ported from `tools/pre_release.ps1` to avoid requiring PowerShell.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _run_checked(name: str, argv: list[str]) -> None:
    print(f"[pre-release] {name}")
    proc = subprocess.run(argv, check=False)
    if proc.returncode != 0:
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-fmt", action="store_true")
    parser.add_argument("--skip-clippy", action="store_true")
    parser.add_argument("--skip-nextest", action="store_true")
    parser.add_argument("--skip-icons", action="store_true")
    parser.add_argument("--skip-diff-check", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    py = sys.executable

    _run_checked(
        "ADR ID uniqueness",
        [py, str(repo_root / "tools/check_adr_numbers.py")],
    )
    _run_checked(
        "Workspace layering policy",
        [py, str(repo_root / "tools/check_layering.py")],
    )
    _run_checked(
        "Execution surface policy",
        [py, str(repo_root / "tools/check_execution_surface.py")],
    )
    _run_checked(
        "Stringly command parsing policy",
        [py, str(repo_root / "tools/check_stringly_command_parsing.py")],
    )

    if not args.skip_icons:
        icon_args = [py, str(repo_root / "tools/pre_release_icons.py")]
        if args.skip_diff_check:
            icon_args.append("--skip-diff-check")
        _run_checked("icons checks", icon_args)

    if not args.skip_fmt:
        _run_checked("cargo fmt --check", ["cargo", "fmt", "--all", "--", "--check"])

    if not args.skip_clippy:
        _run_checked(
            "cargo clippy (workspace, all targets)",
            ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
        )

    if not args.skip_nextest:
        if shutil.which("cargo-nextest") is None:
            print(
                "[pre-release] warning: cargo-nextest is not installed; falling back to cargo test --workspace"
            )
            _run_checked("cargo test --workspace", ["cargo", "test", "--workspace"])
        else:
            _run_checked("cargo nextest run", ["cargo", "nextest", "run"])

    print("[pre-release] done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

