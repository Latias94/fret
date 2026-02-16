#!/usr/bin/env python3
"""
Verify that vendored shadcn/ui v4 theme JSONs match the pinned upstream snapshot in repo-ref/.

Why:
- We vendor upstream registry theme JSONs under `ecosystem/fret-ui-shadcn/assets/...` for stable,
  offline builds.
- The "repo-ref" snapshot is our pinned reference for upstream parity workstreams.

This tool compares `cssVars.light` and `cssVars.dark` maps for each `theme-*.json` file.
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def _css_vars(data: dict) -> tuple[dict, dict]:
    css = data.get("cssVars", {})
    light = css.get("light", {}) if isinstance(css, dict) else {}
    dark = css.get("dark", {}) if isinstance(css, dict) else {}
    if not isinstance(light, dict):
        light = {}
    if not isinstance(dark, dict):
        dark = {}
    return light, dark


def _diff_maps(a: dict, b: dict) -> tuple[list[str], list[str], list[str]]:
    ak = set(a.keys())
    bk = set(b.keys())
    missing = sorted(ak - bk)
    extra = sorted(bk - ak)
    changed: list[str] = []
    for k in sorted(ak & bk):
        if a.get(k) != b.get(k):
            changed.append(k)
    return missing, extra, changed


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--upstream-dir",
        default="repo-ref/ui/apps/v4/public/r/styles/new-york-v4",
        help="Pinned upstream theme directory (repo-relative).",
    )
    parser.add_argument(
        "--vendored-dir",
        default="ecosystem/fret-ui-shadcn/assets/shadcn/themes/new-york-v4",
        help="Vendored theme directory (repo-relative).",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=50,
        help="Max per-file changed keys to print per scheme.",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    upstream_dir = (repo_root / args.upstream_dir).resolve()
    vendored_dir = (repo_root / args.vendored_dir).resolve()

    if not upstream_dir.exists():
        print(f"upstream dir not found: {upstream_dir}", file=sys.stderr)
        return 2
    if not vendored_dir.exists():
        print(f"vendored dir not found: {vendored_dir}", file=sys.stderr)
        return 2

    upstream_files = sorted(upstream_dir.glob("theme-*.json"))
    vendored_files = {p.name: p for p in sorted(vendored_dir.glob("theme-*.json"))}

    any_failed = False
    for up in upstream_files:
        vend = vendored_files.get(up.name)
        if vend is None:
            print(f"[FAIL] missing vendored: {up.name}", file=sys.stderr)
            any_failed = True
            continue

        up_data = _load_json(up)
        vend_data = _load_json(vend)

        up_light, up_dark = _css_vars(up_data)
        vend_light, vend_dark = _css_vars(vend_data)

        file_failed = False
        for scheme, a, b in (
            ("light", up_light, vend_light),
            ("dark", up_dark, vend_dark),
        ):
            missing, extra, changed = _diff_maps(a, b)
            if missing or extra or changed:
                any_failed = True
                file_failed = True
                print(
                    f"[FAIL] {up.name} ({scheme})",
                    file=sys.stderr,
                )
                if missing:
                    print("  missing keys:", file=sys.stderr)
                    for k in missing[: args.limit]:
                        print(f"    - {k}", file=sys.stderr)
                    if len(missing) > args.limit:
                        print(f"    ... ({len(missing) - args.limit} more)", file=sys.stderr)
                if extra:
                    print("  extra keys:", file=sys.stderr)
                    for k in extra[: args.limit]:
                        print(f"    - {k}", file=sys.stderr)
                    if len(extra) > args.limit:
                        print(f"    ... ({len(extra) - args.limit} more)", file=sys.stderr)
                if changed:
                    print("  changed values:", file=sys.stderr)
                    for k in changed[: args.limit]:
                        print(
                            f"    - {k}: upstream={a.get(k)!r} vendored={b.get(k)!r}",
                            file=sys.stderr,
                        )
                    if len(changed) > args.limit:
                        print(f"    ... ({len(changed) - args.limit} more)", file=sys.stderr)
        if not file_failed:
            print(f"[OK]   {up.name}")

    return 1 if any_failed else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
