#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shutil
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _pack_dir(root: Path, pack: str) -> Path:
    pack_dir = root / "ecosystem" / f"fret-icons-{pack}"
    if pack_dir.exists():
        return pack_dir
    raise SystemExit(f"Unable to locate icon pack directory for '{pack}' under ecosystem/.")


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


def _existing_dirs(paths: list[Path]) -> list[Path]:
    return [path for path in paths if path.exists()]


def _find_icon_source(icon_name: str, source_dirs: list[Path]) -> Path | None:
    for source_dir in source_dirs:
        candidate = source_dir / icon_name
        if candidate.exists():
            return candidate
    return None


def _sync_pack(
    name: str,
    source_dirs: list[Path],
    list_path: Path,
    dest_dir: Path,
    clean: bool,
) -> None:
    available_sources = _existing_dirs(source_dirs)
    if not available_sources:
        expected = "\n  - ".join(str(path) for path in source_dirs)
        raise SystemExit(
            f"No source directory found for pack '{name}'. Tried:\n  - {expected}"
        )

    if not list_path.exists():
        raise SystemExit(f"Path not found: {list_path}")

    dest_dir.mkdir(parents=True, exist_ok=True)

    items = _read_icon_list(list_path)
    copied = 0
    missing: list[str] = []

    for item in items:
        src = _find_icon_source(item, available_sources)
        if src is None:
            missing.append(item)
            continue

        dst = dest_dir / item
        if src.resolve() == dst.resolve():
            continue

        shutil.copy2(src, dst)
        copied += 1

    if missing:
        missing_lines = "\n  - ".join(missing)
        source_lines = "\n  - ".join(str(path) for path in available_sources)
        raise SystemExit(
            f"Missing {len(missing)} icon(s) for pack '{name}' while syncing {list_path}:"
            f"\n  - {missing_lines}\nSearched in:\n  - {source_lines}"
        )

    deleted = 0
    if clean:
        keep = set(items)
        for f in dest_dir.glob("*.svg"):
            if f.name not in keep:
                f.unlink()
                deleted += 1

    print(f"Synced {name} icons: {copied} files (cleaned {deleted})")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Sync vendored SVG icon packs from upstream sources into ecosystem/*/assets."
    )
    parser.add_argument("--pack", choices=["lucide", "radix", "all"], default="all")
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Delete extra *.svg files in destination not in icon-list.txt",
    )
    args = parser.parse_args()

    root = _repo_root()

    if args.pack in ("lucide", "all"):
        lucide_pack = _pack_dir(root, "lucide")
        lucide_assets = lucide_pack / "assets" / "icons"
        lucide_sources = [
            root / "third_party" / "lucide" / "icons",
            lucide_assets,
        ]
        _sync_pack(
            "lucide",
            lucide_sources,
            lucide_pack / "icon-list.txt",
            lucide_assets,
            args.clean,
        )

    if args.pack in ("radix", "all"):
        radix_pack = _pack_dir(root, "radix")
        radix_assets = radix_pack / "assets" / "icons"
        radix_sources = [
            radix_assets,
            root / "third_party" / "radix-icons" / "icons",
            root / "repo-ref" / "icons" / "packages" / "radix-icons" / "icons",
        ]
        _sync_pack(
            "radix",
            radix_sources,
            radix_pack / "icon-list.txt",
            radix_assets,
            args.clean,
        )


if __name__ == "__main__":
    main()
