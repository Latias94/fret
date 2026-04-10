#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


ICON_ID_LITERAL_RE = re.compile(r'(?P<quote>["\'])(?P<pack>lucide|radix)\.(?P<stem>[a-z0-9-]+)(?P=quote)')


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def read_icon_stems(pack_dir: Path) -> set[str]:
    assets_dir = pack_dir / "assets" / "icons"
    if not assets_dir.exists():
        return set()
    return {p.stem for p in assets_dir.glob("*.svg")}


def read_vendor_aliases(pack_dir: Path) -> dict[str, str]:
    aliases_path = pack_dir / "vendor-aliases.txt"
    if not aliases_path.exists():
        return {}

    aliases: dict[str, str] = {}
    for line_no, raw in enumerate(aliases_path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue

        alias_name, sep, target_name = line.partition("=")
        if not sep:
            raise SystemExit(
                f"Invalid alias entry in {aliases_path}:{line_no}: expected <legacy-name>=<canonical-name>."
            )

        alias_name = alias_name.strip()
        target_name = target_name.strip()
        if not alias_name or not target_name:
            raise SystemExit(
                f"Invalid alias entry in {aliases_path}:{line_no}: empty alias or target name."
            )
        aliases[alias_name] = target_name

    return aliases


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

        for match in ICON_ID_LITERAL_RE.finditer(text):
            pack = match.group("pack")
            stem = match.group("stem")
            refs[pack].setdefault(stem, []).append(rel)

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
        assets = read_icon_stems(pack_dir)
        aliases = read_vendor_aliases(pack_dir)
        invalid_alias_targets = sorted(
            alias_name
            for alias_name, target_name in aliases.items()
            if target_name not in assets
        )
        if invalid_alias_targets:
            print(
                f"[verify-icons] INVALID: {pack} vendor aliases target missing canonical assets",
                file=sys.stderr,
            )
            for alias_name in invalid_alias_targets:
                print(
                    f"  - {pack}.{alias_name} -> {pack}.{aliases[alias_name]}",
                    file=sys.stderr,
                )
            return 2

        available = assets | set(aliases)

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
