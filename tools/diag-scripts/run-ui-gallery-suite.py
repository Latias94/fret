#!/usr/bin/env python3
"""
Run the UI Gallery diag suite (optionally starting the app).

Cross-platform replacement for `tools/diag-scripts/run-ui-gallery-suite.ps1`.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.parent.resolve()


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--diag-dir", default="target/fret-diag")
    parser.add_argument("--timeout-ms", type=int, default=60000)
    parser.add_argument("--poll-ms", type=int, default=50)
    parser.add_argument("--startup-delay-seconds", type=int, default=3)
    parser.add_argument("--start-app", action="store_true")
    parser.add_argument("--enable-inspect", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    os.environ["FRET_DIAG_DIR"] = args.diag_dir

    app_proc: subprocess.Popen[str] | None = None
    if args.start_app:
        print("Starting UI Gallery (diagnostics enabled)...")
        app_proc = subprocess.Popen(
            ["cargo", "run", "-p", "fret-ui-gallery"],
            cwd=str(repo_root),
        )
        print(f"UI Gallery pid={app_proc.pid}")
        time.sleep(max(0, int(args.startup_delay_seconds)))

        if args.enable_inspect:
            subprocess.run(
                ["cargo", "run", "-p", "fretboard", "--", "diag", "inspect", "on", "--dir", args.diag_dir],
                cwd=str(repo_root),
            )
    else:
        print("Assuming the app is already running (UI Gallery or another diagnostics-enabled app).")

    print("Running diag suite: ui-gallery")
    proc = subprocess.run(
        [
            "cargo",
            "run",
            "-p",
            "fretboard",
            "--",
            "diag",
            "suite",
            "ui-gallery",
            "--dir",
            args.diag_dir,
            "--timeout-ms",
            str(args.timeout_ms),
            "--poll-ms",
            str(args.poll_ms),
        ],
        cwd=str(repo_root),
    )

    # Match PowerShell behavior: do not auto-kill the started app.
    return proc.returncode


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
