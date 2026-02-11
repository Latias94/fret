#!/usr/bin/env python3
"""
Run the built-in shadcn conformance diag suite.

Cross-platform helper that wraps:
  `cargo run -p fretboard -- diag suite ui-gallery-shadcn-conformance`

If you pass `--launch`, this script uses fretboard's `--launch` support so the suite can
control the child process' environment consistently.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description="Run `fretboard diag suite ui-gallery-shadcn-conformance` (optionally launching UI Gallery).",
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

    args = ap.parse_args(argv)

    repo_root = _repo_root()

    cmd: list[str] = [
        "cargo",
        "run",
        "-p",
        "fretboard",
        "--",
        "diag",
        "suite",
        "ui-gallery-shadcn-conformance",
        "--dir",
        str(args.dir),
        "--timeout-ms",
        str(int(args.timeout_ms)),
        "--poll-ms",
        str(int(args.poll_ms)),
    ]

    if args.launch:
        launch_cmd: list[str] = ["cargo", "run", "-p", "fret-ui-gallery"]
        if not args.no_release:
            launch_cmd.append("--release")
        cmd += ["--launch", "--", *launch_cmd]

    print("[suite] cmd:", " ".join(cmd))
    return int(subprocess.run(cmd, cwd=str(repo_root)).returncode)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)

