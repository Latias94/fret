#!/usr/bin/env python3
"""Representative feature-matrix gate for `fret-fonts`."""

from __future__ import annotations

import shutil
import subprocess
import sys


def _run_checked(name: str, argv: list[str]) -> None:
    print(f"[fonts-matrix] {name}")
    proc = subprocess.run(argv, check=False)
    if proc.returncode != 0:
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")


def _cargo_test_args(*extra: str) -> list[str]:
    if shutil.which("cargo-nextest") is not None:
        return ["cargo", "nextest", "run", *extra]

    print(
        "[fonts-matrix] warning: cargo-nextest is not installed; falling back to cargo test"
    )
    return ["cargo", "test", *extra]


def main(argv: list[str]) -> int:
    if argv:
        raise SystemExit("This script takes no arguments.")

    _run_checked(
        "default feature tests",
        _cargo_test_args("-p", "fret-fonts", "--locked"),
    )
    _run_checked(
        "no-default-features compile",
        ["cargo", "check", "-p", "fret-fonts", "--locked", "--no-default-features"],
    )
    _run_checked(
        "cjk extension tests",
        _cargo_test_args("-p", "fret-fonts-cjk", "--locked"),
    )
    _run_checked(
        "emoji extension tests",
        _cargo_test_args("-p", "fret-fonts-emoji", "--locked"),
    )
    _run_checked(
        "package boundary",
        ["python3", "tools/check_fret_fonts_package_boundary.py"],
    )

    print("[fonts-matrix] done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
