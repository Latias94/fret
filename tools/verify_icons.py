#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


LUCIDE_RE = re.compile(r"\blucide\.([a-z0-9-]+)\b")
RADIX_RE = re.compile(r"\bradix\.([a-z0-9-]+)\b")


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def read_icon_stems(pack_dir: Path) -> set[str]:
    assets_dir = pack_dir / "assets" / "icons"
    if not assets_dir.exists():
        return set()
    return {p.stem for p in assets_dir.glob("*.svg")}


def iter_workspace_files(root: Path):
    include_ext = {".rs"}
    exclude_dirs = {
        ".git",
        "target",
        ".fret",
        "repo-ref",
        "goldens",
        "screenshots",
        "third_party",
        ".agents",
    }

    for top in ("apps", "crates", "ecosystem", "docs", "tools"):
        base = root / top
        if not base.exists():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            if any(part in exclude_dirs for part in path.parts):
                continue
            if path.suffix.lower() not in include_ext:
                continue
            yield path


def collect_vendor_refs(root: Path):
    refs: dict[str, dict[str, list[str]]] = {"lucide": {}, "radix": {}}

    for file in iter_workspace_files(root):
        try:
            text = file.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            text = file.read_text(encoding="utf-8", errors="ignore")

        rel = file.relative_to(root).as_posix()

        for match in LUCIDE_RE.finditer(text):
            stem = match.group(1)
            refs["lucide"].setdefault(stem, []).append(rel)

        for match in RADIX_RE.finditer(text):
            stem = match.group(1)
            refs["radix"].setdefault(stem, []).append(rel)

    return refs


def find_pack_dir(root: Path, pack: str) -> Path:
    pack_dir = root / "ecosystem" / f"fret-icons-{pack}"
    if pack_dir.exists():
        return pack_dir
    raise SystemExit(f"Unable to locate fret-icons-{pack} under ecosystem/.")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Verify that referenced vendor icon IDs exist in vendored pack assets."
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Treat missing icons as hard failure (default).",
    )
    args = parser.parse_args()

    root = repo_root()
    refs = collect_vendor_refs(root)

    missing_records: list[tuple[str, str, list[str]]] = []

    for pack in ("lucide", "radix"):
        pack_dir = find_pack_dir(root, pack)
        available = read_icon_stems(pack_dir)

        for stem, files in sorted(refs[pack].items()):
            if stem not in available:
                missing_records.append((pack, stem, sorted(set(files))))

    lucide_count = len(refs["lucide"])
    radix_count = len(refs["radix"])
    print(f"[verify-icons] referenced unique ids: lucide={lucide_count}, radix={radix_count}")

    if not missing_records:
        print("[verify-icons] OK: all referenced vendor icons are present in assets.")
        return 0

    print(f"[verify-icons] MISSING: {len(missing_records)} icon ids")
    for pack, stem, files in missing_records:
        sample = ", ".join(files[:3])
        extra = "" if len(files) <= 3 else f" (+{len(files) - 3} more)"
        print(f"  - {pack}.{stem} -> {sample}{extra}")

    if args.strict:
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())

