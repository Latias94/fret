#!/usr/bin/env python3
"""
Cross-platform helper to run the builtin `ui-gallery` diag suite.

This is a Python alternative to `tools/diag-scripts/run-ui-gallery-suite.ps1`.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def _workspace_root() -> Path:
    return Path(__file__).resolve().parents[2]


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run `fretboard diag suite ui-gallery` with an optional launch command.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--dir", default="target/fret-diag", help="Diagnostics output directory.")
    ap.add_argument("--timeout-ms", type=int, default=60_000)
    ap.add_argument("--poll-ms", type=int, default=50)
    ap.add_argument(
        "--launch",
        action="store_true",
        help="Launch UI Gallery via `cargo run -p fret-ui-gallery --release`.",
    )
    ap.add_argument(
        "--no-release",
        action="store_true",
        help="When used with --launch, run UI Gallery without --release.",
    )

    args = ap.parse_args()

    workspace_root = _workspace_root()

    cmd: list[str] = [
        "cargo",
        "run",
        "-p",
        "fretboard",
        "--",
        "diag",
        "suite",
        "ui-gallery",
        "--dir",
        str(args.dir),
        "--timeout-ms",
        str(int(args.timeout_ms)),
        "--poll-ms",
        str(int(args.poll_ms)),
    ]

    if args.launch:
        cmd += ["--launch", "--", "cargo", "run", "-p", "fret-ui-gallery"]
        if not args.no_release:
            cmd += ["--release"]

    print("[suite] cmd:", " ".join(cmd))
    return int(subprocess.run(cmd, cwd=str(workspace_root)).returncode)


if __name__ == "__main__":
    raise SystemExit(main())

