from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence


WORKSPACE_ROOT = Path(__file__).resolve().parent.parent


@dataclass(frozen=True)
class Hit:
    path: Path
    line_no: int
    line: str
    pattern: str


def iter_files(
    roots: Sequence[Path],
    *,
    include_glob: str | None = None,
    max_depth: int | None = None,
) -> list[Path]:
    files: list[Path] = []
    for root in roots:
        if root.is_file():
            if include_glob is None or root.match(include_glob):
                files.append(root)
            continue

        if not root.is_dir():
            continue

        root_depth = len(root.parts)
        for path in root.rglob("*"):
            if max_depth is not None:
                depth = len(path.parts) - root_depth
                if depth > max_depth:
                    continue
            if not path.is_file():
                continue
            if include_glob is not None and not path.match(include_glob):
                continue
            files.append(path)
    return files


def scan_regexes(
    files: Iterable[Path],
    patterns: Sequence[str],
    *,
    encoding: str = "utf-8",
) -> list[Hit]:
    regexes = [re.compile(p) for p in patterns]
    hits: list[Hit] = []
    for path in files:
        try:
            text = path.read_text(encoding=encoding, errors="replace")
        except OSError:
            continue
        for idx, line in enumerate(text.splitlines(), start=1):
            for pattern, rx in zip(patterns, regexes, strict=True):
                if rx.search(line) is not None:
                    hits.append(
                        Hit(
                            path=path,
                            line_no=idx,
                            line=line.rstrip("\n"),
                            pattern=pattern,
                        )
                    )
    return hits


def print_hits(
    gate_name: str,
    hits: Sequence[Hit],
    *,
    max_lines: int = 40,
) -> None:
    print(f"[gate] {gate_name}")
    if not hits:
        print("[gate] ok")
        return

    print(f"[gate] FAIL: {len(hits)} match(es)")
    for hit in hits[:max_lines]:
        rel = hit.path.resolve().relative_to(WORKSPACE_ROOT)
        print(f"  - {rel}:{hit.line_no}: {hit.pattern}")
        print(f"      {hit.line.strip()}")
    if len(hits) > max_lines:
        print(f"  ... and {len(hits) - max_lines} more")


def fail(gate_name: str, message: str) -> "NoReturn":
    print(f"[gate] {gate_name}")
    print(f"[gate] FAIL: {message}")
    raise SystemExit(1)


def ok(gate_name: str) -> None:
    print(f"[gate] {gate_name}")
    print("[gate] ok")


def run_regex_gate(
    gate_name: str,
    *,
    roots: Sequence[Path],
    patterns: Sequence[str],
    include_glob: str | None = None,
    max_depth: int | None = None,
) -> None:
    roots = [r if r.is_absolute() else (WORKSPACE_ROOT / r) for r in roots]
    files = iter_files(roots, include_glob=include_glob, max_depth=max_depth)
    if not files:
        fail(gate_name, f"no files found under: {', '.join(str(r) for r in roots)}")
    hits = scan_regexes(files, patterns)
    print_hits(gate_name, hits)
    if hits:
        raise SystemExit(1)


def main(argv: Sequence[str] | None = None) -> int:
    _ = argv
    print("This module is a helper; run a concrete gate script instead.")
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

