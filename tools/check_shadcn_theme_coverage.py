#!/usr/bin/env python3
"""
Check that shipped Fret themes cover shadcn/ui v4 (new-york-v4) CSS variable keys.

Source of truth:
  ecosystem/fret-ui-shadcn/assets/shadcn/themes/new-york-v4/theme-*.json

Targets (by default):
  themes/fret-default-dark.json
  themes/godot-default-dark.json
  themes/hardhacker-dark.json

Notes:
- Upstream registry theme JSON stores `radius` as a CSS var; in Fret we represent it as metrics
  (`metric.radius.*`). This checker treats `radius` as a required *metric* surface.
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


def _required_shadcn_keys(shadcn_dir: Path) -> tuple[set[str], bool]:
    required_colors: set[str] = set()
    requires_radius = False

    for path in sorted(shadcn_dir.glob("theme-*.json")):
        data = _load_json(path)
        css_vars = data.get("cssVars", {})
        for scheme in ("light", "dark"):
            vars_map = css_vars.get(scheme, {})
            if not isinstance(vars_map, dict):
                continue
            for key in vars_map.keys():
                if key == "radius":
                    requires_radius = True
                    continue
                required_colors.add(str(key))

    return required_colors, requires_radius


def _theme_surface(path: Path) -> tuple[set[str], set[str]]:
    data = _load_json(path)
    colors = data.get("colors", {})
    metrics = data.get("metrics", {})
    if not isinstance(colors, dict):
        colors = {}
    if not isinstance(metrics, dict):
        metrics = {}
    return {str(k) for k in colors.keys()}, {str(k) for k in metrics.keys()}


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--shadcn-dir",
        default="ecosystem/fret-ui-shadcn/assets/shadcn/themes/new-york-v4",
        help="Path (repo-relative) containing vendored shadcn v4 theme JSONs.",
    )
    parser.add_argument(
        "--theme",
        action="append",
        default=[
            "themes/fret-default-dark.json",
            "themes/godot-default-dark.json",
            "themes/hardhacker-dark.json",
        ],
        help="Theme JSON path to validate (repeatable, repo-relative).",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    shadcn_dir = (repo_root / args.shadcn_dir).resolve()
    if not shadcn_dir.exists():
        print(f"shadcn dir not found: {shadcn_dir}", file=sys.stderr)
        return 2

    required_colors, requires_radius = _required_shadcn_keys(shadcn_dir)
    if not required_colors:
        print(f"no shadcn keys found under: {shadcn_dir}", file=sys.stderr)
        return 2

    required_radius_metrics = {"metric.radius.sm", "metric.radius.md", "metric.radius.lg"}

    failed = False
    for raw in args.theme:
        theme_path = (repo_root / raw).resolve()
        if not theme_path.exists():
            print(f"theme not found: {raw}", file=sys.stderr)
            failed = True
            continue

        colors, metrics = _theme_surface(theme_path)
        missing_colors = sorted(required_colors - colors)
        missing_metrics: list[str] = []
        if requires_radius:
            missing_metrics = sorted(required_radius_metrics - metrics)

        if missing_colors or missing_metrics:
            failed = True
            print(f"[FAIL] {theme_path.relative_to(repo_root).as_posix()}", file=sys.stderr)
            if missing_colors:
                print("  missing shadcn color keys:", file=sys.stderr)
                for k in missing_colors:
                    print(f"    - {k}", file=sys.stderr)
            if missing_metrics:
                print("  missing radius metrics:", file=sys.stderr)
                for k in missing_metrics:
                    print(f"    - {k}", file=sys.stderr)
        else:
            print(f"[OK]   {theme_path.relative_to(repo_root).as_posix()}")

    return 1 if failed else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)

