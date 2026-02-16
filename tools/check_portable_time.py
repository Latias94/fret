#!/usr/bin/env python3
"""
Portable time-source checks (cross-platform).

Goal: prefer `fret_core::time::Instant` over `std::time::Instant` so code remains wasm-friendly.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Violation:
    name: str
    path: str
    line: int
    text: str


STD_INSTANT_QUALIFIED = re.compile(r"\bstd::time::Instant\b", flags=re.ASCII)
USE_STD_TIME_INSTANT = re.compile(r"^\s*use\s+std::time::Instant\b", flags=re.ASCII)

ALLOWLIST_PREFIXES: tuple[str, ...] = (
    # Tooling crates are currently native-only and may use std time directly.
    "crates/fret-diag/",
    "crates/fret-diag-protocol/",
    "crates/fret-diag-ws/",
)


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _normalize_repo_path(repo_root: Path, path: Path) -> str:
    try:
        rel = path.resolve().relative_to(repo_root)
        return rel.as_posix()
    except Exception:
        return path.as_posix()


def _scan_use_std_block(block_text: str) -> bool:
    # Inside `use std::{ ... }`, `time::` refers to `std::time::`.
    #
    # Examples:
    # - use std::{ time::{Duration, Instant}, ... };
    # - use std::{ time::Instant, ... };
    return bool(
        re.search(r"\btime\s*::\s*Instant\b", block_text, flags=re.ASCII)
        or re.search(r"\btime\s*::\s*\{[^}]*\bInstant\b", block_text, flags=re.ASCII | re.DOTALL)
    )


def _scan_use_std_time_block(block_text: str) -> bool:
    # Inside `use std::time::{ ... }`, `Instant` refers to `std::time::Instant`.
    return bool(re.search(r"\bInstant\b", block_text, flags=re.ASCII))


def _iter_rs_files(repo_root: Path) -> list[Path]:
    # Keep the guardrails focused on framework + ecosystem crates.
    # Apps often have native-only tooling that doesn't need to be wasm-portable.
    roots = ("crates", "ecosystem")
    out: list[Path] = []
    for root in roots:
        base = repo_root / root
        if not base.exists():
            continue
        out.extend([p for p in base.rglob("*.rs") if p.is_file()])
    return out


def main(argv: list[str]) -> int:
    _ = argv
    repo_root = _repo_root()
    violations: list[Violation] = []

    for path in _iter_rs_files(repo_root):
        rel = _normalize_repo_path(repo_root, path)
        if any(rel.startswith(prefix) for prefix in ALLOWLIST_PREFIXES):
            continue
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except Exception as e:
            print(f"error: failed to read {rel}: {e}", file=sys.stderr)
            return 1

        # 1) Explicit qualified usage in code: `std::time::Instant`.
        for idx, line in enumerate(lines, start=1):
            stripped = line.lstrip()
            if stripped.startswith("//"):
                continue
            if STD_INSTANT_QUALIFIED.search(line):
                violations.append(
                    Violation(
                        name="no-std-time-instant",
                        path=rel,
                        line=idx,
                        text=line.strip(),
                    )
                )

        # 2) `use std::{ ... }` blocks importing `time::Instant` or `time::{..., Instant ...}`.
        in_use_std = False
        std_block_start_line = 0
        std_block_parts: list[str] = []

        # 3) `use std::time::{ ... }` blocks importing `Instant`.
        in_use_std_time = False
        std_time_block_start_line = 0
        std_time_block_parts: list[str] = []

        for idx, line in enumerate(lines, start=1):
            if not in_use_std and not in_use_std_time:
                if USE_STD_TIME_INSTANT.search(line):
                    violations.append(
                        Violation(
                            name="no-use-std-time-instant",
                            path=rel,
                            line=idx,
                            text=line.strip(),
                        )
                    )
                    continue

                if re.match(r"^\s*use\s+std::time::\{", line):
                    in_use_std_time = True
                    std_time_block_start_line = idx
                    std_time_block_parts = [line]
                    if ";" in line:
                        in_use_std_time = False
                        block = "\n".join(std_time_block_parts)
                        if _scan_use_std_time_block(block):
                            violations.append(
                                Violation(
                                    name="no-use-std-time-brace-instant",
                                    path=rel,
                                    line=std_time_block_start_line,
                                    text="use std::time::{ ... Instant ... }",
                                )
                            )
                    continue

                if re.match(r"^\s*use\s+std::\s*\{", line):
                    in_use_std = True
                    std_block_start_line = idx
                    std_block_parts = [line]
                    if ";" in line:
                        in_use_std = False
                        block = "\n".join(std_block_parts)
                        if _scan_use_std_block(block):
                            violations.append(
                                Violation(
                                    name="no-use-std-time-instant-via-std-brace",
                                    path=rel,
                                    line=std_block_start_line,
                                    text="use std::{ ... time::{ ... Instant ... } ... }",
                                )
                            )
                    continue

            if in_use_std_time:
                std_time_block_parts.append(line)
                if ";" in line:
                    in_use_std_time = False
                    block = "\n".join(std_time_block_parts)
                    if _scan_use_std_time_block(block):
                        violations.append(
                            Violation(
                                name="no-use-std-time-brace-instant",
                                path=rel,
                                line=std_time_block_start_line,
                                text="use std::time::{ ... Instant ... }",
                            )
                        )
                continue

            if in_use_std:
                std_block_parts.append(line)
                if ";" in line:
                    in_use_std = False
                    block = "\n".join(std_block_parts)
                    if _scan_use_std_block(block):
                        violations.append(
                            Violation(
                                name="no-use-std-time-instant-via-std-brace",
                                path=rel,
                                line=std_block_start_line,
                                text="use std::{ ... time::{ ... Instant ... } ... }",
                            )
                        )
                continue

    if violations:
        for v in violations:
            print(
                f"Portable time violation ({v.name}): {v.path}:{v.line}: {v.text}",
                file=sys.stderr,
            )
        return 1

    print("Portable time check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
