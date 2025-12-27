#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _read_icon_list(list_path: Path) -> list[str]:
    items: list[str] = []
    for raw in list_path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if not line.endswith(".svg"):
            raise SystemExit(f"Invalid entry (expected *.svg): {line} in {list_path}")
        items.append(line)
    return items


def _sync_pack(name: str, src_dir: Path, list_path: Path, dest_dir: Path, clean: bool) -> None:
    if not src_dir.exists():
        raise SystemExit(f"Path not found: {src_dir}")
    if not list_path.exists():
        raise SystemExit(f"Path not found: {list_path}")

    dest_dir.mkdir(parents=True, exist_ok=True)

    items = _read_icon_list(list_path)
    copied = 0
    for item in items:
        src = src_dir / item
        if not src.exists():
            raise SystemExit(f"Missing icon: {src} (listed in {list_path})")
        dst = dest_dir / item
        shutil.copy2(src, dst)
        copied += 1

    deleted = 0
    if clean:
        keep = set(items)
        for f in dest_dir.glob("*.svg"):
            if f.name not in keep:
                f.unlink()
                deleted += 1

    print(f"Synced {name} icons: {copied} files (cleaned {deleted})")


def main() -> None:
    parser = argparse.ArgumentParser(description="Sync vendored SVG icon packs from repo-ref into crates/*/assets.")
    parser.add_argument("--pack", choices=["lucide", "radix", "all"], default="all")
    parser.add_argument("--clean", action="store_true", help="Delete extra *.svg files in dest not in icon-list.txt")
    args = parser.parse_args()

    root = _repo_root()

    if args.pack in ("lucide", "all"):
        _sync_pack(
            "lucide",
            root / "repo-ref" / "lucide" / "icons",
            root / "crates" / "fret-icons-lucide" / "icon-list.txt",
            root / "crates" / "fret-icons-lucide" / "assets" / "icons",
            args.clean,
        )

    if args.pack in ("radix", "all"):
        _sync_pack(
            "radix",
            root / "repo-ref" / "icons" / "packages" / "radix-icons" / "icons",
            root / "crates" / "fret-icons-radix" / "icon-list.txt",
            root / "crates" / "fret-icons-radix" / "assets" / "icons",
            args.clean,
        )


if __name__ == "__main__":
    main()

