#!/usr/bin/env python3
"""
Consumption profile compile gates (workspace-level).

Goal: keep Fret usable in a modular, Bevy-like way ("depend on only what you need") by preventing
accidental dependency growth and feature drift from breaking minimal build profiles.

This script is intentionally lightweight: it runs a small set of `cargo check` commands for
portable profiles that should not pull platform/render backends.

Intended usage:
  python3 tools/check_consumption_profiles.py
"""

from __future__ import annotations

import subprocess
import sys


def _run_checked(name: str, argv: list[str]) -> None:
    print(f"[profiles] {name}")
    proc = subprocess.run(argv, check=False)
    if proc.returncode != 0:
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")


def main(argv: list[str]) -> int:
    if argv:
        raise SystemExit("This script takes no arguments.")

    # Profile A — contracts-only (portable)
    _run_checked("contracts: fret-core", ["cargo", "check", "-p", "fret-core", "--locked"])
    _run_checked("contracts: fret-runtime", ["cargo", "check", "-p", "fret-runtime", "--locked"])
    _run_checked("contracts: fret-platform (contracts)", ["cargo", "check", "-p", "fret-platform", "--locked"])
    _run_checked("contracts: fret-render-core", ["cargo", "check", "-p", "fret-render-core", "--locked"])

    # Profile B — UI substrate (portable kernel)
    _run_checked("ui-substrate: fret-ui", ["cargo", "check", "-p", "fret-ui", "--locked"])

    # Profile C — advanced/manual assembly surface (portable facade only; no backends)
    _run_checked(
        "assembly: fret-framework (core+runtime+ui)",
        [
            "cargo",
            "check",
            "-p",
            "fret-framework",
            "--locked",
            "--no-default-features",
            "--features",
            "core,runtime,ui",
        ],
    )

    # Profile D — launcher facade
    _run_checked("launch: fret-launch", ["cargo", "check", "-p", "fret-launch", "--locked"])

    print("[profiles] done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
