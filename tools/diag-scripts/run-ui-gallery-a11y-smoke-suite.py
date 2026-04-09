#!/usr/bin/env python3
"""
Run a small, cross-platform UI Gallery accessibility smoke suite.

This intentionally focuses on:
  - Dialog open/close via keyboard shortcuts
  - Stable a11y selectors (role + name) for gating
  - Focus restore on dismiss (Escape)

It wraps multiple invocations of:
  `cargo run -p fretboard-dev -- diag run <script> --env FRET_DIAG_REDACT_TEXT=0 ...`
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


SCRIPTS: list[str] = [
    "tools/diag-scripts/ui-gallery-command-palette-shortcut-primary.json",
    "tools/diag-scripts/ui-gallery-a11y-command-dialog-shortcut-primary.json",
    "tools/diag-scripts/ui-gallery-alert-dialog-escape-dismiss-focus-restore.json",
    "tools/diag-scripts/ui-gallery-sheet-escape-focus-restore.json",
]


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description="Run a small UI Gallery accessibility smoke suite (optionally launching UI Gallery).",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--dir", default="target/fret-diag", help="Diagnostics output directory.")
    ap.add_argument("--timeout-ms", type=int, default=90_000)
    ap.add_argument("--poll-ms", type=int, default=50)
    ap.add_argument(
        "--launch",
        action="store_true",
        help="Launch UI Gallery via `cargo run -p fret-ui-gallery --release` per script.",
    )
    ap.add_argument(
        "--no-release",
        action="store_true",
        help="When used with --launch, run UI Gallery without --release.",
    )
    args = ap.parse_args(argv)

    repo_root = _repo_root()

    launch_cmd: list[str] = ["cargo", "run", "-p", "fret-ui-gallery"]
    if not args.no_release:
        launch_cmd.append("--release")

    for script in SCRIPTS:
        cmd: list[str] = [
            "cargo",
            "run",
            "-p",
            "fretboard",
            "--",
            "diag",
            "run",
            script,
            "--env",
            "FRET_DIAG_REDACT_TEXT=0",
            "--dir",
            str(args.dir),
            "--timeout-ms",
            str(int(args.timeout_ms)),
            "--poll-ms",
            str(int(args.poll_ms)),
        ]

        if args.launch:
            cmd += ["--launch", "--", *launch_cmd]

        print("[a11y-smoke] cmd:", " ".join(cmd))
        rc = int(subprocess.run(cmd, cwd=str(repo_root)).returncode)
        if rc != 0:
            return rc

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)

