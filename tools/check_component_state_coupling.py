#!/usr/bin/env python3
"""
Component state coupling checks.

Cross-platform replacement for `tools/check_component_state_coupling.ps1`.

It enforces two policies:
1) "Primitive" component modules must not directly depend on query/selector helpers, except in
   allowlisted `state/` modules.
2) If a crate depends on `fret-query` / `fret-selector`, the dependency must be optional and the
   crate must expose corresponding feature gates (and a `state` umbrella feature).
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Violation:
    rule: str
    path: str
    detail: str


def _resolve_repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _to_repo_rel(repo_root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except Exception:
        return str(path)


def _is_allowlisted(rel_path: str, allow_regexes: list[re.Pattern[str]]) -> bool:
    return any(p.search(rel_path) is not None for p in allow_regexes)


def _scan_source_files(
    *,
    repo_root: Path,
    source_roots: list[str],
    allow_regexes: list[re.Pattern[str]],
    code_rules: list[tuple[str, re.Pattern[str]]],
) -> list[Violation]:
    violations: list[Violation] = []

    roots = [repo_root / r for r in source_roots]
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*.rs"):
            if not path.is_file():
                continue
            rel = _to_repo_rel(repo_root, path)
            if _is_allowlisted(rel, allow_regexes):
                continue

            try:
                with path.open("r", encoding="utf-8", errors="replace") as f:
                    for line_no, line in enumerate(f, start=1):
                        for rule_name, pattern in code_rules:
                            if pattern.search(line) is not None:
                                violations.append(
                                    Violation(
                                        rule=rule_name,
                                        path=rel,
                                        detail=f"line {line_no}: {line.strip()}",
                                    )
                                )
            except OSError as exc:
                violations.append(Violation(rule="io-error", path=rel, detail=str(exc)))

    return violations


def _scan_manifests(
    *,
    repo_root: Path,
    manifest_paths: list[str],
    dep_policies: list[tuple[str, str]],
) -> list[Violation]:
    violations: list[Violation] = []

    for manifest in manifest_paths:
        path = repo_root / manifest
        if not path.is_file():
            continue

        rel = _to_repo_rel(repo_root, path)
        content = path.read_text(encoding="utf-8", errors="replace")

        for dep_name, feature in dep_policies:
            dep_re = re.compile(rf"(?m)^\\s*{re.escape(dep_name)}\\s*=\\s*(.+)$")
            matches = list(dep_re.finditer(content))
            if not matches:
                continue

            for m in matches:
                line = m.group(0).strip()
                if re.search(r"optional\\s*=\\s*true", line) is None:
                    violations.append(
                        Violation(
                            rule=f"{dep_name}-must-be-optional",
                            path=rel,
                            detail=line,
                        )
                    )

            if re.search(rf"(?m)^\\s*{re.escape(feature)}\\s*=", content) is None:
                violations.append(
                    Violation(
                        rule=f"missing-{feature}-feature",
                        path=rel,
                        detail=f"dependency `{dep_name}` exists but feature `{feature}` is missing",
                    )
                )

        has_state_selector = re.search(r"(?m)^\\s*state-selector\\s*=", content) is not None
        has_state_query = re.search(r"(?m)^\\s*state-query\\s*=", content) is not None
        if (has_state_selector or has_state_query) and re.search(r"(?m)^\\s*state\\s*=", content) is None:
            violations.append(
                Violation(
                    rule="missing-state-umbrella-feature",
                    path=rel,
                    detail='define state = ["state-selector", "state-query"]',
                )
            )

    return violations


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--repo-root",
        default="",
        help="Repo root (defaults to tools/..).",
    )
    args = parser.parse_args(argv)

    repo_root = Path(args.repo_root).resolve() if args.repo_root else _resolve_repo_root()

    source_roots = [
        "ecosystem/fret-ui-kit/src",
        "ecosystem/fret-ui-shadcn/src",
        "ecosystem/fret-ui-material3/src",
        "ecosystem/fret-imui/src",
    ]

    manifest_paths = [
        "ecosystem/fret-ui-kit/Cargo.toml",
        "ecosystem/fret-ui-shadcn/Cargo.toml",
        "ecosystem/fret-ui-material3/Cargo.toml",
        "ecosystem/fret-imui/Cargo.toml",
    ]

    allow_regexes = [
        re.compile(r"^ecosystem/fret-ui-kit/src/state(?:/|\\.rs)"),
        re.compile(r"^ecosystem/fret-ui-shadcn/src/state(?:/|\\.rs)"),
        re.compile(r"^ecosystem/fret-ui-material3/src/state(?:/|\\.rs)"),
        re.compile(r"^ecosystem/fret-imui/src/state(?:/|\\.rs)"),
    ]

    code_rules = [
        ("no-fret-query-import-in-primitives", re.compile(r"\\bfret_query::")),
        ("no-fret-selector-import-in-primitives", re.compile(r"\\bfret_selector::")),
        ("no-use-query-sugar-in-primitives", re.compile(r"\\.use_query(?:_async|_async_local)?\\s*\\(")),
        ("no-use-selector-sugar-in-primitives", re.compile(r"\\.use_selector(?:_keyed)?\\s*\\(")),
    ]

    dep_policies = [
        ("fret-query", "state-query"),
        ("fret-selector", "state-selector"),
    ]

    violations: list[Violation] = []
    violations.extend(
        _scan_source_files(
            repo_root=repo_root,
            source_roots=source_roots,
            allow_regexes=allow_regexes,
            code_rules=code_rules,
        )
    )
    violations.extend(
        _scan_manifests(
            repo_root=repo_root,
            manifest_paths=manifest_paths,
            dep_policies=dep_policies,
        )
    )

    if violations:
        for v in violations:
            print(f"Component state coupling violation ({v.rule}): {v.path}: {v.detail}", file=sys.stderr)
        return 1

    print("Component state coupling check passed.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
