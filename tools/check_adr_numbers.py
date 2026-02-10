#!/usr/bin/env python3
"""
Check ADR ID uniqueness under `docs/adr`.

Ported from `tools/check_adr_numbers.ps1` to avoid requiring PowerShell.

Usage:
    python3 tools/check_adr_numbers.py [--adr-dir docs/adr]
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from collections import defaultdict


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--adr-dir", default="docs/adr")
    args = parser.parse_args(argv)

    adr_dir = args.adr_dir
    if not os.path.isdir(adr_dir):
        print(f"ADR directory not found: {adr_dir}", file=sys.stderr)
        return 2

    groups: dict[int, list[str]] = defaultdict(list)
    pattern = re.compile(r"^(\d+)-")

    for name in os.listdir(adr_dir):
        if not name.endswith(".md"):
            continue
        m = pattern.match(name)
        if not m:
            continue
        groups[int(m.group(1))].append(name)

    dups = sorted((adr_id, sorted(names)) for adr_id, names in groups.items() if len(names) > 1)
    if dups:
        print(f"Duplicate ADR IDs found in {adr_dir}:", file=sys.stderr)
        for adr_id, names in dups:
            print(f"  {adr_id}: {', '.join(names)}", file=sys.stderr)
        return 1

    print("ADR ID check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

