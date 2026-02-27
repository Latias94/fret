#!/usr/bin/env python3
"""
Validate (and optionally regenerate) the diagnostics script registry.

This is intentionally dependency-free (stdlib only) so it can run in CI.

Registry scope (v1):
- "Promoted" scripts that are reachable from in-tree suites:
  - tools/diag-scripts/suites/**.json (script_redirect stubs)
- Preludes:
  - tools/diag-scripts/_prelude/*.json

Non-goal (v1): index the entire script library (thousands of ad-hoc scripts).
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


REPO_ROOT_SENTINEL = "Cargo.toml"
SCRIPTS_DIR = Path("tools/diag-scripts")
REGISTRY_PATH = SCRIPTS_DIR / "index.json"
SUITES_DIR = SCRIPTS_DIR / "suites"
PRELUDE_DIR = SCRIPTS_DIR / "_prelude"


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


def resolve_redirect_path(repo_root: Path, path: Path, *, max_hops: int = 16) -> Path:
    """
    Resolve a tooling-side script_redirect chain to a canonical script JSON file.
    """
    seen: set[Path] = set()
    cur = path
    for _ in range(max_hops):
        cur = cur.resolve()
        if cur in seen:
            chain = " -> ".join(str(p.relative_to(repo_root)).replace("\\", "/") for p in seen)
            raise SystemExit(f"error: redirect loop detected while resolving {path}: {chain}")
        seen.add(cur)
        if not cur.is_file():
            raise SystemExit(f"error: redirect target does not exist: {cur}")
        obj = read_json(cur)
        if not is_redirect_stub(obj):
            return cur
        to = obj.get("to")
        if not isinstance(to, str) or not to.strip():
            raise SystemExit(f"error: invalid script_redirect stub (missing 'to'): {cur}")
        cur = (repo_root / Path(to)).resolve()
    raise SystemExit(f"error: redirect chain exceeded max hops ({max_hops}): {path}")


@dataclass(frozen=True)
class ScriptRegistryEntryV1:
    id: str
    path: str
    tags: list[str]
    target_hints: list[str]
    required_capabilities: list[str]
    suite_memberships: list[str]


def normalize_string_list(v: Any) -> list[str]:
    if not isinstance(v, list):
        return []
    out: list[str] = []
    for item in v:
        if isinstance(item, str) and item.strip():
            out.append(item.strip())
    return out


def derive_entry_id(script_path: Path) -> str:
    # Draft rule: stable id is the file stem (path-independent).
    #
    # Note: if we ever need a stronger guarantee (avoid stem collisions while
    # still allowing fearless path moves), introduce an explicit `meta.id` field
    # in scripts and let it override this default.
    return script_path.stem


def build_registry(repo_root: Path) -> dict[str, Any]:
    if not (repo_root / SUITES_DIR).is_dir():
        raise SystemExit(f"error: suites dir not found: {repo_root / SUITES_DIR}")

    canonical_to_suites: dict[Path, set[str]] = {}

    # 1) Suites: stubs under tools/diag-scripts/suites/<suite>/*.json
    for suite_dir in sorted((repo_root / SUITES_DIR).iterdir()):
        if not suite_dir.is_dir():
            continue
        suite_name = suite_dir.name
        for stub in sorted(suite_dir.glob("*.json")):
            stub_obj = read_json(stub)
            if not is_redirect_stub(stub_obj):
                raise SystemExit(
                    f"error: suite entry is expected to be a script_redirect stub: {stub}"
                )
            to = stub_obj.get("to")
            if not isinstance(to, str) or not to.strip():
                raise SystemExit(f"error: invalid suite stub (missing to): {stub}")
            canonical = resolve_redirect_path(repo_root, repo_root / Path(to))
            canonical_to_suites.setdefault(canonical, set()).add(suite_name)

    # 2) Preludes: canonical scripts under tools/diag-scripts/_prelude/*.json
    if (repo_root / PRELUDE_DIR).is_dir():
        for p in sorted((repo_root / PRELUDE_DIR).glob("*.json")):
            obj = read_json(p)
            if is_redirect_stub(obj):
                continue
            canonical_to_suites.setdefault(p.resolve(), set()).add("_prelude")

    entries: list[ScriptRegistryEntryV1] = []
    seen_ids: dict[str, Path] = {}

    for script_path in sorted(canonical_to_suites.keys(), key=lambda p: p.name.lower()):
        obj = read_json(script_path)
        if is_redirect_stub(obj):
            raise SystemExit(f"error: canonical set includes a redirect stub: {script_path}")

        meta: Any = obj.get("meta") if isinstance(obj, dict) else None
        if not isinstance(meta, dict):
            meta = {}

        tags = sorted(set(normalize_string_list(meta.get("tags"))))
        target_hints = normalize_string_list(meta.get("target_hints"))
        required_capabilities = sorted(set(normalize_string_list(meta.get("required_capabilities"))))

        meta_id = meta.get("id")
        if isinstance(meta_id, str) and meta_id.strip():
            entry_id = meta_id.strip()
        else:
            entry_id = derive_entry_id(script_path)
        if entry_id in seen_ids:
            prev = seen_ids[entry_id]
            raise SystemExit(
                "error: duplicate registry id detected (file stem collision). "
                f"id={entry_id} a={prev} b={script_path}"
            )
        seen_ids[entry_id] = script_path

        rel = script_path.relative_to(repo_root).as_posix()
        suite_memberships = sorted(canonical_to_suites.get(script_path, set()))

        entries.append(
            ScriptRegistryEntryV1(
                id=entry_id,
                path=rel,
                tags=tags,
                target_hints=target_hints,
                required_capabilities=required_capabilities,
                suite_memberships=suite_memberships,
            )
        )

    entries.sort(key=lambda e: e.id)

    return {
        "schema_version": 1,
        "kind": "diag_script_registry",
        "scope": "suites+prelude",
        "scripts": [e.__dict__ for e in entries],
    }


def canonical_json_bytes(obj: Any) -> bytes:
    return (json.dumps(obj, indent=2, sort_keys=True) + "\n").encode("utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(description="Validate the diag script registry (index.json).")
    ap.add_argument(
        "--cwd",
        default=".",
        help="Starting directory used to locate repo root (default: .).",
    )
    ap.add_argument(
        "--write",
        action="store_true",
        help="Rewrite tools/diag-scripts/index.json to the expected content.",
    )
    args = ap.parse_args()

    repo_root = find_repo_root(Path(args.cwd))
    expected = build_registry(repo_root)
    expected_bytes = canonical_json_bytes(expected)

    registry_path = repo_root / REGISTRY_PATH
    if args.write:
        registry_path.parent.mkdir(parents=True, exist_ok=True)
        registry_path.write_bytes(expected_bytes)
        print(f"wrote: {registry_path}")
        return

    if not registry_path.is_file():
        raise SystemExit(
            f"error: missing registry file: {registry_path} (run with --write to generate)"
        )

    actual = read_json(registry_path)
    actual_bytes = canonical_json_bytes(actual)
    if actual_bytes != expected_bytes:
        print("error: diag script registry is out of date:", file=sys.stderr)
        print(f"- file: {REGISTRY_PATH.as_posix()}", file=sys.stderr)
        print("hint: run `python tools/check_diag_scripts_registry.py --write`", file=sys.stderr)
        raise SystemExit(2)

    print("ok: diag script registry is up to date.")


if __name__ == "__main__":
    main()
