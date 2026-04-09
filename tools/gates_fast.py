#!/usr/bin/env python3
"""Fast gate runner (cross-platform, canonical entrypoint)."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _run_checked(name: str, argv: list[str]) -> None:
    print(f"[gates-fast] {name}")
    proc = subprocess.run(argv, check=False)
    if proc.returncode != 0:
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-layering", action="store_true")
    parser.add_argument("--skip-fmt", action="store_true")
    parser.add_argument("--skip-nextest", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    py = sys.executable

    if not args.skip_layering:
        _run_checked(
            "Workspace layering policy",
            [py, str(repo_root / "tools/check_layering.py")],
        )
        _run_checked(
            "Workstream catalog integrity",
            [py, str(repo_root / "tools/check_workstream_catalog.py")],
        )

    if not args.skip_fmt:
        _run_checked("cargo fmt --check", ["cargo", "fmt", "--all", "--", "--check"])

    if not args.skip_nextest:
        packages = [
            "fret-core",
            "fret-runtime",
            "fret-ui",
            "fret-ui-kit",
            "fret-runner-winit",
            "fret-ui-shadcn",
        ]
        if shutil.which("cargo-nextest") is None:
            print(
                "[gates-fast] warning: cargo-nextest is not installed; falling back to "
                "cargo test (subset)"
            )
            for package in packages:
                _run_checked(f"cargo test -p {package}", ["cargo", "test", "-p", package])
        else:
            for package in packages:
                _run_checked(
                    f"cargo nextest run -p {package}",
                    ["cargo", "nextest", "run", "-p", package],
                )

    print("[gates-fast] done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
