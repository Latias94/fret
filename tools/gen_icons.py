#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def run_script(script: Path) -> None:
    cmd = [sys.executable, str(script)]
    proc = subprocess.run(cmd, check=False)
    if proc.returncode != 0:
        raise SystemExit(proc.returncode)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate icon metadata/constants for one or more packs."
    )
    parser.add_argument("--pack", choices=["lucide", "radix", "all"], default="all")
    args = parser.parse_args()

    root = repo_root()

    if args.pack in ("lucide", "all"):
        run_script(root / "tools" / "gen_lucide.py")

    if args.pack in ("radix", "all"):
        run_script(root / "tools" / "gen_radix.py")


if __name__ == "__main__":
    main()

