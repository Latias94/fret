#!/usr/bin/env python3
"""
Crate audit snapshot (cross-platform).

This is a cross-platform replacement for `tools/audit_crate.ps1`.

The goal is not to be "perfect"; it's to provide a fast, evidence-friendly snapshot:
- crate identity + location
- top Rust files by line count
- a minimal public-surface scan (lib.rs pub mod / pub use)
- direct dependencies (workspace vs external)
- kernel forbidden deps spot check (name patterns)
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


KERNEL_CRATES = {"fret-core", "fret-runtime", "fret-app", "fret-ui"}
FORBIDDEN_DEP_PATTERNS = [
    r"^winit($|[-_])",
    r"^wgpu($|[-_])",
    r"^web-sys$",
    r"^js-sys$",
    r"^wasm-bindgen($|[-_])",
    r"^tokio($|[-_])",
    r"^reqwest($|[-_])",
]


def _die(msg: str, code: int = 1) -> "NoReturn":
    print(f"[ERROR] {msg}", file=sys.stderr)
    raise SystemExit(code)


def _run_json(cmd: list[str]) -> object:
    try:
        out = subprocess.check_output(cmd, text=True)
    except FileNotFoundError:
        _die(f"Missing executable: {cmd[0]}")
    except subprocess.CalledProcessError as e:
        _die(f"Command failed ({e.returncode}): {' '.join(cmd)}")
    try:
        return json.loads(out)
    except Exception:
        _die(f"Failed to parse JSON from: {' '.join(cmd)}")


def _line_count(path: Path) -> int:
    try:
        with path.open("rb") as f:
            return sum(1 for _ in f)
    except OSError:
        return 0


def _matches_any(value: str, patterns: list[str]) -> bool:
    return any(re.search(p, value) for p in patterns)


@dataclass(frozen=True)
class AuditRow:
    path: str
    lines: int


def _print_table(rows: list[AuditRow]) -> None:
    if not rows:
        print("  (none)")
        return
    width = max(len(r.path) for r in rows)
    for r in rows:
        print(f"  {r.path.ljust(width)}  {r.lines}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--crate", required=True, help="Cargo package name (e.g. fret-runtime)")
    parser.add_argument("--top-files", type=int, default=12)
    parser.add_argument("--min-lines", type=int, default=200)
    parser.add_argument("--enforce-kernel-forbidden-deps", action="store_true")
    parser.add_argument(
        "--no-locked",
        action="store_true",
        help="Do not pass --locked to cargo metadata (may update Cargo.lock).",
    )
    args = parser.parse_args(argv)

    metadata_cmd = ["cargo", "metadata", "--format-version", "1"]
    if not args.no_locked:
        metadata_cmd.append("--locked")
    metadata = _run_json(metadata_cmd)
    if not isinstance(metadata, dict):
        _die("Unexpected cargo metadata shape")

    packages = metadata.get("packages", [])
    if not isinstance(packages, list):
        _die("Unexpected cargo metadata.packages shape")

    pkg = next((p for p in packages if isinstance(p, dict) and p.get("name") == args.crate), None)
    if not pkg:
        _die(f"crate not found in cargo metadata: {args.crate}", code=2)

    manifest_path = Path(str(pkg.get("manifest_path", "")))
    crate_dir = manifest_path.parent
    src_dir = crate_dir / "src"

    print(f"crate: {pkg.get('name')}")
    print(f"version: {pkg.get('version')}")
    print(f"manifest: {manifest_path}")
    print(f"dir: {crate_dir}")

    print()
    print("top files (src/, by lines):")
    if src_dir.is_dir():
        rows: list[AuditRow] = []
        for path in sorted(src_dir.rglob("*.rs")):
            rel = path.relative_to(crate_dir).as_posix()
            rows.append(AuditRow(path=rel, lines=_line_count(path)))
        rows = [r for r in rows if r.lines >= args.min_lines]
        rows.sort(key=lambda r: r.lines, reverse=True)
        _print_table(rows[: args.top_files])
    else:
        print("  (no src/ directory)")

    print()
    print("public surface (src/lib.rs quick scan):")
    lib_rs = src_dir / "lib.rs"
    if lib_rs.is_file():
        try:
            lib = lib_rs.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            lib = []
        pub_use = sum(1 for line in lib if re.search(r"^\s*pub\s+use\s+", line))
        pub_mod = sum(1 for line in lib if re.search(r"^\s*pub\s+mod\s+", line))
        print(f"  pub mod: {pub_mod}")
        print(f"  pub use: {pub_use}")
    else:
        print("  (no src/lib.rs)")

    print()
    print("dependencies (direct, from cargo metadata):")

    deps = pkg.get("dependencies", [])
    if not isinstance(deps, list):
        deps = []

    direct = []
    for d in deps:
        if not isinstance(d, dict):
            continue
        kind = d.get("kind")
        if kind in (None, "normal"):
            name = d.get("name")
            if isinstance(name, str) and name:
                direct.append(name)

    workspace_member_ids = set(metadata.get("workspace_members", []) or [])
    workspace_names: set[str] = set()
    for p in packages:
        if not isinstance(p, dict):
            continue
        if p.get("id") in workspace_member_ids:
            n = p.get("name")
            if isinstance(n, str) and n:
                workspace_names.add(n)

    workspace_direct = sorted({d for d in direct if d in workspace_names})
    external_direct = sorted({d for d in direct if d not in workspace_names})

    print("  workspace:")
    if not workspace_direct:
        print("    (none)")
    else:
        for n in workspace_direct:
            print(f"    - {n}")

    print("  external:")
    if not external_direct:
        print("    (none)")
    else:
        for n in external_direct:
            print(f"    - {n}")

    print()
    print("kernel forbidden deps spot check (name patterns):")
    if args.crate in KERNEL_CRATES:
        hits = [n for n in external_direct if _matches_any(n, FORBIDDEN_DEP_PATTERNS)]
        if not hits:
            print("  ok (no obvious forbidden deps)")
        else:
            print("  potential violations:")
            for n in hits:
                print(f"    - {n}")
            if args.enforce_kernel_forbidden_deps:
                return 3
    else:
        print("  (skipped: not a kernel crate)")

    print()
    print("suggested gates (fast):")
    print("  - cargo fmt")
    print(f"  - cargo nextest run -p {args.crate}")
    print("  - python3 tools/check_layering.py")

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
