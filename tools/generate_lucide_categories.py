#!/usr/bin/env python3
"""
Generate a Lucide category -> icons map from a local Lucide checkout.

Cross-platform replacement for `tools/generate_lucide_categories.ps1`.
"""

from __future__ import annotations

import argparse
import json
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
    parser.add_argument("--lucide-dir", default="repo-ref/lucide")
    parser.add_argument("--out-path", default="crates/fret-icons-lucide/categories.json")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    lucide_root = repo_root / args.lucide_dir
    lucide_categories = lucide_root / "categories"
    lucide_icon_meta = lucide_root / "icons"
    out_file = repo_root / args.out_path

    _assert_exists(lucide_categories)
    _assert_exists(lucide_icon_meta)

    known_categories = sorted(p.stem for p in lucide_categories.glob("*.json") if p.is_file())
    known_set = set(known_categories)

    category_to_icons: dict[str, set[str]] = {}
    for meta_path in lucide_icon_meta.glob("*.json"):
        if not meta_path.is_file():
            continue
        icon_name = meta_path.stem
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
        for cat in meta.get("categories", []):
            if cat not in known_set:
                raise RuntimeError(
                    f"Unknown Lucide category '{cat}' referenced by icon '{icon_name}' (update repo-ref/lucide/categories)"
                )
            category_to_icons.setdefault(cat, set()).add(icon_name)

    ordered: dict[str, list[str]] = {}
    for cat in known_categories:
        icons = sorted(category_to_icons.get(cat, set()))
        ordered[cat] = icons

    out_file.parent.mkdir(parents=True, exist_ok=True)
    out_file.write_text(json.dumps(ordered, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote Lucide category map: {out_file.as_posix()}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
