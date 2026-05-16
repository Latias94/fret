#!/usr/bin/env python3
"""
Check shadcn recipe-internal slot markers do not leak into public UI mechanisms.

`fret-ui-shadcn.*` strings are reserved for recipe-owned child classification. They should be
declared as `*_SLOT` constants and attached through `AnyElement::component_slot(...)`, not exported
as diagnostics `test_id`s and not stored in shortcut `key_context`.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT_SENTINEL = "Cargo.toml"
SHADCN_SRC = Path("ecosystem/fret-ui-shadcn/src")
INTERNAL_SLOT_PREFIX = "fret-ui-shadcn."
CONST_RE = re.compile(
    r'^\s*const\s+([A-Z0-9_]+)\s*:\s*&str\s*=\s*"([^"]+)";'
)
FORBIDDEN_CALLS = (".test_id(", "attach_test_id(", ".key_context(")


@dataclass(frozen=True)
class Violation:
    path: Path
    line: int
    message: str

    def format(self) -> str:
        return f"{self.path.as_posix()}:{self.line}: {self.message}"


def find_repo_root(start: Path) -> Path:
    cur = start.resolve()
    for parent in [cur, *cur.parents]:
        if (parent / REPO_ROOT_SENTINEL).is_file():
            return parent
    raise SystemExit(
        f"error: failed to locate repo root (missing {REPO_ROOT_SENTINEL} in ancestors)"
    )


def collect_internal_slot_consts(text: str) -> set[str]:
    names: set[str] = set()
    for line in text.splitlines():
        match = CONST_RE.match(line)
        if match is None:
            continue
        name, value = match.groups()
        if value.startswith(INTERNAL_SLOT_PREFIX):
            names.add(name)
    return names


def lint_source(path: Path, rel_path: Path) -> list[Violation]:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    internal_consts = collect_internal_slot_consts(text)
    violations: list[Violation] = []

    for index, line in enumerate(lines, start=1):
        const_match = CONST_RE.match(line)
        if const_match is not None:
            name, value = const_match.groups()
            if value.startswith(INTERNAL_SLOT_PREFIX) and not name.endswith("_SLOT"):
                violations.append(
                    Violation(
                        rel_path,
                        index,
                        f"internal slot string constant `{name}` must end with `_SLOT`",
                    )
                )

        if not any(call in line for call in FORBIDDEN_CALLS):
            continue

        window = "\n".join(lines[index - 1 : min(len(lines), index + 4)])
        if INTERNAL_SLOT_PREFIX in window:
            violations.append(
                Violation(
                    rel_path,
                    index,
                    "internal slot literal must not be attached through test_id/key_context",
                )
            )
            continue

        matched_consts = sorted(name for name in internal_consts if name in window)
        if matched_consts:
            violations.append(
                Violation(
                    rel_path,
                    index,
                    "internal slot constant must not be attached through "
                    f"test_id/key_context: {', '.join(matched_consts)}",
                )
            )

    return violations


def lint_repo(repo_root: Path) -> list[Violation]:
    src_root = repo_root / SHADCN_SRC
    violations: list[Violation] = []
    for path in sorted(src_root.rglob("*.rs")):
        violations.extend(lint_source(path, path.relative_to(repo_root)))
    return violations


def main(argv: list[str]) -> int:
    repo_root = find_repo_root(Path(argv[1]) if len(argv) > 1 else Path.cwd())
    violations = lint_repo(repo_root)
    if violations:
        for violation in violations:
            print(f"error: {violation.format()}", file=sys.stderr)
        return 1

    print("ok: shadcn internal slot markers use component_slot-only surfaces.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
