#!/usr/bin/env python3
"""
Workspace crate boundary checks (ADR 0037 / docs/dependency-policy.md).

This is a cross-platform replacement for `tools/check_layering.ps1`.
It validates:

- Core portability boundaries (no backend leakage into portable crates).
- Core vs ecosystem vs apps dependency direction.
- Feature allowlist for `fret-ui/unstable-retained-bridge`.

Intended usage:
    python3 tools/check_layering.py
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from typing import Any, Iterable


@dataclass(frozen=True)
class Violation:
    rule: str
    message: str


def _matches_any_pattern(value: str, patterns: list[re.Pattern[str]]) -> bool:
    return any(p.search(value) is not None for p in patterns)


def _run_cargo_metadata() -> dict[str, Any]:
    proc = subprocess.run(
        ["cargo", "metadata", "--format-version", "1"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or "cargo metadata failed")
    return json.loads(proc.stdout)


def _workspace_name_sets(metadata: dict[str, Any]) -> tuple[set[str], set[str]]:
    workspace_ids = set(metadata["workspace_members"])

    ecosystem = set()
    kernel = set()
    for pkg in metadata["packages"]:
        if pkg["id"] not in workspace_ids:
            continue
        manifest = str(pkg.get("manifest_path", ""))
        name = str(pkg.get("name", ""))
        if "/ecosystem/" in manifest or "\\ecosystem\\" in manifest:
            ecosystem.add(name)
        if "/crates/" in manifest or "\\crates\\" in manifest:
            kernel.add(name)
    return ecosystem, kernel


def _deps_by_from(metadata: dict[str, Any]) -> dict[str, list[str]]:
    workspace_ids = set(metadata["workspace_members"])
    id_to_name = {}
    for pkg in metadata["packages"]:
        id_to_name[pkg["id"]] = pkg["name"]

    out: dict[str, list[str]] = {}
    for node in metadata["resolve"]["nodes"]:
        if node["id"] not in workspace_ids:
            continue
        from_name = id_to_name[node["id"]]
        out.setdefault(from_name, [])
        for dep in node.get("deps", []):
            to_id = dep.get("pkg")
            if to_id in workspace_ids:
                out[from_name].append(id_to_name[to_id])
    return out


def _assert_no_workspace_deps_matching(
    deps_by_from: dict[str, list[str]],
    *,
    from_name: str,
    rule: str,
    forbidden_patterns: list[re.Pattern[str]],
    violations: list[Violation],
) -> None:
    for to in deps_by_from.get(from_name, []):
        if _matches_any_pattern(to, forbidden_patterns):
            violations.append(
                Violation(
                    rule=rule,
                    message=f"Layering violation ({rule}): {from_name} must not depend on {to}",
                )
            )


def _assert_no_external_deps(
    name_to_pkg: dict[str, dict[str, Any]],
    *,
    crate: str,
    rule: str,
    forbidden_dep_names: set[str],
    violations: list[Violation],
) -> None:
    pkg = name_to_pkg.get(crate)
    if not pkg:
        return

    for dep in pkg.get("dependencies", []):
        kind = dep.get("kind")
        if kind is not None and kind != "normal":
            continue
        dep_name = dep.get("name")
        if dep_name in forbidden_dep_names:
            violations.append(
                Violation(
                    rule=rule,
                    message=(
                        f"Layering violation ({rule}): {crate} must not depend on external crate {dep_name}"
                    ),
                )
            )


def _check_unstable_retained_bridge_allowlist(
    metadata: dict[str, Any],
    *,
    allowlist: set[str],
    violations: list[Violation],
) -> None:
    workspace_ids = set(metadata["workspace_members"])
    for pkg in metadata["packages"]:
        if pkg["id"] not in workspace_ids:
            continue
        from_name = pkg["name"]
        for dep in pkg.get("dependencies", []):
            if dep.get("name") != "fret-ui":
                continue
            features = dep.get("features") or []
            if "unstable-retained-bridge" in features and from_name not in allowlist:
                violations.append(
                    Violation(
                        rule="unstable-retained-bridge-allowlist",
                        message=(
                            "Layering violation (unstable-retained-bridge-allowlist): "
                            f"{from_name} must not enable fret-ui/unstable-retained-bridge"
                        ),
                    )
                )


def main(argv: list[str]) -> int:
    _ = argv

    try:
        metadata = _run_cargo_metadata()
    except Exception as e:
        print(f"error: {e}", file=sys.stderr)
        return 2

    workspace_ids = set(metadata["workspace_members"])
    name_to_pkg = {pkg["name"]: pkg for pkg in metadata["packages"] if pkg["id"] in workspace_ids}
    ecosystem_names, kernel_names = _workspace_name_sets(metadata)
    deps_by_from = _deps_by_from(metadata)

    violations: list[Violation] = []

    platform_patterns = [re.compile(r"^fret-platform($|-)")]
    renderer_patterns = [re.compile(r"^fret-render($|-)")]
    runner_patterns = [re.compile(r"^fret-runner($|-)")]
    components_patterns = [re.compile(r"^fret-components-")]

    ecosystem_allow_backend_deps = {"fret-bootstrap"}

    # 1) `fret-core` must not depend on any other workspace crate.
    for to in deps_by_from.get("fret-core", []):
        violations.append(
            Violation(
                rule="core-is-leaf",
                message=f"Layering violation (core-is-leaf): fret-core must not depend on {to}",
            )
        )

    # 2) Runtime + UI substrate must not depend on backend crates.
    forbidden_backends = platform_patterns + renderer_patterns + runner_patterns
    for from_name in ("fret-runtime", "fret-app", "fret-ui"):
        _assert_no_workspace_deps_matching(
            deps_by_from,
            from_name=from_name,
            rule="portable-no-backends",
            forbidden_patterns=forbidden_backends,
            violations=violations,
        )

    # 3) Component crates must not depend on platform/render/runner crates.
    for from_name in deps_by_from.keys():
        if not _matches_any_pattern(from_name, components_patterns):
            continue
        _assert_no_workspace_deps_matching(
            deps_by_from,
            from_name=from_name,
            rule="components-no-backends",
            forbidden_patterns=forbidden_backends,
            violations=violations,
        )

    # 3.5) Ecosystem crates must not depend on platform/render/runner crates (unless explicitly allowlisted).
    for from_name in deps_by_from.keys():
        if from_name not in ecosystem_names:
            continue
        if from_name in ecosystem_allow_backend_deps:
            continue
        _assert_no_workspace_deps_matching(
            deps_by_from,
            from_name=from_name,
            rule="ecosystem-no-backends",
            forbidden_patterns=forbidden_backends,
            violations=violations,
        )

    # 3.75) Kernel (`crates/*`) must not depend on ecosystem crates.
    for from_name, tos in deps_by_from.items():
        if from_name not in kernel_names:
            continue
        for to in tos:
            if to in ecosystem_names:
                violations.append(
                    Violation(
                        rule="kernel-no-ecosystem",
                        message=f"Layering violation (kernel-no-ecosystem): {from_name} must not depend on {to}",
                    )
                )

    # 4) Backend crates must not depend on UI/component crates.
    backend_froms = (
        "fret-render",
        "fret-render-core",
        "fret-render-wgpu",
        "fret-platform",
        "fret-platform-native",
        "fret-platform-web",
    )
    for from_name in backend_froms:
        for to in deps_by_from.get(from_name, []):
            if to == "fret-ui" or _matches_any_pattern(to, components_patterns) or to in ecosystem_names:
                violations.append(
                    Violation(
                        rule="backends-no-ui",
                        message=f"Layering violation (backends-no-ui): {from_name} must not depend on {to}",
                    )
                )

    # External dependency checks for portable crates (cheap sanity guards).
    forbidden_in_portable = {
        "wgpu",
        "winit",
        "taffy",
        "accesskit",
        "accesskit_winit",
        "cosmic-text",
        "lyon",
        "resvg",
        "usvg",
        "arboard",
        "rfd",
        "webbrowser",
    }
    _assert_no_external_deps(
        name_to_pkg,
        crate="fret-core",
        rule="core-portable-deps",
        forbidden_dep_names=forbidden_in_portable,
        violations=violations,
    )
    _assert_no_external_deps(
        name_to_pkg,
        crate="fret-runtime",
        rule="runtime-portable-deps",
        forbidden_dep_names=forbidden_in_portable,
        violations=violations,
    )
    _assert_no_external_deps(
        name_to_pkg,
        crate="fret-app",
        rule="app-portable-deps",
        forbidden_dep_names=forbidden_in_portable,
        violations=violations,
    )
    _assert_no_external_deps(
        name_to_pkg,
        crate="fret-platform",
        rule="platform-contracts-portable-deps",
        forbidden_dep_names=forbidden_in_portable,
        violations=violations,
    )

    # Feature usage policy: retained bridge must remain explicitly opt-in and tightly scoped.
    unstable_retained_bridge_allowlist = {
        "fret-chart",
        "fret-docking",
        "fret-node",
        "fret-plot",
        "fret-plot3d",
    }
    _check_unstable_retained_bridge_allowlist(
        metadata,
        allowlist=unstable_retained_bridge_allowlist,
        violations=violations,
    )

    if violations:
        for v in violations:
            print(v.message, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

