#!/usr/bin/env python3
"""
Clean up unused Lucide icon SVGs.

Cross-platform replacement for `tools/cleanup_lucide_icons.ps1`.
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _assert_exists(path: Path) -> None:
    if not path.exists():
        raise RuntimeError(f"Path not found: {path}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--list-path", default="crates/fret-icons-lucide/icon-list.txt")
    parser.add_argument("--dest-dir", default="crates/fret-icons-lucide/assets/icons")
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    list_file = repo_root / args.list_path
    dest = repo_root / args.dest_dir

    _assert_exists(list_file)
    _assert_exists(dest)

    keep: set[str] = set()
    for raw in list_file.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if not line.endswith(".svg"):
            raise RuntimeError(f"Invalid entry (expected *.svg): {line}")
        keep.add(line)

    all_svgs = sorted(p for p in dest.glob("*.svg") if p.is_file())
    to_delete = [p for p in all_svgs if p.name not in keep]

    print("Lucide icon cleanup")
    print(f"List file: {args.list_path}")
    print(f"Total SVGs: {len(all_svgs)}")
    print(f"Keep SVGs: {len(keep)}")
    print(f"Delete SVGs: {len(to_delete)}")

    if not args.apply:
        print("Dry run (pass --apply to actually delete).")
        for p in to_delete[:40]:
            print(p.name)
        return 0

    deleted = 0
    for p in to_delete:
        p.unlink()
        deleted += 1

    print(f"Deleted: {deleted}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
