#!/usr/bin/env python3
"""
Guard against stringly command parsing patterns in app/demo code.

Ported from `tools/check_stringly_command_parsing.ps1`.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Rule:
    name: str
    pattern: re.Pattern[str]


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _normalize_repo_path(repo_root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root).as_posix()
    except Exception:
        return path.as_posix()


def main(argv: list[str]) -> int:
    _ = argv

    repo_root = _repo_root()
    roots = (
        "apps/fret-examples/src",
        "apps/fretboard/src/scaffold",
        "apps/fret-ui-gallery/src",
    )

    rules = [
        Rule(
            name="no-command-strip-prefix-parsing",
            pattern=re.compile(r"command\.as_str\(\)\.strip_prefix\(", flags=re.ASCII),
        ),
        Rule(
            name="no-command-cmd-prefix-strip-prefix-parsing",
            pattern=re.compile(
                r"strip_prefix\((?:crate::commands::)?CMD_[A-Z0-9_]+_PREFIX\)",
                flags=re.ASCII,
            ),
        ),
    ]

    had_errors = False

    for root in roots:
        base = repo_root / root
        if not base.exists():
            continue
        for path in base.rglob("*.rs"):
            if not path.is_file():
                continue
            rel = _normalize_repo_path(repo_root, path)
            try:
                lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
            except Exception as e:
                print(f"error: failed to read {rel}: {e}", file=sys.stderr)
                had_errors = True
                continue

            for idx, line in enumerate(lines, start=1):
                for rule in rules:
                    if rule.pattern.search(line) is None:
                        continue
                    print(
                        f"Stringly command parsing violation ({rule.name}): {rel}:{idx}: {line.strip()}",
                        file=sys.stderr,
                    )
                    had_errors = True

    if had_errors:
        return 1
    print("Stringly command parsing check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

