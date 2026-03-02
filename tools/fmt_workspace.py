#!/usr/bin/env python3
"""
Windows-friendly workspace formatter.

Why this exists:
- `cargo fmt --all` can hit Windows `os error 206` (command line too long) when it tries to pass a very
  large file list to `rustfmt` in a single process invocation.

What this script does:
- Formats per-package (`cargo fmt -p <pkg>`) to keep argument lists small.
- If a package still trips `os error 206`, it falls back to calling `rustfmt` directly in small chunks.

Intended usage:
    python3 tools/fmt_workspace.py
    python3 tools/fmt_workspace.py --changed
    python3 tools/fmt_workspace.py --check
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


def _die(msg: str, code: int = 1) -> "NoReturn":
    print(f"[ERROR] {msg}", file=sys.stderr)
    raise SystemExit(code)


def _run(cmd: list[str], *, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=str(cwd) if cwd is not None else None,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        encoding="utf-8",
        errors="replace",
        text=True,
    )


def _run_json(cmd: list[str]) -> object:
    proc = _run(cmd)
    if proc.returncode != 0:
        _die(proc.stderr.strip() or f"command failed: {' '.join(cmd)}")
    try:
        return json.loads(proc.stdout)
    except Exception:
        _die(f"failed to parse JSON from: {' '.join(cmd)}")


def _is_win_cmdline_too_long(stderr: str) -> bool:
    # Localized + English variants seen in the wild.
    s = stderr.lower()
    return (
        "os error 206" in s
        or "the filename or extension is too long" in s
        or "文件名或扩展名太长" in stderr
    )


def _long_path(p: Path) -> str:
    # Keep parity with cargo/rustfmt behavior on Windows: use extended-length paths.
    s = str(p.resolve())
    if os.name != "nt":
        return s
    if s.startswith("\\\\?\\"):
        return s
    if s.startswith("\\\\"):
        # UNC path: \\server\share\... -> \\?\UNC\server\share\...
        return "\\\\?\\UNC\\" + s.lstrip("\\")
    return "\\\\?\\" + s


def _find_rustfmt() -> str:
    proc = _run(["rustfmt", "--version"])
    if proc.returncode == 0:
        return "rustfmt"
    proc = _run(["rustup", "which", "rustfmt"])
    if proc.returncode != 0:
        _die("rustfmt not found (try `rustup component add rustfmt`)")
    return proc.stdout.strip()


def _iter_rs_files(crate_dir: Path) -> Iterable[Path]:
    # Include untracked files; skip target-like directories.
    skip_names = {"target", ".git", ".hg", ".svn", "node_modules"}
    for root, dirs, files in os.walk(crate_dir):
        dirs[:] = [d for d in dirs if d not in skip_names]
        for name in files:
            if name.endswith(".rs"):
                yield Path(root) / name


def _chunked(items: list[str], chunk_size: int) -> Iterable[list[str]]:
    for i in range(0, len(items), chunk_size):
        yield items[i : i + chunk_size]


@dataclass(frozen=True)
class Package:
    name: str
    manifest_path: Path
    edition: str

    @property
    def dir(self) -> Path:
        return self.manifest_path.parent


def _load_workspace_packages(*, locked: bool) -> list[Package]:
    cmd = ["cargo", "metadata", "--format-version", "1", "--no-deps"]
    if locked:
        cmd.append("--locked")
    metadata = _run_json(cmd)
    if not isinstance(metadata, dict):
        _die("unexpected cargo metadata shape")

    workspace_ids = set(metadata.get("workspace_members", []))
    pkgs = metadata.get("packages", [])
    if not isinstance(pkgs, list):
        _die("unexpected cargo metadata.packages shape")

    out: list[Package] = []
    for p in pkgs:
        if not isinstance(p, dict):
            continue
        if p.get("id") not in workspace_ids:
            continue
        name = str(p.get("name", ""))
        manifest = Path(str(p.get("manifest_path", "")))
        edition = str(p.get("edition", "2024"))
        if not name or not manifest.exists():
            continue
        out.append(Package(name=name, manifest_path=manifest, edition=edition))

    out.sort(key=lambda x: x.name)
    return out


def _changed_paths() -> set[str]:
    # Includes staged, unstaged, and untracked.
    paths: set[str] = set()

    for cmd in (["git", "diff", "--name-only"], ["git", "diff", "--name-only", "--cached"]):
        proc = _run(cmd)
        if proc.returncode != 0:
            continue
        for line in proc.stdout.splitlines():
            line = line.strip()
            if line:
                paths.add(line)

    proc = _run(["git", "status", "--porcelain"])
    if proc.returncode == 0:
        for raw in proc.stdout.splitlines():
            line = raw.strip()
            if not line:
                continue
            # Examples:
            #   " M path"
            #   "A  path"
            #   "R  old -> new"
            #   "?? path"
            payload = line[2:].strip() if len(line) >= 3 else ""
            if "->" in payload:
                payload = payload.split("->", 1)[1].strip()
            if payload:
                paths.add(payload)

    return paths


def _pick_changed_packages(packages: list[Package]) -> list[Package]:
    root = Path.cwd().resolve()
    changed = _changed_paths()
    if not changed:
        return []

    picked: list[Package] = []
    for pkg in packages:
        pkg_dir = pkg.dir.resolve()
        # Normalize to forward slashes for prefix checks.
        pkg_rel = str(pkg_dir.relative_to(root)).replace("\\", "/").rstrip("/") + "/"
        hit = False
        for p in changed:
            pp = p.replace("\\", "/")
            if pp.startswith(pkg_rel) and (pp.endswith(".rs") or pp.endswith("Cargo.toml")):
                hit = True
                break
        if hit:
            picked.append(pkg)
    return picked


def _cargo_fmt(pkg: Package, *, check: bool) -> subprocess.CompletedProcess[str]:
    cmd = ["cargo", "fmt", "-p", pkg.name]
    if check:
        cmd += ["--", "--check"]
    return _run(cmd)


def _rustfmt_fallback(pkg: Package, *, check: bool, chunk_size: int) -> int:
    rustfmt = _find_rustfmt()
    rs_files = sorted({_long_path(p) for p in _iter_rs_files(pkg.dir)})
    if not rs_files:
        return 0

    common = [rustfmt, "--edition", pkg.edition]
    if check:
        common.append("--check")

    for chunk in _chunked(rs_files, chunk_size=chunk_size):
        proc = _run(common + chunk, cwd=pkg.dir)
        if proc.returncode != 0:
            sys.stderr.write(proc.stderr)
            return proc.returncode
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--changed",
        action="store_true",
        help="Only format workspace packages touched by git changes (staged/unstaged/untracked).",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Run rustfmt in check mode (no files are modified).",
    )
    parser.add_argument(
        "--package",
        action="append",
        default=[],
        help="Format only these packages (repeatable).",
    )
    parser.add_argument(
        "--exclude",
        action="append",
        default=[],
        help="Exclude these packages (repeatable).",
    )
    parser.add_argument(
        "--no-locked",
        action="store_true",
        help="Do not pass --locked to cargo metadata (may update Cargo.lock).",
    )
    parser.add_argument(
        "--fallback-chunk-size",
        type=int,
        default=64,
        help="Files per rustfmt invocation when falling back (lower reduces cmdline pressure).",
    )
    args = parser.parse_args(argv)

    packages = _load_workspace_packages(locked=not args.no_locked)

    if args.package:
        want = set(args.package)
        packages = [p for p in packages if p.name in want]

    if args.exclude:
        skip = set(args.exclude)
        packages = [p for p in packages if p.name not in skip]

    if args.changed:
        packages = _pick_changed_packages(packages)

    if not packages:
        print("[fmt] no packages selected")
        return 0

    total = len(packages)
    for i, pkg in enumerate(packages, start=1):
        print(f"[fmt] ({i}/{total}) {pkg.name}")
        proc = _cargo_fmt(pkg, check=args.check)
        if proc.returncode == 0:
            continue

        sys.stderr.write(proc.stderr)
        if _is_win_cmdline_too_long(proc.stderr):
            print(f"[fmt] {pkg.name}: falling back to chunked rustfmt", file=sys.stderr)
            rc = _rustfmt_fallback(pkg, check=args.check, chunk_size=args.fallback_chunk_size)
            if rc != 0:
                return rc
            continue

        return proc.returncode

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

