#!/usr/bin/env python3
"""
Sync Lucide SVGs from a local Lucide checkout into this repo by category.

Cross-platform replacement for `tools/sync_lucide_icons_by_category.ps1`.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _assert_exists(path: Path) -> None:
    if not path.exists():
        raise RuntimeError(f"Path not found: {path}")


def _split_categories(values: list[str]) -> list[str]:
    out: list[str] = []
    for v in values:
        for part in v.split(","):
            p = part.strip()
            if p:
                out.append(p)
    return out


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--category", action="append", required=True, help="Category name (repeatable or comma-separated).")
    parser.add_argument("--lucide-dir", default="repo-ref/lucide")
    parser.add_argument("--category-map", default="crates/fret-icons-lucide/categories.json")
    parser.add_argument("--dest-dir", default="crates/fret-icons-lucide/assets/icons")
    args = parser.parse_args(argv)

    categories = _split_categories(args.category)

    repo_root = _repo_root()
    lucide_root = repo_root / args.lucide_dir
    lucide_svgs = lucide_root / "icons"
    map_file = repo_root / args.category_map
    dest = repo_root / args.dest_dir

    _assert_exists(lucide_svgs)
    _assert_exists(map_file)
    dest.mkdir(parents=True, exist_ok=True)

    mapping: dict[str, list[str]] = json.loads(map_file.read_text(encoding="utf-8"))

    copied = 0
    missing = 0
    for cat in categories:
        if cat not in mapping:
            raise RuntimeError(f"Category '{cat}' not found in {map_file.as_posix()}")
        for icon in mapping[cat]:
            src = lucide_svgs / f"{icon}.svg"
            dst = dest / f"{icon}.svg"
            if src.exists():
                shutil.copyfile(src, dst)
                copied += 1
            else:
                missing += 1

    print(f"Synced Lucide categories: {', '.join(categories)}")
    print(f"Copied: {copied} Missing SVGs: {missing}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
