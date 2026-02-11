#!/usr/bin/env python3
"""
Sync Lucide SVGs from a gpui-component-derived icon list.

Cross-platform replacement for `tools/sync_lucide_icons_from_gpui_component.ps1`.
"""

from __future__ import annotations

import argparse
import os
import shutil
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _assert_exists(path: Path) -> None:
    if not path.exists():
        raise RuntimeError(f"Path not found: {path}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--gpui-list", default="tools/gpui-icon-list.txt")
    parser.add_argument("--lucide-dir", default="repo-ref/lucide/icons")
    parser.add_argument("--dest-dir", default="crates/fret-icons-lucide/assets/icons")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    list_file = repo_root / args.gpui_list
    src = repo_root / args.lucide_dir
    dst = repo_root / args.dest_dir

    _assert_exists(list_file)
    _assert_exists(src)
    dst.mkdir(parents=True, exist_ok=True)

    items: list[str] = []
    for raw in list_file.read_text(encoding="utf-8", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        items.append(line)

    copied = 0
    missing: list[str] = []
    for name in items:
        from_path = src / name
        to_path = dst / name
        if from_path.exists():
            shutil.copyfile(from_path, to_path)
            copied += 1
        else:
            missing.append(name)

    print("Synced Lucide icons from gpui-component list")
    print(f"Copied: {copied}")
    print(f"Missing: {len(missing)}")
    if missing:
        print("Missing examples:")
        for m in missing[:20]:
            print(m)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
