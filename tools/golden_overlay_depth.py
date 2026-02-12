#!/usr/bin/env python3
"""
Golden overlay "depth" / menu-height gate coverage for shadcn-web.

This is a cross-platform replacement for `tools/golden_overlay_depth.ps1`.

It focuses on `*.open.json` keys (optionally normalizing `.open` suffix) and
heuristically associates them with "menu/listbox height" gates by scanning
non-smoke Rust tests under `ecosystem/fret-ui-shadcn/tests`.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path


def _resolve_repo_root(start_dir: Path) -> Path:
    current = start_dir.resolve()
    while True:
        if (current / "Cargo.toml").is_file() and (current / "goldens").is_dir():
            return current
        parent = current.parent
        if parent == current:
            break
        current = parent
    raise RuntimeError(
        f"Unable to locate repo root from {start_dir} (expected Cargo.toml + goldens/)."
    )


def _git_ls_files(repo_root: Path, rel_dir: str) -> list[str]:
    proc = subprocess.run(
        ["git", "-C", str(repo_root), "ls-files", "--", rel_dir],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        msg = proc.stderr.strip() or "git ls-files failed"
        raise RuntimeError(f"{msg} (RepoRoot={repo_root}, Dir={rel_dir})")
    return [line for line in proc.stdout.splitlines() if line.strip()]


def _tracked_golden_files(repo_root: Path, golden_dir: Path) -> list[str]:
    repo_root_abs = repo_root.resolve()
    golden_dir_abs = golden_dir.resolve()
    try:
        rel = golden_dir_abs.relative_to(repo_root_abs)
    except ValueError as exc:
        raise RuntimeError(
            f"GoldenDir is not under RepoRoot (RepoRoot={repo_root_abs}, GoldenDir={golden_dir_abs})"
        ) from exc
    return _git_ls_files(repo_root_abs, rel.as_posix())


def _extract_open_keys(paths: list[str], *, normalize_open_suffix: bool) -> list[str]:
    keys: set[str] = set()
    for p in paths:
        if not p.endswith(".open.json"):
            continue
        name = Path(p).stem  # foo.open.json -> foo.open
        if normalize_open_suffix:
            name = re.sub(r"\.open$", "", name)
        keys.add(name)
    return sorted(keys)


def _detect_gate_kinds(test_name: str, *, gate_kind: str) -> list[str]:
    kinds: list[str] = []
    if gate_kind == "menu-height":
        if re.search(r"panel_size", test_name):
            kinds.append("panel-size")
        if re.search(r"panel_height", test_name):
            kinds.append("panel-height")
        if re.search(r"panel_rect", test_name):
            kinds.append("panel-rect")
        if re.search(r"menu_item_height", test_name):
            kinds.append("menu-item-height")
        if re.search(r"listbox_height", test_name):
            kinds.append("listbox-height")
        if re.search(r"option_height", test_name):
            kinds.append("listbox-option-height")
        if re.search(r"content_insets", test_name):
            kinds.append("content-insets")
        if re.search(r"wheel_scroll_matches_web_scrolled", test_name):
            kinds.append("wheel-scroll")
        if re.search(r"viewport_height_matches", test_name):
            kinds.append("viewport-height")
        return kinds

    if re.search(r"panel_size", test_name):
        kinds.append("panel-size")
    if re.search(r"height", test_name):
        kinds.append("height")
    if re.search(r"insets", test_name):
        kinds.append("insets")
    return kinds


_TEST_BLOCK_SPLIT_RE = re.compile(r"(?m)^(?=#\[test\])")
_TEST_NAME_RE = re.compile(r"(?m)^fn\s+([A-Za-z0-9_]+)\s*\(")
_KEY_IN_QUOTES_RE = re.compile(r'"([a-z0-9][a-z0-9_.-]{1,160})"')


def _extract_keys_from_block(block: str, *, key_set: set[str]) -> set[str]:
    hits: set[str] = set()
    for m in _KEY_IN_QUOTES_RE.finditer(block):
        s = m.group(1)
        if s in key_set:
            hits.add(s)
    return hits


def _has_menu_height_gate(gate_kinds: set[str]) -> bool:
    return any(
        k in gate_kinds
        for k in (
            "wheel-scroll",
            "panel-size",
            "panel-height",
            "panel-rect",
            "viewport-height",
            "menu-item-height",
            "listbox-height",
            "listbox-option-height",
            "content-insets",
        )
    )


def _split_prefix(key: str, split_pattern: str) -> str:
    parts = re.split(split_pattern, key)
    return parts[0] if parts and parts[0] else key


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--style", default="v4/new-york-v4")
    parser.add_argument("--repo-root", default="", help="Repo root (auto-detected if empty).")
    parser.add_argument("--tracked-only", action="store_true")
    parser.add_argument(
        "--normalize-open-suffix",
        action=argparse.BooleanOptionalAction,
        default=True,
        help='Normalize open keys by stripping a trailing ".open".',
    )
    parser.add_argument("--gate-kind", default="menu-height")
    parser.add_argument(
        "--overlay-family",
        choices=["menu-list", "all-overlays"],
        default="menu-list",
    )
    parser.add_argument("--constrained-viewport-token", default="vp375x240")
    parser.add_argument("--filter-key-regex", default="")
    parser.add_argument("--exclude-key-regex", default="(focus-first|highlight-first|then-hover)")
    parser.add_argument("--dump-key", default="")
    parser.add_argument("--top-missing", type=int, default=50)
    parser.add_argument("--group-missing-by-prefix", action="store_true")
    parser.add_argument("--top-groups", type=int, default=25)
    parser.add_argument("--group-split-pattern", default=r"[\\.-]")
    parser.add_argument("--as-markdown", action="store_true")

    args = parser.parse_args(argv)

    filter_key_regex = args.filter_key_regex.strip()
    if not filter_key_regex:
        if args.overlay_family == "menu-list":
            filter_key_regex = r"(menu|dropdown|select|combobox|command)"
        else:
            filter_key_regex = r"(menu|dropdown|select|combobox|command|popover|tooltip|hover-card|dialog|alert-dialog|sheet|drawer|calendar|date-picker)"

    repo_root = Path(args.repo_root) if args.repo_root else _resolve_repo_root(Path(__file__).parent.parent)
    golden_dir = repo_root / "goldens" / "shadcn-web" / args.style
    if not golden_dir.is_dir():
        raise RuntimeError(f"Missing golden directory: {golden_dir}")

    test_dir = repo_root / "ecosystem" / "fret-ui-shadcn" / "tests"
    if not test_dir.is_dir():
        raise RuntimeError(f"Missing test directory: {test_dir}")

    if args.tracked_only:
        golden_files = _tracked_golden_files(repo_root, golden_dir)
    else:
        golden_files = [str(p) for p in golden_dir.glob("*.json") if p.is_file()]

    open_keys = _extract_open_keys(golden_files, normalize_open_suffix=args.normalize_open_suffix)
    key_set = set(open_keys)

    gate_kinds_by_key: dict[str, set[str]] = {k: set() for k in open_keys}

    test_files = sorted(p for p in test_dir.glob("*.rs") if p.is_file() and not p.name.endswith("_goldens_smoke.rs"))
    for path in test_files:
        text = path.read_text(encoding="utf-8")
        parts = _TEST_BLOCK_SPLIT_RE.split(text)
        for part in parts:
            if not re.search(r"(?m)^#\[test\]", part):
                continue
            m = _TEST_NAME_RE.search(part)
            if not m:
                continue
            test_name = m.group(1)
            kinds = _detect_gate_kinds(test_name, gate_kind=args.gate_kind)
            if not kinds:
                continue
            keys = _extract_keys_from_block(part, key_set=key_set)
            for key in keys:
                gate_kinds_by_key[key].update(kinds)

    if args.dump_key.strip():
        dump_key = args.dump_key.strip()
        if dump_key not in gate_kinds_by_key:
            raise RuntimeError(f"DumpKey not found in open keys: {dump_key}")
        kinds = ", ".join(sorted(gate_kinds_by_key[dump_key]))
        if args.as_markdown:
            print(f"- `{dump_key}` gate kinds: {kinds}")
        else:
            print(f"{dump_key} gate kinds: {kinds}")
        return 0

    candidates = [
        k
        for k in open_keys
        if re.search(filter_key_regex, k)
        and (args.constrained_viewport_token in k)
    ]

    exclude_key_regex = args.exclude_key_regex.strip()
    if exclude_key_regex:
        candidates = [k for k in candidates if not re.search(exclude_key_regex, k)]

    missing = [k for k in candidates if not _has_menu_height_gate(gate_kinds_by_key[k])]

    if args.as_markdown:
        tracked_note = " (tracked-only)" if args.tracked_only else ""
        print(f"- shadcn-web open overlay depth{tracked_note}: {len(open_keys)} open keys")
        print(f"  - constrained token: `{args.constrained_viewport_token}`")
        print(f"  - filter: `{filter_key_regex}`")
        if exclude_key_regex:
            print(f"  - exclude: `{exclude_key_regex}`")
        print(f"  - candidates: {len(candidates)}")
        print(f"  - missing menu/listbox height gates: {len(missing)}")
    else:
        print(f"Golden overlay depth (shadcn-web/{args.style})")
        print(f"  RepoRoot:  {repo_root}")
        print(f"  GoldenDir: {golden_dir}")
        print(f"  TestsDir:  {test_dir}")
        print(f"  Tracked:   {'yes' if args.tracked_only else 'no'}")
        print(f"  Open keys: {len(open_keys)}")
        print(f"  Constrained token: {args.constrained_viewport_token}")
        print(f"  Filter regex: {filter_key_regex}")
        if exclude_key_regex:
            print(f"  Exclude regex: {exclude_key_regex}")
        print(f"  Candidates: {len(candidates)}")
        print(f"  Missing menu/listbox height gates: {len(missing)}")

    if args.group_missing_by_prefix:
        prefixes = [_split_prefix(k, args.group_split_pattern) for k in missing]
        groups = Counter(prefixes).most_common(args.top_groups)
        if args.as_markdown:
            print(f"\n- Missing keys grouped by prefix (Top {args.top_groups}):")
            for name, count in groups:
                print(f"  - `{name}`: {count}")
        else:
            print(f"\nMissing keys grouped by prefix (Top {args.top_groups}):")
            for name, count in groups:
                print(f"  {name}: {count}")

    if args.top_missing > 0:
        head = missing[: args.top_missing]
        if args.as_markdown:
            print(f"\n- Missing keys (first {args.top_missing}):")
            for k in head:
                print(f"  - `{k}`")
        else:
            print(f"\nMissing keys (first {args.top_missing}):")
            for k in head:
                print(f"  {k}")

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
