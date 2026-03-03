#!/usr/bin/env python3
"""
Migrate diag script suites from "redirect stub files" to "suite manifest" format.

This is a tooling-only refactor intended to reduce file-count noise under:
  tools/diag-scripts/suites/<suite>/**/*.json

Suite manifest format (v1):
  tools/diag-scripts/suites/<suite>/suite.json
  {
    "schema_version": 1,
    "kind": "diag_script_suite_manifest",
    "scripts": ["tools/diag-scripts/...", ...]
  }

Important:
- This does not change any runtime contracts.
- It intentionally keeps suite membership deterministic.
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


REPO_ROOT_SENTINEL = "Cargo.toml"
SUITES_DIR = Path("tools/diag-scripts/suites")
MANIFEST_FILENAME = "suite.json"
ALT_MANIFEST_FILENAME = "_suite.json"


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


def is_redirect_stub(obj: Any) -> bool:
    return isinstance(obj, dict) and obj.get("kind") == "script_redirect"


def is_suite_manifest(obj: Any) -> bool:
    return isinstance(obj, dict) and obj.get("kind") == "diag_script_suite_manifest"


def canonical_suite_json_bytes(obj: Any) -> bytes:
    # Keep suite.json stable and review-friendly: no key sorting, but consistent indentation.
    return (json.dumps(obj, indent=2, sort_keys=False) + "\n").encode("utf-8")


def normalize_script_path(s: str) -> str:
    return s.strip().replace("\\", "/")


@dataclass(frozen=True)
class SuiteMigrationPlan:
    suite: str
    suite_dir: Path
    manifest_path: Path
    scripts: list[str]
    delete_paths: list[Path]


def plan_suite(repo_root: Path, suite_dir: Path) -> SuiteMigrationPlan | None:
    if not suite_dir.is_dir():
        return None

    suite = suite_dir.name
    manifest_path = suite_dir / MANIFEST_FILENAME
    alt_manifest_path = suite_dir / ALT_MANIFEST_FILENAME

    if alt_manifest_path.is_file() and not manifest_path.is_file():
        manifest_path = alt_manifest_path

    # Collect all legacy stub JSONs (excluding suite.json/_suite.json).
    stub_paths = [
        p
        for p in sorted(suite_dir.rglob("*.json"))
        if p.name not in (MANIFEST_FILENAME, ALT_MANIFEST_FILENAME)
    ]

    scripts: list[str] = []
    delete_paths: list[Path] = []

    if manifest_path.is_file():
        obj = read_json(manifest_path)
        if not is_suite_manifest(obj):
            raise SystemExit(
                f"error: suite manifest kind mismatch (expected diag_script_suite_manifest): {manifest_path}"
            )
        scripts_obj = obj.get("scripts")
        if not isinstance(scripts_obj, list):
            raise SystemExit(
                f"error: invalid suite manifest (expected list field: scripts): {manifest_path}"
            )
        scripts = [normalize_script_path(str(s)) for s in scripts_obj if str(s).strip()]

        # If there are still stub files, we will delete them (but we do not attempt to merge).
        delete_paths = stub_paths
    else:
        # Derive scripts from redirect stubs, preserving legacy deterministic ordering:
        # lexicographic by stub path (relative to repo root).
        for stub in stub_paths:
            obj = read_json(stub)
            if not is_redirect_stub(obj):
                raise SystemExit(
                    f"error: suite entry is not a script_redirect stub: {stub}"
                )
            to = obj.get("to")
            if not isinstance(to, str) or not to.strip():
                raise SystemExit(f"error: invalid script_redirect stub (missing to): {stub}")
            scripts.append(normalize_script_path(to))
            delete_paths.append(stub)

    if not scripts:
        # Skip empty suites (should not exist, but avoid writing junk).
        return None

    # De-dupe while preserving order.
    seen: set[str] = set()
    deduped: list[str] = []
    for s in scripts:
        if s in seen:
            continue
        seen.add(s)
        deduped.append(s)

    # Validate targets exist (best-effort; we only require the file exists on disk).
    for s in deduped:
        target = (repo_root / Path(s)).resolve()
        if not target.exists():
            raise SystemExit(
                f"error: suite script path does not exist: suite={suite} path={s} (resolved: {target})"
            )

    # Use canonical suite.json path for output when converting.
    out_manifest_path = suite_dir / MANIFEST_FILENAME
    return SuiteMigrationPlan(
        suite=suite,
        suite_dir=suite_dir,
        manifest_path=out_manifest_path,
        scripts=deduped,
        delete_paths=delete_paths,
    )


def apply_plan(plan: SuiteMigrationPlan) -> None:
    obj = {
        "schema_version": 1,
        "kind": "diag_script_suite_manifest",
        "scripts": plan.scripts,
    }
    plan.manifest_path.write_bytes(canonical_suite_json_bytes(obj))

    for p in plan.delete_paths:
        try:
            p.unlink()
        except FileNotFoundError:
            continue


def main() -> None:
    ap = argparse.ArgumentParser(description="Migrate diag script suites to suite.json manifests.")
    ap.add_argument(
        "--cwd",
        default=".",
        help="Starting directory used to locate repo root (default: .).",
    )
    ap.add_argument(
        "--apply",
        action="store_true",
        help="Apply migration (write suite.json + delete stub files).",
    )
    ap.add_argument(
        "--suite",
        action="append",
        default=[],
        help="Only migrate these suite names (repeatable). Default: all suites.",
    )
    args = ap.parse_args()

    repo_root = find_repo_root(Path(args.cwd))
    suites_root = repo_root / SUITES_DIR
    if not suites_root.is_dir():
        raise SystemExit(f"error: suites dir not found: {suites_root}")

    only = set([s.strip() for s in args.suite if s.strip()])

    plans: list[SuiteMigrationPlan] = []
    for suite_dir in sorted(suites_root.iterdir()):
        if not suite_dir.is_dir():
            continue
        if only and suite_dir.name not in only:
            continue
        plan = plan_suite(repo_root, suite_dir)
        if plan is None:
            continue
        plans.append(plan)

    if not plans:
        print("no suites to migrate")
        return

    for p in plans:
        print(f"- {p.suite}: scripts={len(p.scripts)} delete={len(p.delete_paths)}")

    if not args.apply:
        print("dry run (use --apply to write changes)")
        return

    for p in plans:
        apply_plan(p)

    print("ok")


if __name__ == "__main__":
    main()

