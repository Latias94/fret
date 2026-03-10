#!/usr/bin/env python3
"""Full gate runner (cross-platform, canonical entrypoint)."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _run_checked(name: str, argv: list[str]) -> None:
    print(f"[gates-full] {name}")
    proc = subprocess.run(argv, check=False)
    if proc.returncode != 0:
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-pre-release", action="store_true")
    parser.add_argument("--skip-web-goldens", action="store_true")
    parser.add_argument("--with-icons", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    py = sys.executable

    if not args.skip_pre_release:
        pre_release_args = [py, str(repo_root / "tools/pre_release.py")]
        if not args.with_icons:
            pre_release_args.append("--skip-icons")
        _run_checked(
            "pre_release.py (workspace policies + fmt + clippy + nextest)",
            pre_release_args,
        )

    if not args.skip_web_goldens:
        if shutil.which("cargo-nextest") is None:
            print(
                "[gates-full] warning: cargo-nextest is not installed; falling back to "
                "cargo test -p fret-ui-shadcn --features web-goldens"
            )
            _run_checked(
                "cargo test (web-goldens)",
                ["cargo", "test", "-p", "fret-ui-shadcn", "--features", "web-goldens"],
            )
        else:
            _run_checked(
                "cargo nextest run -p fret-ui-shadcn --features web-goldens",
                [
                    "cargo",
                    "nextest",
                    "run",
                    "-p",
                    "fret-ui-shadcn",
                    "--features",
                    "web-goldens",
                ],
            )

    print("[gates-full] done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
