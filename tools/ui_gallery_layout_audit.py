#!/usr/bin/env python3

from __future__ import annotations

import argparse
import pathlib
import re
import sys


MAX_W_RE = re.compile(r"\.max_w\s*\(\s*Px\s*\(\s*([0-9]+(?:\.[0-9]+)?)\s*\)")


def repo_root_from(script_path: pathlib.Path) -> pathlib.Path:
    return script_path.resolve().parent.parent


def iter_rs_files(root: pathlib.Path) -> list[pathlib.Path]:
    return sorted(p for p in root.rglob("*.rs") if p.is_file())


def extract_max_ws(text: str) -> list[str]:
    return sorted(set(MAX_W_RE.findall(text)), key=lambda s: float(s))


def fmt_widths(widths: list[str]) -> str:
    if not widths:
        return ""
    if len(widths) == 1:
        return widths[0]
    return ", ".join(widths)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audit UI Gallery max_w overrides (coarse text scan)."
    )
    parser.add_argument(
        "--root",
        type=pathlib.Path,
        default=None,
        help="Repo root; defaults to the script's parent directory.",
    )
    parser.add_argument(
        "--dir",
        type=pathlib.Path,
        default=pathlib.Path("apps/fret-ui-gallery/src/ui/pages"),
        help="Directory to scan, relative to --root.",
    )
    args = parser.parse_args()

    root = args.root or repo_root_from(pathlib.Path(__file__))
    scan_dir = (root / args.dir).resolve()

    if not scan_dir.exists():
        print(f"error: scan dir not found: {scan_dir}", file=sys.stderr)
        return 2

    rows: list[tuple[str, list[str]]] = []
    for path in iter_rs_files(scan_dir):
        text = path.read_text(encoding="utf-8")
        widths = extract_max_ws(text)
        if widths:
            rows.append((path.relative_to(root).as_posix(), widths))

    print("| File | `.max_w(Px(..))` values |")
    print("|---|---|")
    for rel, widths in rows:
        print(f"| `{rel}` | {fmt_widths(widths)} |")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

