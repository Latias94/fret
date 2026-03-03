#!/usr/bin/env python3
"""
Edit diag suite manifests safely.

Suites live under:
  tools/diag-scripts/suites/<suite-name>/suite.json

Manifest format (v1):
  {
    "schema_version": 1,
    "kind": "diag_script_suite_manifest",
    "scripts": ["tools/diag-scripts/...", ...]
  }

This tool:
- keeps `scripts` de-duped and deterministically ordered,
- validates that referenced script paths exist,
- optionally refreshes the promoted registry (`tools/diag-scripts/index.json`).
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional


REPO_ROOT_SENTINEL = "Cargo.toml"
SUITES_DIR = Path("tools/diag-scripts/suites")
MANIFEST_PATH = Path("suite.json")


def find_repo_root(start: Path) -> Path:
    cur = start.resolve()
    for parent in [cur, *cur.parents]:
        if (parent / REPO_ROOT_SENTINEL).is_file():
            return parent
    raise SystemExit(
        f"error: failed to locate repo root (missing {REPO_ROOT_SENTINEL} in ancestors)"
    )


def read_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as e:
        raise SystemExit(f"error: failed to read JSON: {path} ({e})")


def write_json(path: Path, obj: Any) -> None:
    path.write_text(json.dumps(obj, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def normalize_script_path(s: str) -> str:
    return s.strip().replace("\\", "/")


def is_suite_manifest(obj: Any) -> bool:
    return isinstance(obj, dict) and obj.get("kind") == "diag_script_suite_manifest"


@dataclass(frozen=True)
class Suite:
    suite: str
    suite_dir: Path
    manifest_path: Path
    scripts: list[str]


def load_suite(repo_root: Path, suite: str) -> Suite:
    suite_dir = (repo_root / SUITES_DIR / suite).resolve()
    manifest_path = suite_dir / MANIFEST_PATH
    if not suite_dir.is_dir():
        raise SystemExit(f"error: suite directory not found: {suite_dir}")
    if not manifest_path.is_file():
        raise SystemExit(f"error: suite manifest not found: {manifest_path}")

    obj = read_json(manifest_path)
    if not is_suite_manifest(obj):
        raise SystemExit(
            f"error: suite manifest kind mismatch (expected diag_script_suite_manifest): {manifest_path}"
        )
    schema_version = obj.get("schema_version")
    if schema_version != 1:
        raise SystemExit(
            f"error: suite manifest schema_version mismatch (expected 1): {manifest_path}"
        )

    scripts_obj = obj.get("scripts")
    if not isinstance(scripts_obj, list):
        raise SystemExit(
            f"error: suite manifest missing list field: scripts ({manifest_path})"
        )

    scripts: list[str] = []
    for item in scripts_obj:
        if not isinstance(item, str) or not item.strip():
            raise SystemExit(
                f"error: suite manifest contains invalid scripts entry (expected non-empty string): {manifest_path}"
            )
        scripts.append(normalize_script_path(item))

    return Suite(
        suite=suite, suite_dir=suite_dir, manifest_path=manifest_path, scripts=scripts
    )


def canonicalize_suite_scripts(repo_root: Path, suite: Suite) -> list[str]:
    # De-dupe and sort for deterministic diffs.
    seen: set[str] = set()
    deduped: list[str] = []
    for s in suite.scripts:
        s = normalize_script_path(s)
        if not s:
            continue
        if s in seen:
            continue
        seen.add(s)
        deduped.append(s)

    deduped.sort()

    # Validate targets exist.
    for s in deduped:
        target = (repo_root / Path(s)).resolve()
        if not target.exists():
            raise SystemExit(
                f"error: suite script path does not exist: suite={suite.suite} path={s} (resolved: {target})"
            )

    return deduped


def write_suite(repo_root: Path, suite: Suite) -> None:
    scripts = canonicalize_suite_scripts(repo_root, suite)
    write_json(
        suite.manifest_path,
        {
            "schema_version": 1,
            "kind": "diag_script_suite_manifest",
            "scripts": scripts,
        },
    )


def refresh_promoted_registry(repo_root: Path) -> None:
    cmd = [sys.executable, "tools/check_diag_scripts_registry.py", "--write"]
    subprocess.check_call(cmd, cwd=repo_root)
    cmd = [sys.executable, "tools/check_diag_scripts_registry.py"]
    subprocess.check_call(cmd, cwd=repo_root)


def cmd_check(repo_root: Path, suite: Optional[str]) -> None:
    suites_root = repo_root / SUITES_DIR
    if suite is not None:
        s = load_suite(repo_root, suite)
        canonicalize_suite_scripts(repo_root, s)
        print(f"ok: {suite}")
        return

    for suite_dir in sorted(suites_root.iterdir()):
        if not suite_dir.is_dir():
            continue
        name = suite_dir.name
        s = load_suite(repo_root, name)
        canonicalize_suite_scripts(repo_root, s)
    print("ok: all suites")


def cmd_add(repo_root: Path, suite: str, scripts: list[str], refresh_index: bool) -> None:
    s = load_suite(repo_root, suite)
    new_scripts = s.scripts + [normalize_script_path(raw) for raw in scripts]
    s = Suite(
        suite=s.suite,
        suite_dir=s.suite_dir,
        manifest_path=s.manifest_path,
        scripts=new_scripts,
    )
    write_suite(repo_root, s)
    if refresh_index:
        refresh_promoted_registry(repo_root)
    print(f"ok: added {len(scripts)} scripts to {suite}")


def cmd_remove(repo_root: Path, suite: str, scripts: list[str], refresh_index: bool) -> None:
    s = load_suite(repo_root, suite)
    remove = set([normalize_script_path(x) for x in scripts])
    before = len(s.scripts)
    s = Suite(
        suite=s.suite,
        suite_dir=s.suite_dir,
        manifest_path=s.manifest_path,
        scripts=[x for x in s.scripts if normalize_script_path(x) not in remove],
    )
    write_suite(repo_root, s)
    if refresh_index:
        refresh_promoted_registry(repo_root)
    print(f"ok: removed {before - len(s.scripts)} scripts from {suite}")


def cmd_sort(repo_root: Path, suite: str, refresh_index: bool) -> None:
    s = load_suite(repo_root, suite)
    write_suite(repo_root, s)
    if refresh_index:
        refresh_promoted_registry(repo_root)
    print(f"ok: sorted {suite}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Edit diag suite manifests (suite.json).")
    ap.add_argument(
        "--cwd",
        default=".",
        help="Starting directory used to locate repo root (default: .).",
    )

    sub = ap.add_subparsers(dest="cmd", required=True)

    ap_check = sub.add_parser("check", help="Validate suite manifests (does not rewrite).")
    ap_check.add_argument("--suite", help="Suite name (default: all suites).")

    ap_fmt = sub.add_parser("fmt", help="Rewrite suite manifests in canonical form (sorted, de-duped).")
    ap_fmt.add_argument("--suite", help="Suite name (default: all suites).")
    ap_fmt.add_argument(
        "--refresh-index",
        action="store_true",
        help="Regenerate tools/diag-scripts/index.json after formatting.",
    )

    ap_add = sub.add_parser("add", help="Add script paths to a suite manifest.")
    ap_add.add_argument("suite", help="Suite name (directory under tools/diag-scripts/suites).")
    ap_add.add_argument("scripts", nargs="+", help="Script paths to add (repo-relative).")
    ap_add.add_argument(
        "--refresh-index",
        action="store_true",
        help="Regenerate tools/diag-scripts/index.json after editing.",
    )

    ap_rm = sub.add_parser("remove", help="Remove script paths from a suite manifest.")
    ap_rm.add_argument("suite", help="Suite name.")
    ap_rm.add_argument("scripts", nargs="+", help="Script paths to remove (repo-relative).")
    ap_rm.add_argument("--refresh-index", action="store_true")

    ap_sort = sub.add_parser("sort", help="Canonicalize ordering and de-dupe a suite manifest.")
    ap_sort.add_argument("suite", help="Suite name.")
    ap_sort.add_argument("--refresh-index", action="store_true")

    args = ap.parse_args()
    repo_root = find_repo_root(Path(args.cwd))

    if args.cmd == "check":
        cmd_check(repo_root, args.suite)
        return
    if args.cmd == "fmt":
        suites_root = repo_root / SUITES_DIR
        if args.suite is not None:
            s = load_suite(repo_root, args.suite)
            write_suite(repo_root, s)
            if args.refresh_index:
                refresh_promoted_registry(repo_root)
            print(f"ok: formatted {args.suite}")
            return

        for suite_dir in sorted(suites_root.iterdir()):
            if not suite_dir.is_dir():
                continue
            name = suite_dir.name
            s = load_suite(repo_root, name)
            write_suite(repo_root, s)
        if args.refresh_index:
            refresh_promoted_registry(repo_root)
        print("ok: formatted all suites")
        return
    if args.cmd == "add":
        cmd_add(repo_root, args.suite, args.scripts, args.refresh_index)
        return
    if args.cmd == "remove":
        cmd_remove(repo_root, args.suite, args.scripts, args.refresh_index)
        return
    if args.cmd == "sort":
        cmd_sort(repo_root, args.suite, args.refresh_index)
        return

    raise SystemExit(f"unknown command: {args.cmd}")


if __name__ == "__main__":
    main()
