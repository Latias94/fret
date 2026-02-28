#!/usr/bin/env python3
"""
Portability gate for `fret-launch-core`.

Goal:
- Keep `crates/fret-launch-core` platform-SDK-free (no direct deps on OS/browser SDK crates).
- Prevent accidental leakage of platform-specific workspace crates into the shared launcher surface.

This check is intentionally strict-but-local:
- It inspects direct Cargo.toml dependency keys (including target-specific tables).
- It does not attempt to validate transitive deps (e.g. `winit` may pull platform crates behind cfgs).

Intended usage:
  python3 tools/check_launch_core_portability.py
"""

from __future__ import annotations

import sys
from pathlib import Path


def _load_toml(path: Path) -> dict:
    import tomllib  # Python 3.11+

    data = tomllib.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise SystemExit(f"Invalid TOML root: {path}")
    return data


def _iter_dependency_tables(doc: dict) -> list[tuple[str, dict]]:
    """
    Return (label, table) pairs for all dependency tables that can declare crate keys.
    """
    out: list[tuple[str, dict]] = []

    for key in ("dependencies", "dev-dependencies", "build-dependencies"):
        table = doc.get(key)
        if isinstance(table, dict):
            out.append((key, table))

    target = doc.get("target")
    if isinstance(target, dict):
        for target_key, target_table in target.items():
            if not isinstance(target_table, dict):
                continue
            for dep_key in ("dependencies", "dev-dependencies", "build-dependencies"):
                deps = target_table.get(dep_key)
                if isinstance(deps, dict):
                    out.append((f"target.{target_key}.{dep_key}", deps))

    return out


def _is_forbidden_dep(name: str) -> bool:
    # External platform SDK crates (direct deps are forbidden in launch-core).
    forbidden_exact = {
        "web-sys",
        "js-sys",
        "wasm-bindgen",
        "wasm-bindgen-futures",
        "objc",
        "windows",
        "windows-sys",
        "zbus",
        "block2",
    }

    # Internal workspace crates that are platform/backend specific (should not be depended on here).
    forbidden_workspace = {
        "fret-platform-native",
        "fret-platform-web",
        "fret-runner-winit",
        "fret-runner-web",
        "fret-render-wgpu",
        "fret-launch-desktop",
        "fret-launch-web",
    }

    if name in forbidden_exact or name in forbidden_workspace:
        return True

    # Prefix-based families.
    forbidden_prefixes = (
        "objc2",
        "windows-",
    )
    return name.startswith(forbidden_prefixes)


def main(argv: list[str]) -> int:
    if argv:
        raise SystemExit("This script takes no arguments.")

    cargo_toml = Path("crates/fret-launch-core/Cargo.toml")
    if not cargo_toml.exists():
        raise SystemExit(f"Missing {cargo_toml}")

    doc = _load_toml(cargo_toml)

    failures: list[str] = []
    for label, deps in _iter_dependency_tables(doc):
        for dep_name in deps.keys():
            if _is_forbidden_dep(dep_name):
                failures.append(f"- forbidden direct dep `{dep_name}` in [{label}]")

    if failures:
        msg = "\n".join(
            ["[launch-core-portability] FAILED (platform-SDK-free contract violated):", *failures]
        )
        raise SystemExit(msg)

    print("[launch-core-portability] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

