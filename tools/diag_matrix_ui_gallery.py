#!/usr/bin/env python3
"""
Run the UI Gallery scripted regression matrix (uncached vs cached) with shell reuse enabled.

Cross-platform replacement for `tools/diag_matrix_ui_gallery.ps1`.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", default="target/fret-diag")
    parser.add_argument("--warmup-frames", type=int, default=5)
    parser.add_argument("--timeout-ms", type=int, default=180000)
    parser.add_argument("--poll-ms", type=int, default=50)
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)

    workspace_root = _repo_root()

    launch = ["cargo", "run", "-p", "fret-ui-gallery"]
    if args.release:
        launch.append("--release")

    cmd = [
        "cargo",
        "run",
        "-p",
        "fretboard",
        "--",
        "diag",
        "matrix",
        "ui-gallery",
        "--dir",
        args.out_dir,
        "--timeout-ms",
        str(args.timeout_ms),
        "--poll-ms",
        str(args.poll_ms),
        "--warmup-frames",
        str(args.warmup_frames),
        "--compare-ignore-bounds",
        "--compare-ignore-scene-fingerprint",
        "--check-view-cache-reuse-min",
        "1",
        "--check-overlay-synthesis-min",
        "1",
        "--env",
        "FRET_UI_GALLERY_VIEW_CACHE_SHELL=1",
        "--env",
        "FRET_DIAG_SEMANTICS=1",
        "--launch",
        "--",
        *launch,
    ]

    if args.json:
        cmd.append("--json")

    proc = subprocess.run(cmd, cwd=str(workspace_root))
    return proc.returncode


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
