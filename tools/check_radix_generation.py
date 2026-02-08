#!/usr/bin/env python3
"""
Radix icon generation gate.

Cross-platform replacement for `tools/check_radix_generation.ps1`.
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
    parser.add_argument("--skip-sync", action="store_true")
    parser.add_argument("--skip-verify", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    script = repo_root / "tools" / "check_icons_generation.py"

    cmd = [sys.executable, str(script), "--pack", "radix"]
    if args.skip_sync:
        cmd.append("--skip-sync")
    if args.skip_verify:
        cmd.append("--skip-verify")

    proc = subprocess.run(cmd, cwd=str(repo_root))
    return proc.returncode


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
