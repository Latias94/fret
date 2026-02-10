#!/usr/bin/env python3
"""
Build all `apps/fret-demo` binaries with conservative parallelism.

Cross-platform replacement for `tools/windows/build-fret-demo-bins.ps1`.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.parent.resolve()


def main(argv: list[str]) -> int:
    if argv:
        print("This script does not accept positional args.", file=sys.stderr)
        return 2

    repo_root = _repo_root()

    if not os.environ.get("CARGO_TARGET_DIR"):
        os.environ["CARGO_TARGET_DIR"] = str(repo_root / "target")
    if not os.environ.get("CARGO_BUILD_JOBS"):
        os.environ["CARGO_BUILD_JOBS"] = "4"

    proc = subprocess.run(["cargo", "build", "-p", "fret-demo", "--bins"], cwd=str(repo_root))
    return proc.returncode


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
