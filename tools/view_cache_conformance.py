#!/usr/bin/env python3
"""
Run view-cache conformance tests.

Cross-platform replacement for `tools/view_cache_conformance.ps1`.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _run(cmd: list[str], *, cwd: Path) -> int:
    proc = subprocess.run(cmd, cwd=str(cwd))
    return proc.returncode


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--quick", action="store_true")
    parser.add_argument("--full", action="store_true")
    args = parser.parse_args(argv)

    full = args.full or (not args.quick and not args.full)
    repo_root = _repo_root()

    code = _run(["cargo", "nextest", "run", "-p", "fret-ui", "view_cache"], cwd=repo_root)
    if code != 0:
        return code

    code = _run(["cargo", "nextest", "run", "-p", "fret-ui-kit", "window_overlays"], cwd=repo_root)
    if code != 0:
        return code

    if full:
        code = _run(["cargo", "nextest", "run", "-p", "fret-ui-shadcn", "tooltip", "hover_card", "dropdown_menu"], cwd=repo_root)
        if code != 0:
            return code

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
