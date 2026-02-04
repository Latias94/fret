#!/usr/bin/env python3
"""
Fret design-system style generator (no external dependencies).

Usage:
  python scripts/stylegen.py --list
  python scripts/stylegen.py --suggest "compact dense editor"
  python scripts/stylegen.py --style editor-compact > theme_overrides.json
  python scripts/stylegen.py --style editor-hud --out theme_overrides.json
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple


@dataclass(frozen=True)
class StyleEntry:
    style_id: str
    label: str
    keywords: List[str]
    baseline: Dict[str, str]
    overrides: Dict[str, Any]


def _catalog_path() -> str:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    return os.path.join(script_dir, "..", "references", "style_catalog.json")


def load_catalog() -> List[StyleEntry]:
    path = _catalog_path()
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)

    out: List[StyleEntry] = []
    for raw in data.get("styles", []):
        out.append(
            StyleEntry(
                style_id=str(raw["id"]),
                label=str(raw.get("label", raw["id"])),
                keywords=[str(k) for k in raw.get("keywords", [])],
                baseline=dict(raw.get("baseline", {})),
                overrides=dict(raw.get("overrides", {})),
            )
        )
    return out


def score_style(query: str, style: StyleEntry) -> int:
    q = query.strip().lower()
    if not q:
        return 0
    score = 0

    # Direct id/label matches get a boost.
    if style.style_id.lower() in q:
        score += 5
    if style.label.lower() in q:
        score += 3

    # Keyword matches: exact substring count.
    for kw in style.keywords:
        kw_l = kw.lower()
        if kw_l and kw_l in q:
            score += 2
    return score


def suggest_styles(
    styles: List[StyleEntry], query: str, limit: int
) -> List[Tuple[int, StyleEntry]]:
    scored = [(score_style(query, s), s) for s in styles]
    scored = [x for x in scored if x[0] > 0]
    scored.sort(key=lambda x: (-x[0], x[1].style_id))
    return scored[:limit]


def find_style(styles: List[StyleEntry], style_id: str) -> Optional[StyleEntry]:
    for s in styles:
        if s.style_id == style_id:
            return s
    return None


def print_baseline(style: StyleEntry) -> None:
    baseline = style.baseline
    preset = baseline.get("preset", "shadcn-new-york-v4")
    base = baseline.get("base", "zinc")
    scheme = baseline.get("scheme", "dark")
    print(
        f"[baseline] preset={preset} base={base} scheme={scheme}",
        file=sys.stderr,
    )
    print(
        "           apply_shadcn_new_york_v4(app, ShadcnBaseColor::<Base>, ShadcnColorScheme::<Scheme>)",
        file=sys.stderr,
    )


def write_json(path: str, obj: Dict[str, Any]) -> None:
    os.makedirs(os.path.dirname(os.path.abspath(path)) or ".", exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(obj, f, ensure_ascii=False, indent=2)
        f.write("\n")


def main(argv: List[str]) -> int:
    parser = argparse.ArgumentParser(description="Fret design-system style generator")
    parser.add_argument("--list", action="store_true", help="List available styles")
    parser.add_argument(
        "--suggest",
        metavar="QUERY",
        help="Suggest styles for a free-form query (e.g. 'compact editor')",
    )
    parser.add_argument("--style", metavar="ID", help="Emit ThemeConfig overrides for style ID")
    parser.add_argument(
        "--out",
        metavar="PATH",
        help="Write overrides JSON to PATH (prints a summary; does not print JSON)",
    )
    parser.add_argument(
        "--top",
        type=int,
        default=3,
        help="Max suggestions to print for --suggest (default: 3)",
    )
    args = parser.parse_args(argv)

    styles = load_catalog()

    if args.list:
        for s in styles:
            print(f"- {s.style_id}: {s.label}")
        return 0

    if args.suggest is not None:
        hits = suggest_styles(styles, args.suggest, max(1, args.top))
        if not hits:
            print("No matches. Try keywords like: compact, comfortable, soft, hud, glass.", file=sys.stderr)
            return 2
        for score, s in hits:
            print(f"- {s.style_id} ({score}): {s.label}")
        return 0

    if args.style is not None:
        s = find_style(styles, args.style)
        if s is None:
            print(f"Unknown style id: {args.style}", file=sys.stderr)
            print("Use --list to see available styles.", file=sys.stderr)
            return 2

        print_baseline(s)

        if args.out:
            write_json(args.out, s.overrides)
            print(f"[ok] wrote {args.out}", file=sys.stderr)
            return 0

        json.dump(s.overrides, sys.stdout, ensure_ascii=False, indent=2)
        sys.stdout.write("\n")
        return 0

    parser.print_help()
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
