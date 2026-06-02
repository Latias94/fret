#!/usr/bin/env python3
"""
Validate the workstream catalog indexes under docs/workstreams/.

This checker keeps the human-maintained README indexes honest:

- docs/workstreams/README.md
  - dedicated directory count
  - standalone markdown file count
  - Directory Index coverage for dedicated workstream directories
  - Directory Index status-prefix agreement with WORKSTREAM.json when a status prefix is present
- docs/workstreams/standalone/README.md
  - File Index coverage for standalone markdown files

Intended usage:
    python3 tools/check_workstream_catalog.py
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


DEDICATED_COUNT_RE = re.compile(r"^- Dedicated directories: (\d+)\s*$")
STANDALONE_COUNT_RE = re.compile(r"^- Standalone markdown files: (\d+)\b")
STANDALONE_BUCKET_COUNT_RE = re.compile(
    r"^- `docs/workstreams/standalone/README\.md` — .*?, (\d+) markdown docs(?:\b| )"
)
DIRECTORY_ENTRY_RE = re.compile(r"^- `docs/workstreams/([^`/]+)/`")
DIRECTORY_ENTRY_DETAIL_RE = re.compile(r"^- `docs/workstreams/([^`/]+)/` — .*?\(([^()]*)\)")
STANDALONE_FILE_ENTRY_RE = re.compile(r"^- `([^`/]+\.md)`")
STATUS_PREFIXES_BY_WORKSTREAM_STATUS = {
    "active": {"active"},
    "closed": {"closed"},
    "complete": {"complete"},
    "completed": {"complete", "completed"},
    "maintenance": {"maintenance"},
}
STATUS_PREFIX_WORDS = {
    prefix for prefixes in STATUS_PREFIXES_BY_WORKSTREAM_STATUS.values() for prefix in prefixes
}


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _extract_section_lines(readme_path: Path, heading: str) -> list[str]:
    lines = readme_path.read_text(encoding="utf-8").splitlines()
    start: int | None = None
    for index, line in enumerate(lines):
        if line.strip() == heading:
            start = index + 1
            break
    if start is None:
        raise ValueError(f"{readme_path}: missing section heading {heading!r}")

    end = len(lines)
    for index in range(start, len(lines)):
        if lines[index].startswith("## "):
            end = index
            break
    return lines[start:end]


def _parse_required_count(readme_path: Path, pattern: re.Pattern[str], label: str) -> int:
    for line in readme_path.read_text(encoding="utf-8").splitlines():
        match = pattern.match(line)
        if match is not None:
            return int(match.group(1))
    raise ValueError(f"{readme_path}: missing {label} count line")


def _find_duplicates(values: list[str]) -> list[str]:
    seen: set[str] = set()
    duplicates: list[str] = []
    for value in values:
        if value in seen and value not in duplicates:
            duplicates.append(value)
        seen.add(value)
    return duplicates


def _validate_dedicated_catalog(repo_root: Path, errors: list[str]) -> tuple[int, int]:
    catalog_path = repo_root / "docs" / "workstreams" / "README.md"
    workstreams_root = repo_root / "docs" / "workstreams"

    actual_dirs = sorted(
        path.name for path in workstreams_root.iterdir() if path.is_dir() and path.name != "standalone"
    )
    declared_dir_count = _parse_required_count(
        catalog_path, DEDICATED_COUNT_RE, "Dedicated directories"
    )
    declared_standalone_count = _parse_required_count(
        catalog_path, STANDALONE_COUNT_RE, "Standalone markdown files"
    )
    declared_standalone_bucket_count = _parse_required_count(
        catalog_path, STANDALONE_BUCKET_COUNT_RE, "Standalone Bucket README entry"
    )

    standalone_files = sorted(
        path.name
        for path in (workstreams_root / "standalone").glob("*.md")
        if path.name != "README.md"
    )

    directory_section = _extract_section_lines(catalog_path, "## Directory Index")
    listed_dirs = [
        match.group(1)
        for line in directory_section
        if (match := DIRECTORY_ENTRY_RE.match(line)) is not None
    ]

    if declared_dir_count != len(actual_dirs):
        errors.append(
            f"{catalog_path}: Dedicated directories says {declared_dir_count}, "
            f"but the repository contains {len(actual_dirs)} dedicated directories"
        )
    if declared_standalone_count != len(standalone_files):
        errors.append(
            f"{catalog_path}: Standalone markdown files says {declared_standalone_count}, "
            f"but the repository contains {len(standalone_files)} standalone markdown files"
        )
    if declared_standalone_bucket_count != len(standalone_files):
        errors.append(
            f"{catalog_path}: Standalone Bucket README entry says {declared_standalone_bucket_count}, "
            f"but the repository contains {len(standalone_files)} standalone markdown files"
        )

    duplicates = _find_duplicates(listed_dirs)
    if duplicates:
        errors.append(
            f"{catalog_path}: Directory Index contains duplicate dedicated entries: "
            + ", ".join(duplicates)
        )

    missing = [name for name in actual_dirs if name not in listed_dirs]
    extra = [name for name in listed_dirs if name not in actual_dirs]
    if missing:
        errors.append(
            f"{catalog_path}: Directory Index is missing dedicated directories: "
            + ", ".join(missing)
        )
    if extra:
        errors.append(
            f"{catalog_path}: Directory Index lists non-existent dedicated directories: "
            + ", ".join(extra)
        )

    _validate_dedicated_entry_statuses(catalog_path, workstreams_root, directory_section, errors)

    return len(actual_dirs), len(standalone_files)


def _workstream_status(workstream_json_path: Path) -> str:
    data = json.loads(workstream_json_path.read_text(encoding="utf-8"))
    status = data.get("status")
    if not isinstance(status, str) or not status:
        raise ValueError(f"{workstream_json_path}: missing string status")
    return status.lower()


def _status_prefix(parenthetical: str) -> str | None:
    words = parenthetical.split(maxsplit=1)
    if not words:
        return None
    first_word = words[0].strip(".,;:").lower()
    if first_word in STATUS_PREFIX_WORDS:
        return first_word
    return None


def _validate_dedicated_entry_statuses(
    catalog_path: Path, workstreams_root: Path, directory_section: list[str], errors: list[str]
) -> None:
    for line in directory_section:
        match = DIRECTORY_ENTRY_DETAIL_RE.match(line)
        if match is None:
            continue

        slug, parenthetical = match.groups()
        workstream_json_path = workstreams_root / slug / "WORKSTREAM.json"
        if not workstream_json_path.exists():
            continue

        declared_prefix = _status_prefix(parenthetical)
        if declared_prefix is None:
            continue

        actual_status = _workstream_status(workstream_json_path)
        expected_prefixes = STATUS_PREFIXES_BY_WORKSTREAM_STATUS.get(actual_status)
        if expected_prefixes is None:
            continue

        if declared_prefix not in expected_prefixes:
            expected = "/".join(sorted(expected_prefixes))
            errors.append(
                f"{catalog_path}: Directory Index entry {slug!r} starts with "
                f"{declared_prefix!r}, but {workstream_json_path} status is "
                f"{actual_status!r}; expected prefix {expected!r}"
            )


def _validate_standalone_catalog(repo_root: Path, errors: list[str]) -> int:
    catalog_path = repo_root / "docs" / "workstreams" / "standalone" / "README.md"
    standalone_root = catalog_path.parent

    actual_files = sorted(
        path.name for path in standalone_root.glob("*.md") if path.name != "README.md"
    )
    file_index_section = _extract_section_lines(catalog_path, "## File Index")
    listed_files = [
        match.group(1)
        for line in file_index_section
        if (match := STANDALONE_FILE_ENTRY_RE.match(line)) is not None
    ]

    duplicates = _find_duplicates(listed_files)
    if duplicates:
        errors.append(
            f"{catalog_path}: File Index contains duplicate file entries: "
            + ", ".join(duplicates)
        )

    missing = [name for name in actual_files if name not in listed_files]
    extra = [name for name in listed_files if name not in actual_files]
    if missing:
        errors.append(
            f"{catalog_path}: File Index is missing standalone markdown files: "
            + ", ".join(missing)
        )
    if extra:
        errors.append(
            f"{catalog_path}: File Index lists non-existent standalone markdown files: "
            + ", ".join(extra)
        )

    return len(actual_files)


def main() -> int:
    repo_root = _repo_root()
    errors: list[str] = []

    try:
        dedicated_count, standalone_count = _validate_dedicated_catalog(repo_root, errors)
        _validate_standalone_catalog(repo_root, errors)
    except OSError as exc:
        print(f"error: failed to read workstream catalog files: {exc}", file=sys.stderr)
        return 2
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1

    print(
        "Validated workstream catalog indexes: "
        f"{dedicated_count} dedicated directories, {standalone_count} standalone markdown files."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
