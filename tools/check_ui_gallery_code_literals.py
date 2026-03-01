#!/usr/bin/env python3

from __future__ import annotations

import argparse
import pathlib
import re
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]


def iter_rust_files(base: pathlib.Path) -> list[pathlib.Path]:
    return sorted(p for p in base.rglob("*.rs") if p.is_file())


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Find UI Gallery doc code literals that can drift from previews."
    )
    parser.add_argument(
        "--ui-dir",
        default=str(ROOT / "apps" / "fret-ui-gallery" / "src" / "ui"),
        help="UI Gallery UI source directory (default: apps/fret-ui-gallery/src/ui).",
    )
    parser.add_argument(
        "--deny",
        action="store_true",
        help="Exit non-zero if any disallowed literals are found.",
    )
    parser.add_argument(
        "--only",
        action="append",
        default=[],
        help="Restrict scanning to this workspace-relative file or directory (repeatable).",
    )
    parser.add_argument(
        "--allow-file",
        action="append",
        default=[],
        help="Workspace-relative Rust file path to allow (repeatable).",
    )
    args = parser.parse_args()

    ui_dir = pathlib.Path(args.ui_dir).resolve()
    allow = {str((ROOT / p).resolve()) for p in args.allow_file}
    only: list[pathlib.Path] = [(ROOT / p).resolve() for p in args.only]

    # We intentionally focus on explicit multi-line literals (`r#"...`) because those are the
    # highest-drift examples. Short one-liners are tolerated during transition, but should
    # eventually move to snippet-backed regions as well.
    pattern = re.compile(r'\.code\(\s*"rust"\s*,\s*r#"', re.MULTILINE)

    matches: list[tuple[pathlib.Path, int, str]] = []

    scan_roots: list[pathlib.Path] = []
    if only:
        scan_roots = only
    else:
        scan_roots = [ui_dir]

    files: list[pathlib.Path] = []
    for root in scan_roots:
        if root.is_dir():
            files.extend(iter_rust_files(root))
        elif root.is_file() and root.suffix == ".rs":
            files.append(root)

    for path in sorted(set(files)):
        if str(path) in allow:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue

        for m in pattern.finditer(text):
            line = text.count("\n", 0, m.start()) + 1
            snippet = text.splitlines()[line - 1].strip()
            matches.append((path, line, snippet))

    if matches:
        for path, line, snippet in matches:
            rel = path.relative_to(ROOT)
            print(f"{rel}:{line}: {snippet}")
        if args.deny:
            return 1
        return 0

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
