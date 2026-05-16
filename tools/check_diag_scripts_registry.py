#!/usr/bin/env python3
"""
Validate (and optionally regenerate) the diagnostics script registry.

This is intentionally dependency-free (stdlib only) so it can run in CI.

Registry scope (v1):
- "Promoted" scripts that are reachable from in-tree suites:
  - tools/diag-scripts/suites/<suite>/**.json (script_redirect stubs), or
  - tools/diag-scripts/suites/<suite>/suite.json (suite manifest)
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
from typing import Any, Optional


REPO_ROOT_SENTINEL = "Cargo.toml"
SCRIPTS_DIR = Path("tools/diag-scripts")
REGISTRY_PATH = SCRIPTS_DIR / "index.json"
SUITES_DIR = SCRIPTS_DIR / "suites"
PRELUDE_DIR = SCRIPTS_DIR / "_prelude"
SUITE_MANIFEST_FILENAMES = ["suite.json", "_suite.json"]
STRICT_CLICK_VISIBILITY_SUITES = {"ui-gallery-combobox", "ui-gallery-select"}
STRICT_UI_GALLERY_CONTENT_TEST_ID_PREFIXES = (
    "ui-gallery-combobox-",
    "ui-gallery-select-",
)
STRICT_PAGE_ENTRY_SUITES = {"ui-gallery-motion-pilot", "ui-gallery-select", "ui-gallery-combobox"}
UI_GALLERY_PAGE_ENTRY_RULES = {
    "motion_presets": {
        "page_id": "ui-gallery-page-motion-presets",
        "content_prefixes": ("ui-gallery-motion-presets-",),
        "start_page_values": ("motion_presets",),
        "global_ids": (
            "ui-gallery-motion-preset-trigger",
            "ui-gallery-motion-preset-trigger.chrome",
        ),
    },
    "combobox": {
        "page_id": "ui-gallery-page-combobox",
        "content_prefixes": ("ui-gallery-combobox-",),
        "start_page_values": ("combobox",),
        "global_ids": (),
    },
    "select": {
        "page_id": "ui-gallery-page-select",
        "content_prefixes": ("ui-gallery-select-",),
        "start_page_values": ("select",),
        "global_ids": (),
    },
}


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


def find_suite_manifest_path(suite_dir: Path) -> Optional[Path]:
    for name in SUITE_MANIFEST_FILENAMES:
        candidate = suite_dir / name
        if candidate.is_file():
            return candidate
    return None


def path_is_relative_to(path: Path, parent: Path) -> bool:
    try:
        path.resolve().relative_to(parent.resolve())
        return True
    except ValueError:
        return False


def suite_name_from_dir(repo_root: Path, suite_dir: Path) -> str:
    return suite_dir.relative_to(repo_root / SUITES_DIR).as_posix()


def suite_manifest_script_paths(obj: Any) -> list[str]:
    if not is_suite_manifest(obj):
        return []
    scripts = obj.get("scripts")
    if not isinstance(scripts, list):
        raise SystemExit("error: invalid suite manifest (expected list field: scripts)")
    out: list[str] = []
    for item in scripts:
        if isinstance(item, str) and item.strip():
            out.append(item.strip())
        else:
            raise SystemExit(
                "error: invalid suite manifest (scripts entries must be non-empty strings)"
            )
    return out


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
    required_launch_features: list[str]
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

    # 1) Suites: either legacy stubs under tools/diag-scripts/suites/<suite>/**/*.json,
    # or a suite manifest under tools/diag-scripts/suites/<suite>/suite.json.
    #
    # A suite directory may also contain nested suite manifests, e.g.
    # tools/diag-scripts/suites/docking-arbitration/{common,windows}/suite.json.
    # Those child manifests are owned by their own suite names and are not legacy stubs.
    suite_root = repo_root / SUITES_DIR
    manifest_dirs: dict[Path, Path] = {}
    for suite_dir in sorted([p for p in suite_root.rglob("*") if p.is_dir()]):
        manifest_path = find_suite_manifest_path(suite_dir)
        if manifest_path is not None:
            manifest_dirs[suite_dir.resolve()] = manifest_path

    for suite_dir_resolved, manifest_path in sorted(
        manifest_dirs.items(), key=lambda item: suite_name_from_dir(repo_root, item[0])
    ):
        suite_dir = suite_dir_resolved
        suite_name = suite_name_from_dir(repo_root, suite_dir)
        nested_manifest_dirs = [
            d
            for d in manifest_dirs
            if d != suite_dir and path_is_relative_to(d, suite_dir)
        ]

        # Do not allow mixing a manifest with legacy stubs in the same suite ownership area.
        # Nested suite manifests are allowed and skipped here because they own their own membership.
        other_json = [
            p
            for p in suite_dir.rglob("*.json")
            if p.resolve() != manifest_path.resolve()
            and not any(path_is_relative_to(p, nested) for nested in nested_manifest_dirs)
        ]
        if other_json:
            shown = "\n".join(
                f"  - {p.relative_to(repo_root).as_posix()}" for p in other_json[:10]
            )
            raise SystemExit(
                "error: suite directory mixes suite manifest with legacy *.json stubs:\n"
                f"- suite: {suite_name}\n"
                f"- manifest: {manifest_path.relative_to(repo_root).as_posix()}\n"
                f"- other json files (first 10):\n{shown}\n"
                "hint: delete legacy stubs, move them under a nested suite manifest, or remove the manifest"
            )

        manifest_obj = read_json(manifest_path)
        if not is_suite_manifest(manifest_obj):
            raise SystemExit(
                "error: suite manifest must have kind=diag_script_suite_manifest: "
                f"{manifest_path}"
            )
        script_paths = suite_manifest_script_paths(manifest_obj)
        if not script_paths:
            raise SystemExit(f"error: suite manifest contains no scripts: {manifest_path}")
        for to in script_paths:
            canonical = resolve_redirect_path(repo_root, repo_root / Path(to))
            canonical_to_suites.setdefault(canonical, set()).add(suite_name)

    for suite_dir in sorted(suite_root.iterdir()):
        if not suite_dir.is_dir():
            continue
        suite_name = suite_dir.name

        if suite_dir.resolve() in manifest_dirs:
            continue

        nested_manifest_dirs = [
            d
            for d in manifest_dirs
            if path_is_relative_to(d, suite_dir)
        ]
        stubs = [
            p
            for p in sorted(suite_dir.rglob("*.json"))
            if not any(path_is_relative_to(p, nested) for nested in nested_manifest_dirs)
        ]
        for stub in stubs:
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
        required_launch_features = sorted(set(normalize_string_list(meta.get("required_launch_features"))))

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
                required_launch_features=required_launch_features,
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
    return (json.dumps(obj, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode("utf-8")


def test_id_from_target_ref(value: Any) -> Optional[str]:
    if (
        isinstance(value, dict)
        and value.get("kind") == "test_id"
        and isinstance(value.get("id"), str)
        and value["id"].strip()
    ):
        return value["id"].strip()
    return None


def is_strict_ui_gallery_content_target(test_id: str) -> bool:
    return test_id.startswith(STRICT_UI_GALLERY_CONTENT_TEST_ID_PREFIXES)


def collect_test_ids(value: Any) -> list[str]:
    out: list[str] = []
    if isinstance(value, dict):
        test_id = test_id_from_target_ref(value)
        if test_id is not None:
            out.append(test_id)
        for child in value.values():
            out.extend(collect_test_ids(child))
    elif isinstance(value, list):
        for child in value:
            out.extend(collect_test_ids(child))
    return out


def is_page_scoped_test_id(test_id: str, rule: dict[str, Any]) -> bool:
    global_ids = rule.get("global_ids")
    if isinstance(global_ids, tuple) and test_id in global_ids:
        return False
    prefixes = rule.get("content_prefixes")
    return isinstance(prefixes, tuple) and any(test_id.startswith(prefix) for prefix in prefixes)


def start_pages_from_meta(obj: Any) -> set[str]:
    meta = obj.get("meta") if isinstance(obj, dict) else None
    if not isinstance(meta, dict):
        return set()
    env_defaults = meta.get("env_defaults")
    if not isinstance(env_defaults, dict):
        return set()
    start_page = env_defaults.get("FRET_UI_GALLERY_START_PAGE")
    if not isinstance(start_page, str) or not start_page.strip():
        return set()
    return {start_page.strip()}


def lint_strict_page_entry(repo_root: Path, registry: dict[str, Any]) -> list[str]:
    """
    Check promoted scripts whose page-local selectors should not rely on the Gallery default page.

    The first strict page is Motion Presets because a real diagnostics run already found this
    failure mode: waiting for a page-local probe without first entering the owning page.
    """
    scripts = registry.get("scripts")
    if not isinstance(scripts, list):
        return ["registry scripts must be a list"]

    violations: list[str] = []

    for entry in scripts:
        if not isinstance(entry, dict):
            continue
        memberships = entry.get("suite_memberships")
        if not isinstance(memberships, list):
            continue
        if not STRICT_PAGE_ENTRY_SUITES.intersection(
            item for item in memberships if isinstance(item, str)
        ):
            continue

        rel_path = entry.get("path")
        if not isinstance(rel_path, str) or not rel_path.strip():
            continue

        script_path = repo_root / Path(rel_path)
        obj = read_json(script_path)
        steps = obj.get("steps") if isinstance(obj, dict) else None
        if not isinstance(steps, list):
            continue

        entered_pages: set[str] = set()
        start_pages = start_pages_from_meta(obj)
        for page_name, rule in UI_GALLERY_PAGE_ENTRY_RULES.items():
            start_page_values = rule.get("start_page_values")
            if isinstance(start_page_values, tuple) and start_pages.intersection(start_page_values):
                entered_pages.add(page_name)

        for index, step in enumerate(steps):
            if not isinstance(step, dict):
                continue

            step_type = step.get("type")
            ids = collect_test_ids(step)
            for page_name, rule in UI_GALLERY_PAGE_ENTRY_RULES.items():
                page_id = rule.get("page_id")
                if isinstance(page_id, str) and page_id in ids:
                    entered_pages.add(page_name)

                if step_type in {
                    "wait_until",
                    "assert",
                    "click",
                    "click_stable",
                    "move_pointer",
                    "scroll_into_view",
                    "capture_layout_sidecar",
                }:
                    for test_id in ids:
                        if (
                            is_page_scoped_test_id(test_id, rule)
                            and page_name not in entered_pages
                        ):
                            violations.append(
                                f"{rel_path}: step {index}: page-local selector `{test_id}` "
                                f"requires a prior wait/assert for `{page_id}`"
                            )
                            break

    return violations


def lint_strict_click_visibility(repo_root: Path, registry: dict[str, Any]) -> list[str]:
    """
    Check promoted UI Gallery content clicks that are known to run in long pages.

    This intentionally expands suite-by-suite rather than across the full script
    library: the full promoted registry still has legacy click-authoring debt,
    while the strict suites have been cleared to zero violations.
    """
    scripts = registry.get("scripts")
    if not isinstance(scripts, list):
        return ["registry scripts must be a list"]

    violations: list[str] = []

    for entry in scripts:
        if not isinstance(entry, dict):
            continue
        memberships = entry.get("suite_memberships")
        if not isinstance(memberships, list):
            continue
        if not STRICT_CLICK_VISIBILITY_SUITES.intersection(
            item for item in memberships if isinstance(item, str)
        ):
            continue

        rel_path = entry.get("path")
        if not isinstance(rel_path, str) or not rel_path.strip():
            continue

        script_path = repo_root / Path(rel_path)
        obj = read_json(script_path)
        steps = obj.get("steps") if isinstance(obj, dict) else None
        if not isinstance(steps, list):
            continue

        visible_targets: set[str] = set()
        for index, step in enumerate(steps):
            if not isinstance(step, dict):
                continue

            step_type = step.get("type")
            if step_type == "set_window_inner_size":
                visible_targets.clear()

            if step_type == "scroll_into_view":
                target_id = test_id_from_target_ref(step.get("target"))
                if (
                    target_id is not None
                    and is_strict_ui_gallery_content_target(target_id)
                    and step.get("require_fully_within_window") is True
                ):
                    visible_targets.add(target_id)

            if step_type in {"wait_until", "assert"}:
                predicate = step.get("predicate")
                if isinstance(predicate, dict) and predicate.get("kind") == "bounds_within_window":
                    target_id = test_id_from_target_ref(predicate.get("target"))
                    if target_id is not None and is_strict_ui_gallery_content_target(target_id):
                        visible_targets.add(target_id)

            if step_type == "click":
                target_id = test_id_from_target_ref(step.get("target"))
                if target_id is not None and is_strict_ui_gallery_content_target(target_id):
                    violations.append(
                        f"{rel_path}: step {index}: plain click targets long-page content "
                        f"`{target_id}`; use click_stable with a prior bounds_within_window guard"
                    )

            if step_type == "click_stable":
                target_id = test_id_from_target_ref(step.get("target"))
                if (
                    target_id is not None
                    and is_strict_ui_gallery_content_target(target_id)
                    and target_id not in visible_targets
                ):
                    violations.append(
                        f"{rel_path}: step {index}: click_stable target `{target_id}` lacks a "
                        "prior scroll_into_view(require_fully_within_window=true) or "
                        "bounds_within_window guard"
                    )

    return violations


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

    click_visibility_violations = lint_strict_click_visibility(repo_root, expected)
    if click_visibility_violations:
        print("error: promoted diag scripts have unsafe long-page click authoring:", file=sys.stderr)
        for violation in click_visibility_violations:
            print(f"- {violation}", file=sys.stderr)
        raise SystemExit(2)

    page_entry_violations = lint_strict_page_entry(repo_root, expected)
    if page_entry_violations:
        print("error: promoted diag scripts rely on implicit UI Gallery page entry:", file=sys.stderr)
        for violation in page_entry_violations:
            print(f"- {violation}", file=sys.stderr)
        raise SystemExit(2)

    print("ok: diag script registry is up to date.")


if __name__ == "__main__":
    main()
