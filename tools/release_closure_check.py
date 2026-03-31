#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict, deque
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import tomllib


DEPENDENCY_SECTIONS = ("dependencies", "build-dependencies", "dev-dependencies")
# `cargo publish` validates dev-dependencies during package preparation, so first-wave
# publish order must account for them instead of only runtime/build edges.
ORDER_SECTIONS = DEPENDENCY_SECTIONS


@dataclass
class InternalDepIssue:
    crate: str
    dep: str
    section: str
    reason: str
    manifest: str


@dataclass(frozen=True, order=True)
class SemVer:
    major: int
    minor: int
    patch: int

    @classmethod
    def parse(cls, raw: str) -> "SemVer":
        # Cargo version strings may include pre-release/build metadata. We only need the numeric core.
        m = re.match(r"^\s*(\d+)\.(\d+)\.(\d+)", raw)
        if not m:
            raise ValueError(f"invalid semver: {raw!r}")
        return cls(int(m.group(1)), int(m.group(2)), int(m.group(3)))

    def compat_line(self) -> tuple[int, int]:
        # For 0.y.z, we treat y as the compatibility line (Rust community convention pre-1.0).
        return (self.major, self.minor)


@dataclass(frozen=True)
class VersionReq:
    kind: str  # "exact" | "caret"
    lower: SemVer
    upper: SemVer

    def allows(self, version: SemVer) -> bool:
        if self.kind == "exact":
            return version == self.lower
        return self.lower <= version < self.upper


def parse_cargo_version_req(raw: str) -> VersionReq:
    """
    Parse a minimal subset of Cargo version requirements, sufficient for internal workspace deps.

    Supported:
      - "0.2"    (caret, >=0.2.0,<0.3.0)
      - "0.2.1"  (caret, >=0.2.1,<0.3.0)
      - "^0.2.1" (caret)
      - "=0.2.1" (exact)

    Unsupported forms intentionally fail fast to keep manifests consistent:
      - ranges with commas/spaces (">=..., <..."), wildcards ("*"), tilde ("~"), etc.
    """
    s = raw.strip()
    if any(ch in s for ch in [",", " ", "<", ">", "*", "~"]):
        raise ValueError(f"unsupported Cargo version requirement: {raw!r} (use caret-style '0.y' or '0.y.z')")

    if s.startswith("="):
        v = SemVer.parse(s[1:])
        return VersionReq(kind="exact", lower=v, upper=v)

    if s.startswith("^"):
        s = s[1:]

    # Allow "0.2" shorthand.
    m = re.match(r"^(\d+)\.(\d+)(?:\.(\d+))?$", s)
    if not m:
        raise ValueError(f"unsupported Cargo version requirement: {raw!r} (use '0.y' or '0.y.z')")

    major = int(m.group(1))
    minor = int(m.group(2))
    patch = int(m.group(3) or 0)

    lower = SemVer(major, minor, patch)
    if major > 0:
        upper = SemVer(major + 1, 0, 0)
    elif minor > 0:
        upper = SemVer(0, minor + 1, 0)
    else:
        # 0.0.z: caret allows only patch bumps.
        upper = SemVer(0, 0, patch + 1)

    return VersionReq(kind="caret", lower=lower, upper=upper)


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def load_release_scope(config_path: Path) -> list[str]:
    data = tomllib.loads(config_path.read_text(encoding="utf-8"))
    packages = data.get("package", [])
    if not isinstance(packages, list):
        raise SystemExit(f"`package` in {config_path} is not a TOML array.")

    release: list[str] = []
    for item in packages:
        if not isinstance(item, dict):
            continue
        name = item.get("name")
        should_release = bool(item.get("release", False))
        should_publish = bool(item.get("publish", False))
        if isinstance(name, str) and should_release and should_publish:
            release.append(name)

    if not release:
        raise SystemExit(f"No releasable packages found in {config_path}.")
    return release


def cargo_metadata(root: Path) -> dict[str, Any]:
    output = subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=str(root),
        text=True,
    )
    return json.loads(output)


def iter_dep_tables(manifest: dict[str, Any], include_dev: bool) -> list[tuple[str, dict[str, Any]]]:
    result: list[tuple[str, dict[str, Any]]] = []
    sections = DEPENDENCY_SECTIONS if include_dev else ORDER_SECTIONS

    for section in sections:
        table = manifest.get(section)
        if isinstance(table, dict):
            result.append((section, table))

    target = manifest.get("target")
    if isinstance(target, dict):
        for target_name, target_cfg in target.items():
            if not isinstance(target_cfg, dict):
                continue
            for section in sections:
                table = target_cfg.get(section)
                if isinstance(table, dict):
                    result.append((f"target.{target_name}.{section}", table))

    return result


def collect_internal_issues(
    release_scope: list[str],
    manifests: dict[str, Path],
    workspace_names: set[str],
    workspace_versions: dict[str, SemVer],
) -> tuple[list[InternalDepIssue], dict[str, set[str]], list[str], list[str]]:
    release_set = set(release_scope)
    missing_release_crates = [name for name in release_scope if name not in manifests]
    missing_manifests: list[str] = []
    issues: list[InternalDepIssue] = []

    order_graph: dict[str, set[str]] = {name: set() for name in release_scope}

    for name in release_scope:
        manifest_path = manifests.get(name)
        if manifest_path is None:
            continue
        if not manifest_path.exists():
            missing_manifests.append(f"{name}: {manifest_path.as_posix()}")
            continue

        parsed = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
        dep_tables = iter_dep_tables(parsed, include_dev=True)

        for section, table in dep_tables:
            for dep_key, spec in table.items():
                if not isinstance(spec, dict):
                    continue
                if "path" not in spec:
                    continue

                dep_name = spec.get("package", dep_key)
                if not isinstance(dep_name, str):
                    continue

                if dep_name in workspace_names:
                    if "version" not in spec:
                        issues.append(
                            InternalDepIssue(
                                crate=name,
                                dep=dep_name,
                                section=section,
                                reason="internal path dependency misses explicit version",
                                manifest=manifest_path.resolve().as_posix(),
                            )
                        )
                    else:
                        dep_version = workspace_versions.get(dep_name)
                        raw_req = spec.get("version")
                        if dep_version is not None and isinstance(raw_req, str):
                            try:
                                req = parse_cargo_version_req(raw_req)
                            except ValueError as e:
                                issues.append(
                                    InternalDepIssue(
                                        crate=name,
                                        dep=dep_name,
                                        section=section,
                                        reason=str(e),
                                        manifest=manifest_path.resolve().as_posix(),
                                    )
                                )
                            else:
                                if not req.allows(dep_version):
                                    issues.append(
                                        InternalDepIssue(
                                            crate=name,
                                            dep=dep_name,
                                            section=section,
                                            reason=(
                                                "internal path dependency version requirement does not allow "
                                                f"workspace version (req={raw_req!r}, dep_version={dep_version.major}.{dep_version.minor}.{dep_version.patch})"
                                            ),
                                            manifest=manifest_path.resolve().as_posix(),
                                        )
                                    )
                        elif dep_version is not None and raw_req is not None and not isinstance(raw_req, str):
                            issues.append(
                                InternalDepIssue(
                                    crate=name,
                                    dep=dep_name,
                                    section=section,
                                    reason="internal path dependency has non-string version requirement",
                                    manifest=manifest_path.resolve().as_posix(),
                                )
                            )
                    if dep_name not in release_set:
                        issues.append(
                            InternalDepIssue(
                                crate=name,
                                dep=dep_name,
                                section=section,
                                reason="internal dependency not in release scope",
                                manifest=manifest_path.resolve().as_posix(),
                            )
                        )

        order_tables = iter_dep_tables(parsed, include_dev=False)
        for _, table in order_tables:
            for dep_key, spec in table.items():
                if isinstance(spec, dict):
                    dep_name = spec.get("package", dep_key)
                else:
                    dep_name = dep_key
                if isinstance(dep_name, str) and dep_name in release_set and dep_name != name:
                    order_graph[name].add(dep_name)

    return issues, order_graph, missing_release_crates, missing_manifests


def topo_sort(order_graph: dict[str, set[str]]) -> tuple[list[str], list[str]]:
    indegree = {name: 0 for name in order_graph}
    reverse_graph: dict[str, set[str]] = defaultdict(set)

    for name, deps in order_graph.items():
        for dep in deps:
            indegree[name] += 1
            reverse_graph[dep].add(name)

    queue = deque(sorted([name for name, deg in indegree.items() if deg == 0]))
    order: list[str] = []

    while queue:
        name = queue.popleft()
        order.append(name)
        for nxt in sorted(reverse_graph[name]):
            indegree[nxt] -= 1
            if indegree[nxt] == 0:
                queue.append(nxt)

    remaining = sorted([name for name, deg in indegree.items() if deg > 0])
    return order, remaining


def collect_metadata_warnings(release_scope: list[str], manifests: dict[str, Path]) -> list[str]:
    warnings: list[str] = []
    for name in release_scope:
        manifest_path = manifests.get(name)
        if manifest_path is None or not manifest_path.exists():
            continue
        parsed = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
        package = parsed.get("package")
        if not isinstance(package, dict):
            warnings.append(f"{name}: missing [package] table")
            continue

        if "description" not in package:
            warnings.append(f"{name}: missing package.description")
        if "license" not in package and "license-file" not in package:
            warnings.append(f"{name}: missing package.license or package.license-file")
        if "repository" not in package:
            warnings.append(f"{name}: missing package.repository")

    return warnings


def write_order_file(order: list[str], path: Path, include_commands: bool) -> None:
    lines = ["# Release publish order", ""]
    for idx, crate in enumerate(order, start=1):
        lines.append(f"{idx:02d}. {crate}")

    if include_commands:
        lines.append("")
        lines.append("# Commands")
        for crate in order:
            lines.append(f"cargo publish -p {crate}")

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check release-plz publish scope closure and print deterministic publish order."
    )
    parser.add_argument(
        "--config",
        default="release-plz.toml",
        help="Path to release-plz config file (default: release-plz.toml).",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Print machine-readable JSON summary.",
    )
    parser.add_argument(
        "--strict-metadata",
        action="store_true",
        help="Fail when package metadata warnings are found.",
    )
    parser.add_argument(
        "--write-order",
        help="Write computed publish order to a file.",
    )
    parser.add_argument(
        "--print-publish-commands",
        action="store_true",
        help="Print `cargo publish -p <crate>` commands in publish order.",
    )
    args = parser.parse_args()

    root = repo_root()
    config_path = (root / args.config).resolve()
    if not config_path.exists():
        raise SystemExit(f"Config file does not exist: {config_path}")

    release_scope = load_release_scope(config_path)
    metadata = cargo_metadata(root)
    package_entries = metadata.get("packages", [])

    manifests: dict[str, Path] = {}
    workspace_names: set[str] = set()
    workspace_versions: dict[str, SemVer] = {}
    for item in package_entries:
        name = item.get("name")
        manifest_path = item.get("manifest_path")
        version = item.get("version")
        if isinstance(name, str):
            workspace_names.add(name)
            if isinstance(manifest_path, str):
                manifests[name] = Path(manifest_path)
            if isinstance(version, str):
                try:
                    workspace_versions[name] = SemVer.parse(version)
                except ValueError:
                    # Defer reporting to the release-scope version line check below.
                    pass

    issues, order_graph, missing_release_crates, missing_manifests = collect_internal_issues(
        release_scope=release_scope,
        manifests=manifests,
        workspace_names=workspace_names,
        workspace_versions=workspace_versions,
    )
    order, cycle_nodes = topo_sort(order_graph)
    metadata_warnings = collect_metadata_warnings(release_scope, manifests)

    # Version-line guard: all releasable crates must share the same (major, minor) compatibility line.
    compat_lines: dict[tuple[int, int], list[str]] = defaultdict(list)
    version_line_issues: list[str] = []
    for name in release_scope:
        v = workspace_versions.get(name)
        if v is None:
            version_line_issues.append(f"{name}: missing or unparsable version in cargo metadata")
        else:
            compat_lines[v.compat_line()].append(name)

    compat_line_keys = sorted(compat_lines.keys())
    if len(compat_line_keys) > 1:
        parts = []
        for key in compat_line_keys:
            crates = ", ".join(sorted(compat_lines[key]))
            parts.append(f"{key[0]}.{key[1]}: [{crates}]")
        version_line_issues.append(
            "release scope spans multiple compatibility lines (expected a single 0.y line): " + " ; ".join(parts)
        )

    summary = {
        "release_scope_count": len(release_scope),
        "release_scope": release_scope,
        "missing_release_crates": missing_release_crates,
        "missing_manifests": missing_manifests,
        "issue_count": len(issues),
        "issues": [issue.__dict__ for issue in issues],
        "version_line_issue_count": len(version_line_issues),
        "version_line_issues": version_line_issues,
        "compat_line_keys": [f"{k[0]}.{k[1]}" for k in compat_line_keys],
        "publish_order_count": len(order),
        "publish_order": order,
        "cycle_nodes": cycle_nodes,
        "metadata_warning_count": len(metadata_warnings),
        "metadata_warnings": metadata_warnings,
    }

    if args.write_order:
        order_path = (root / args.write_order).resolve()
        write_order_file(order, order_path, include_commands=args.print_publish_commands)

    if args.json:
        print(json.dumps(summary, indent=2))
    else:
        print(f"[release-closure] release scope: {len(release_scope)} crates")
        if missing_release_crates:
            print(f"[release-closure] missing workspace crates: {len(missing_release_crates)}")
            for name in missing_release_crates:
                print(f"  - {name}")

        if missing_manifests:
            print(f"[release-closure] missing manifest paths: {len(missing_manifests)}")
            for item in missing_manifests:
                print(f"  - {item}")

        print(f"[release-closure] internal dependency issues: {len(issues)}")
        for issue in issues:
            print(
                f"  - {issue.crate} -> {issue.dep} ({issue.section}): {issue.reason}"
            )

        print(f"[release-closure] version line issues: {len(version_line_issues)}")
        for issue in version_line_issues:
            print(f"  - {issue}")

        if cycle_nodes:
            print(f"[release-closure] cycle nodes: {', '.join(cycle_nodes)}")
        else:
            print(f"[release-closure] publish order: {len(order)} crates")
            for idx, crate in enumerate(order, start=1):
                print(f"  {idx:02d}. {crate}")

        if args.print_publish_commands:
            print("[release-closure] publish commands")
            for crate in order:
                print(f"  cargo publish -p {crate}")

        print(f"[release-closure] metadata warnings: {len(metadata_warnings)}")
        for warning in metadata_warnings:
            print(f"  - {warning}")

    has_blocking_errors = bool(
        missing_release_crates or missing_manifests or issues or cycle_nodes or version_line_issues
    )
    has_metadata_errors = args.strict_metadata and bool(metadata_warnings)

    return 1 if has_blocking_errors or has_metadata_errors else 0


if __name__ == "__main__":
    sys.exit(main())
