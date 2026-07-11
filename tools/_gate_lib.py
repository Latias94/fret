from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence


WORKSPACE_ROOT = Path(__file__).resolve().parent.parent

_RUST_RAW_STRING_OPEN = re.compile(r'(?:br|cr|r)(?P<hashes>#{0,255})"')
_RUST_CHAR_LITERAL = re.compile(
    r"'(?:[^'\\\r\n]|\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f_]{1,6}\}|[^\r\n]))'"
)


@dataclass(frozen=True)
class Hit:
    path: Path
    line_no: int
    line: str
    pattern: str


def strip_rust_comments(source: str) -> str:
    """Remove Rust comments without changing strings, newlines, or source length."""

    stripped: list[str] = []
    index = 0
    source_len = len(source)

    while index < source_len:
        char_match = _RUST_CHAR_LITERAL.match(source, index)
        if char_match is not None:
            stripped.append(char_match.group(0))
            index = char_match.end()
            continue

        raw_match = _RUST_RAW_STRING_OPEN.match(source, index)
        if raw_match is not None and (
            index == 0 or not (source[index - 1].isalnum() or source[index - 1] == "_")
        ):
            terminator = '"' + raw_match.group("hashes")
            end = source.find(terminator, raw_match.end())
            if end == -1:
                stripped.append(source[index:])
                break
            end += len(terminator)
            stripped.append(source[index:end])
            index = end
            continue

        if source[index] == '"':
            end = index + 1
            while end < source_len:
                if source[end] == "\\":
                    end = min(end + 2, source_len)
                    continue
                end += 1
                if source[end - 1] == '"':
                    break
            stripped.append(source[index:end])
            index = end
            continue

        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            if end == -1:
                end = source_len
            stripped.append("".join("\r" if char == "\r" else " " for char in source[index:end]))
            index = end
            continue

        if source.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < source_len and depth > 0:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            stripped.append(
                "".join(char if char in "\r\n" else " " for char in source[index:end])
            )
            index = end
            continue

        stripped.append(source[index])
        index += 1

    return "".join(stripped)


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
