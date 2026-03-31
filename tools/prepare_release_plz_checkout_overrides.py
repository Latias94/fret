#!/usr/bin/env python3
"""Install git attribute overrides needed by release-plz history checkouts."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

MANAGED_BEGIN = "# fret-release-plz checkout overrides: begin"
MANAGED_END = "# fret-release-plz checkout overrides: end"
OVERRIDE_PATHS = (
    "ecosystem/fret-ui-shadcn/src/slider.rs",
    "ecosystem/fret-ui-shadcn/src/context_menu.rs",
    "ecosystem/fret-ui-shadcn/src/dropdown_menu.rs",
    "ecosystem/fret-ui-shadcn/src/menubar.rs",
    "ecosystem/fret-ui-shadcn/src/tabs.rs",
)


def repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def git_path(repo: Path, relpath: str) -> Path:
    return Path(
        subprocess.check_output(
            ["git", "rev-parse", "--git-path", relpath],
            cwd=repo,
            text=True,
        ).strip()
    )


def strip_managed_block(text: str) -> str:
    lines = text.splitlines()
    cleaned: list[str] = []
    inside_block = False

    for line in lines:
        if line == MANAGED_BEGIN:
            inside_block = True
            continue
        if line == MANAGED_END:
            inside_block = False
            continue
        if not inside_block:
            cleaned.append(line)

    if inside_block:
        raise SystemExit("unterminated managed release-plz override block")

    return "\n".join(cleaned).strip("\n")


def managed_block() -> str:
    lines = [
        MANAGED_BEGIN,
        "# Keep historical CRLF blobs from appearing as dirty files when release-plz",
        "# checks out older commits to compute package diffs.",
    ]
    lines.extend(f"{path} -text" for path in OVERRIDE_PATHS)
    lines.append(MANAGED_END)
    return "\n".join(lines)


def desired_contents(current: str) -> str:
    base = strip_managed_block(current)
    block = managed_block()
    if base:
        return f"{base}\n\n{block}\n"
    return f"{block}\n"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="Exit non-zero if the managed override block is missing or outdated.",
    )
    args = parser.parse_args(argv)

    repo = repo_root()
    info_attributes = git_path(repo, "info/attributes")
    info_attributes.parent.mkdir(parents=True, exist_ok=True)
    current = info_attributes.read_text(encoding="utf-8") if info_attributes.exists() else ""
    desired = desired_contents(current)

    if args.check:
        if current == desired:
            print(f"[release-plz-overrides] ok: {info_attributes}")
            return 0
        print(f"[release-plz-overrides] missing or outdated: {info_attributes}", file=sys.stderr)
        return 1

    if current == desired:
        print(f"[release-plz-overrides] already up to date: {info_attributes}")
        return 0

    info_attributes.write_text(desired, encoding="utf-8")
    print(f"[release-plz-overrides] wrote {info_attributes}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
