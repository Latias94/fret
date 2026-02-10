#!/usr/bin/env python3
"""
Verify generated icon artifacts are idempotent.

Ported from `tools/check_icons_generation.ps1`.

This script regenerates icon lists/ids for the selected pack(s), optionally syncs assets and
verifies referenced vendor ids, then checks that generated files did not change.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _sha256_or_missing(path: Path) -> str:
    if not path.exists():
        return "<missing>"
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def _invoke_checked(name: str, argv: list[str]) -> None:
    print(f"[check-icons] {name}")
    proc = subprocess.run(argv, check=False)
    if proc.returncode != 0:
        raise RuntimeError(f"Step failed: {name} (exit code: {proc.returncode})")


def _python_cmd() -> list[str]:
    # Prefer the current interpreter to keep venv/CI consistent.
    return [sys.executable]


def _pack_generated_files(pack: str) -> list[Path]:
    if pack == "lucide":
        return [
            Path("ecosystem/fret-icons-lucide/icon-list.txt"),
            Path("ecosystem/fret-icons-lucide/src/generated_ids.rs"),
        ]
    if pack == "radix":
        return [
            Path("ecosystem/fret-icons-radix/icon-list.txt"),
            Path("ecosystem/fret-icons-radix/src/generated_ids.rs"),
        ]
    raise ValueError(f"Unsupported pack: {pack}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pack", choices=("lucide", "radix", "all"), default="all")
    parser.add_argument("--skip-sync", action="store_true")
    parser.add_argument("--skip-verify", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    packs: list[str] = []
    if args.pack in ("lucide", "all"):
        packs.append("lucide")
    if args.pack in ("radix", "all"):
        packs.append("radix")

    generated_files: list[Path] = []
    for pack in packs:
        generated_files.extend(_pack_generated_files(pack))

    # Ensure vendor sources exist (these are typically submodules under `third_party/`).
    lucide_source = repo_root / "third_party" / "lucide" / "icons"
    radix_source = (
        repo_root
        / "third_party"
        / "radix-icons"
        / "packages"
        / "radix-icons"
        / "icons"
    )
    if "lucide" in packs and not lucide_source.exists():
        print(
            f"error: Lucide source directory not found: {lucide_source}\n"
            "hint: initialize submodules/vendor assets (e.g. `git submodule update --init --recursive`).",
            file=sys.stderr,
        )
        return 2
    if "radix" in packs and not radix_source.exists():
        print(
            f"error: Radix source directory not found: {radix_source}\n"
            "hint: initialize submodules/vendor assets (e.g. `git submodule update --init --recursive`).",
            file=sys.stderr,
        )
        return 2

    before = {p: _sha256_or_missing(repo_root / p) for p in generated_files}

    try:
        py = _python_cmd()
        _invoke_checked(
            f"generate icon-list and generated_ids (pack={args.pack})",
            py + [str(repo_root / "tools/gen_icons.py"), "--pack", args.pack],
        )

        if not args.skip_sync:
            for pack in packs:
                _invoke_checked(
                    f"sync {pack} assets",
                    py
                    + [
                        str(repo_root / "tools/sync_icons.py"),
                        "--pack",
                        pack,
                        "--clean",
                    ],
                )

        if not args.skip_verify:
            _invoke_checked(
                "verify referenced vendor ids",
                py + [str(repo_root / "tools/verify_icons.py"), "--strict"],
            )
    except RuntimeError as e:
        print(f"error: {e}", file=sys.stderr)
        return 1

    changed: list[str] = []
    for p in generated_files:
        after = _sha256_or_missing(repo_root / p)
        if before[p] != after:
            changed.append(p.as_posix())

    if changed:
        print("[check-icons] generated files changed after regeneration:", file=sys.stderr)
        for p in changed:
            print(f"  - {p}", file=sys.stderr)
        print(
            "Generated icon artifacts are not idempotent. Re-run generation and commit updated outputs.",
            file=sys.stderr,
        )
        return 1

    print("[check-icons] done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
