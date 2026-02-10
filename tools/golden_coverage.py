#!/usr/bin/env python3
"""
Golden coverage heuristics for web goldens.

This is a cross-platform replacement for `tools/golden_coverage.ps1`.

Notes:
- "Gated" coverage uses a string-literal heuristic: a golden key counts as gated if
  it appears as `"key"` in any non-smoke test file under `ecosystem/fret-ui-shadcn/tests`.
- "Targeted" gates exclude specific broad-scope test files (defaults match the PS script).
- Smoke coverage is reported only when we can infer the smoke test targets the requested style.
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


def _golden_names(repo_root: Path, golden_dir: Path, *, tracked_only: bool) -> list[str]:
    if tracked_only:
        repo_root_abs = repo_root.resolve()
        golden_dir_abs = golden_dir.resolve()
        try:
            rel = golden_dir_abs.relative_to(repo_root_abs)
        except ValueError as exc:
            raise RuntimeError(
                f"GoldenDir is not under RepoRoot (RepoRoot={repo_root_abs}, GoldenDir={golden_dir_abs})"
            ) from exc
        rel_posix = rel.as_posix()
        tracked = _git_ls_files(repo_root_abs, rel_posix)
        names = [
            Path(p).stem
            for p in tracked
            if p.endswith(".json") and not p.endswith(".json\n")
        ]
        return sorted(set(names))

    names = [p.stem for p in golden_dir.glob("*.json") if p.is_file()]
    return sorted(set(names))


def _normalize_open_suffix(keys: list[str]) -> list[str]:
    out = [re.sub(r"\.open$", "", k) for k in keys]
    return sorted(set(out))


_JOIN_TOKEN_RE = re.compile(r'\\.join\\("([^"]+)"\\)')


def _infer_smoke_style(smoke_test: Path, *, kind: str) -> str | None:
    if not smoke_test.is_file():
        return None
    text = smoke_test.read_text(encoding="utf-8")
    tokens = [m.group(1) for m in _JOIN_TOKEN_RE.finditer(text)]
    if not tokens:
        return None
    for idx in range(0, len(tokens) - 1):
        if tokens[idx] == "goldens" and tokens[idx + 1] == kind:
            style_tokens = tokens[idx + 2 :]
            return "/".join(style_tokens) if style_tokens else None
    return None


_RUST_STRING_KEY_RE = re.compile(r'"([a-z0-9][a-z0-9_.-]{1,200})"')


def _keys_used_by_file(text: str, *, key_set: set[str]) -> set[str]:
    hits: set[str] = set()
    for m in _RUST_STRING_KEY_RE.finditer(text):
        s = m.group(1)
        if s in key_set:
            hits.add(s)
    return hits


def _split_prefix(key: str, split_pattern: str) -> str:
    parts = re.split(split_pattern, key)
    return parts[0] if parts and parts[0] else key


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--kind",
        choices=["shadcn-web", "radix-web"],
        default="shadcn-web",
        help="Golden kind directory under goldens/.",
    )
    parser.add_argument(
        "--style",
        default="v4/new-york-v4",
        help='Golden style directory under goldens/<kind>/ (e.g. "v4/new-york-v4").',
    )
    parser.add_argument("--repo-root", default="", help="Repo root (auto-detected if empty).")
    parser.add_argument(
        "--normalize-open-suffix",
        action=argparse.BooleanOptionalAction,
        default=True,
        help='For shadcn-web, normalize keys by stripping a trailing ".open".',
    )
    parser.add_argument("--tracked-only", action="store_true", help="Only count tracked golden files.")
    parser.add_argument("--top-missing", type=int, default=50)
    parser.add_argument("--group-missing-by-prefix", action="store_true")
    parser.add_argument("--group-used-by-prefix", action="store_true")
    parser.add_argument("--top-groups", type=int, default=25)
    parser.add_argument("--group-split-pattern", default=r"[\\.-]")
    parser.add_argument("--filter-missing-prefix", default="")
    parser.add_argument("--filter-used-prefix", default="")
    parser.add_argument("--show-used", action="store_true")
    parser.add_argument("--show-missing", action="store_true")
    parser.add_argument("--show-targeted-missing", action="store_true")
    parser.add_argument(
        "--targeted-gate-exclude-file",
        action="append",
        default=["web_vs_fret_layout.rs", "snapshots.rs"],
        help="Test file name to exclude from targeted gate coverage. Repeatable.",
    )
    parser.add_argument("--show-gate-breakdown", action="store_true")
    parser.add_argument("--group-untargeted-by-prefix", action="store_true")
    parser.add_argument("--filter-untargeted-prefix", default="")
    parser.add_argument("--as-markdown", action="store_true")

    args = parser.parse_args(argv)

    repo_root = Path(args.repo_root) if args.repo_root else _resolve_repo_root(Path(__file__).parent.parent)
    golden_dir = repo_root / "goldens" / args.kind / args.style
    if not golden_dir.is_dir():
        raise RuntimeError(f"Missing golden directory: {golden_dir}")

    test_dir = repo_root / "ecosystem" / "fret-ui-shadcn" / "tests"
    if not test_dir.is_dir():
        raise RuntimeError(f"Missing test directory: {test_dir}")

    golden_names = _golden_names(repo_root, golden_dir, tracked_only=args.tracked_only)
    golden_keys = list(golden_names)
    if args.kind == "shadcn-web" and args.normalize_open_suffix:
        golden_keys = _normalize_open_suffix(golden_keys)

    test_files = sorted(p for p in test_dir.glob("*.rs") if p.is_file())
    gate_test_files = [p for p in test_files if not p.name.endswith("_goldens_smoke.rs")]

    golden_key_set = set(golden_keys)
    used: set[str] = set()
    used_targeted: set[str] = set()
    used_by_file: dict[str, set[str]] = {}

    excluded_files = set(args.targeted_gate_exclude_file)
    for path in gate_test_files:
        text = path.read_text(encoding="utf-8")
        file_used = _keys_used_by_file(text, key_set=golden_key_set)
        used_by_file[path.name] = file_used
        used |= file_used
        if path.name not in excluded_files:
            used_targeted |= file_used

    used_names = sorted(used)
    missing_names = [k for k in golden_keys if k not in used]

    used_targeted_names = sorted(used_targeted)
    targeted_missing_names = [k for k in golden_keys if k not in used_targeted]

    total_files = len(golden_names)
    total = len(golden_keys)
    used_count = len(used_names)
    missing_count = len(missing_names)
    used_targeted_count = len(used_targeted_names)
    targeted_missing_count = len(targeted_missing_names)

    coverage = round((used_count * 100.0) / total, 1) if total > 0 else 0.0
    targeted_coverage = round((used_targeted_count * 100.0) / total, 1) if total > 0 else 0.0

    smoke_candidate: Path | None = None
    if args.kind == "shadcn-web":
        smoke_candidate = test_dir / "shadcn_web_goldens_smoke.rs"
    elif args.kind == "radix-web":
        smoke_candidate = test_dir / "radix_web_goldens_smoke.rs"

    smoke_test: Path | None = None
    smoke_style: str | None = None
    if smoke_candidate and smoke_candidate.exists():
        smoke_style = _infer_smoke_style(smoke_candidate, kind=args.kind)
        if smoke_style is None or smoke_style == args.style:
            smoke_test = smoke_candidate

    if args.as_markdown:
        tracked_note = " (tracked-only)" if args.tracked_only else ""
        print(f"- `{args.kind}` goldens{tracked_note}: {total_files} files, {total} keys")
        print(
            f"  - gated (any non-smoke test): {used_count} keys ({coverage}%) [string-literal heuristic], {missing_count} missing"
        )
        excluded = ", ".join(sorted(excluded_files))
        print(
            f"  - targeted gates (excluding {excluded}): {used_targeted_count} keys ({targeted_coverage}%), {targeted_missing_count} missing"
        )
        if smoke_test is not None:
            print(f"  - smoke-parse coverage: 100% (via `{smoke_test.name}`)")
        elif smoke_candidate is not None and smoke_style and smoke_style != args.style:
            print(f"  - smoke-parse coverage: n/a (smoke test targets `{smoke_style}`, not `{args.style}`)")
    else:
        print(f"Golden coverage ({args.kind}/{args.style})")
        print(f"  RepoRoot:  {repo_root}")
        print(f"  GoldenDir: {golden_dir}")
        print(f"  TestsDir:  {test_dir}")
        print(f"  Tracked:   {'yes' if args.tracked_only else 'no'}")
        print(f"  Files:     {total_files}")
        print(f"  Keys:      {total} (NormalizeOpenSuffix={args.normalize_open_suffix})")
        print(f"  Gated:     {used_count} keys ({coverage}%) [any non-smoke test, string-literal heuristic]")
        print(f"  Ungated:   {missing_count} keys [not referenced by non-smoke tests]")
        excluded = ", ".join(sorted(excluded_files))
        print(f"  Targeted:  {used_targeted_count} keys ({targeted_coverage}%) [excluding: {excluded}]")
        print(f"  Untargeted:{targeted_missing_count} keys [only gated by excluded files]")
        if smoke_test is not None:
            print(f"  Smoke:     yes (100%, {smoke_test.name})")
        elif smoke_candidate is not None and smoke_style and smoke_style != args.style:
            print(f"  Smoke:     n/a (smoke test targets {smoke_style}, not {args.style})")
        else:
            print("  Smoke:     n/a")

    if args.show_used:
        print("\nGated keys (unique):")
        for k in used_names:
            print(f"  {k}")

    if args.show_missing:
        print(f"\nUngated keys (first {args.top_missing}):")
        for k in missing_names[: max(args.top_missing, 0)]:
            print(f"  {k}")

    if args.show_targeted_missing:
        print(f"\nUntargeted keys (first {args.top_missing}) [only gated by excluded files]:")
        for k in targeted_missing_names[: max(args.top_missing, 0)]:
            print(f"  {k}")

    if args.show_gate_breakdown:
        rows = [(name, len(keys)) for name, keys in used_by_file.items()]
        rows.sort(key=lambda x: x[1], reverse=True)
        if args.as_markdown:
            print("\n### Gate breakdown (keys referenced per test file)")
            for file_name, count in rows:
                pct = round((count * 100.0) / total, 1) if total > 0 else 0.0
                print(f"- `{file_name}`: {count} ({pct}%)")
        else:
            print("\nGate breakdown (keys referenced per test file):")
            for file_name, count in rows:
                pct = round((count * 100.0) / total, 1) if total > 0 else 0.0
                print(f"  {file_name}: {count} ({pct}%)")

    if args.filter_untargeted_prefix.strip():
        prefix = args.filter_untargeted_prefix.strip()
        filtered = [k for k in targeted_missing_names if k.startswith(prefix)]
        if args.as_markdown:
            print(f"\n### Untargeted keys: {prefix} (first {args.top_missing})")
            for k in filtered[: max(args.top_missing, 0)]:
                print(f"- {k}")
        else:
            print(f"\nUntargeted keys: {prefix} (first {args.top_missing})")
            for k in filtered[: max(args.top_missing, 0)]:
                print(f"  {k}")

    if args.group_untargeted_by_prefix:
        prefixes = [_split_prefix(k, args.group_split_pattern) for k in targeted_missing_names]
        groups = Counter(prefixes).most_common(args.top_groups)
        if args.as_markdown:
            print("\n### Untargeted groups (heuristic)")
            for name, count in groups:
                print(f"- `{name}` ({count})")
        else:
            print("\nUntargeted groups (heuristic):")
            for name, count in groups:
                print(f"  {name}: {count}")

    if args.group_missing_by_prefix:
        prefixes = [_split_prefix(k, args.group_split_pattern) for k in missing_names]
        groups = Counter(prefixes).most_common(args.top_groups)
        if args.as_markdown:
            print(f"\n- Missing keys grouped by prefix (Top {args.top_groups}):")
            for name, count in groups:
                print(f"  - `{name}`: {count}")
        else:
            print(f"\nMissing keys grouped by prefix (Top {args.top_groups}):")
            for name, count in groups:
                print(f"  {name}: {count}")

    if args.group_used_by_prefix:
        prefixes = [_split_prefix(k, args.group_split_pattern) for k in used_names]
        groups = Counter(prefixes).most_common(args.top_groups)
        if args.as_markdown:
            print(f"\n- Referenced keys grouped by prefix (Top {args.top_groups}):")
            for name, count in groups:
                print(f"  - `{name}`: {count}")
        else:
            print(f"\nReferenced keys grouped by prefix (Top {args.top_groups}):")
            for name, count in groups:
                print(f"  {name}: {count}")

    if args.filter_missing_prefix.strip():
        prefix = args.filter_missing_prefix.strip()
        filtered = [k for k in missing_names if k.startswith(prefix)]
        if args.as_markdown:
            print(f"\n- Missing keys with prefix `{prefix}`: {len(filtered)}")
            for k in filtered:
                print(f"  - `{k}`")
        else:
            print(f"\nMissing keys with prefix {prefix}: {len(filtered)}")
            for k in filtered:
                print(f"  {k}")

    if args.filter_used_prefix.strip():
        prefix = args.filter_used_prefix.strip()
        filtered = [k for k in used_names if k.startswith(prefix)]
        if args.as_markdown:
            print(f"\n- Referenced keys with prefix `{prefix}`: {len(filtered)}")
            for k in filtered:
                print(f"  - `{k}`")
        else:
            print(f"\nReferenced keys with prefix {prefix}: {len(filtered)}")
            for k in filtered:
                print(f"  {k}")

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
