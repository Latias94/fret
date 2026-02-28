#!/usr/bin/env python3
"""
Theme token drift checks (lightweight guardrails).

This tool is intentionally narrow: it flags newly-added *stringly-typed* reads of named literal
colors that should be accessed via typed keys (`ThemeNamedColorKey`).

Motivation:
  - Upstream parity (shadcn/ui) sometimes uses literal Tailwind classes like `text-white` or
    `bg-black/50`.
  - In Fret, those literals must stay as "named literal colors" (minimal compatibility surface),
    not be silently treated as semantic palette roles.
  - Using `theme.color_token("white")` / `theme.color_token("black")` works today, but it erodes
    the contract and makes reviews harder: use `theme.named_color(ThemeNamedColorKey::White)` /
    `ColorRef::Named(ThemeNamedColorKey::White)` instead.

Scope:
  - Scans Rust sources under selected roots (defaults to ecosystem + apps).
  - Does not attempt to validate theme JSON coverage (use `tools/check_theme_token_coverage.py`).
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


@dataclass(frozen=True)
class Match:
    rel: str
    line: int
    kind: str
    text: str


_THEME_READ_RE = re.compile(
    r"""\.(?P<method>color_token|color_required|color_by_key)\(\s*"(?P<key>white|black)"\s*\)""",
)

_COLORREF_TOKEN_RE = re.compile(
    r"""ColorRef::Token\s*\{\s*key\s*:\s*"(?P<key>white|black)"\s*,""",
)


def _scan_file(repo_root: Path, path: Path) -> list[Match]:
    rel = path.resolve().relative_to(repo_root.resolve()).as_posix()
    out: list[Match] = []
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        out.append(Match(rel=rel, line=0, kind="read_error", text=str(exc)))
        return out

    for i, line in enumerate(text.splitlines(), start=1):
        if _THEME_READ_RE.search(line):
            out.append(Match(rel=rel, line=i, kind="theme_named_color_read", text=line.strip()))
        if _COLORREF_TOKEN_RE.search(line):
            out.append(Match(rel=rel, line=i, kind="colorref_named_color_token", text=line.strip()))
    return out


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--path",
        action="append",
        default=["ecosystem", "apps"],
        help="Search root (repeatable, repo-relative). Defaults to ecosystem + apps.",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    matches: list[Match] = []

    for raw in args.path:
        root = (repo_root / raw).resolve()
        if not root.exists():
            continue
        for path in root.rglob("*.rs"):
            if not path.is_file():
                continue
            matches.extend(_scan_file(repo_root, path))

    if not matches:
        return 0

    print("Found theme token drift patterns (named colors should use ThemeNamedColorKey):", file=sys.stderr)
    for m in matches:
        if m.line > 0:
            print(f"{m.rel}:{m.line}: {m.kind}: {m.text}", file=sys.stderr)
        else:
            print(f"{m.rel}: {m.kind}: {m.text}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)

