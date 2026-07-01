#!/usr/bin/env python3
"""
Consumption profile compile gates (workspace-level).

Goal: keep Fret usable in a modular, Bevy-like way ("depend on only what you need") by preventing
accidental dependency growth and feature drift from breaking minimal build profiles.

This script is intentionally lightweight: it runs a small set of `cargo check` commands for
portable profiles that should not pull platform/render backends.

Intended usage:
  python3 tools/check_consumption_profiles.py
"""

from __future__ import annotations

import subprocess
import sys


def _run_checked(name: str, argv: list[str]) -> None:
    print(f"[profiles] {name}")
    proc = subprocess.run(argv, check=False)
    if proc.returncode != 0:
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")


def _run_captured(name: str, argv: list[str]) -> str:
    print(f"[profiles] {name}")
    proc = subprocess.run(argv, check=False, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if proc.returncode != 0:
        print(proc.stdout, end="")
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")
    return proc.stdout


def _assert_tree_excludes(name: str, tree_output: str, banned_packages: set[str]) -> None:
    found: list[str] = []
    for line in tree_output.splitlines():
        package = line.strip().split(" v", 1)[0]
        if package in banned_packages:
            found.append(package)
    if found:
        banned = ", ".join(sorted(set(found)))
        raise SystemExit(f"{name} pulled backend/render packages: {banned}")


def main(argv: list[str]) -> int:
    if argv:
        raise SystemExit("This script takes no arguments.")

    # Profile A — contracts-only (portable)
    _run_checked("contracts: fret-core", ["cargo", "check", "-p", "fret-core", "--locked"])
    _run_checked("contracts: fret-runtime", ["cargo", "check", "-p", "fret-runtime", "--locked"])
    _run_checked("contracts: fret-platform (contracts)", ["cargo", "check", "-p", "fret-platform", "--locked"])
    _run_checked("contracts: fret-render-core", ["cargo", "check", "-p", "fret-render-core", "--locked"])

    # Profile B — UI substrate (portable kernel)
    _run_checked("ui-substrate: fret-ui", ["cargo", "check", "-p", "fret-ui", "--locked"])

    # Profile C — advanced/manual assembly surface (portable facade only; no backends)
    _run_checked(
        "assembly: fret-framework (core+runtime+ui)",
        [
            "cargo",
            "check",
            "-p",
            "fret-framework",
            "--locked",
            "--no-default-features",
            "--features",
            "core,runtime,ui",
        ],
    )

    banned_backend_packages = {
        "fret-launch",
        "fret-platform-native",
        "fret-platform-web",
        "fret-render",
        "fret-render-wgpu",
        "fret-runner-web",
        "fret-runner-winit",
        "wgpu",
        "winit",
    }
    _assert_tree_excludes(
        "app-authoring: fret --no-default-features",
        _run_captured(
            "app-authoring: fret --no-default-features tree",
            [
                "cargo",
                "tree",
                "-p",
                "fret",
                "--locked",
                "--no-default-features",
                "-e",
                "normal",
                "--prefix",
                "none",
            ],
        ),
        banned_backend_packages,
    )
    _assert_tree_excludes(
        "app-authoring: fret app",
        _run_captured(
            "app-authoring: fret app tree",
            [
                "cargo",
                "tree",
                "-p",
                "fret",
                "--locked",
                "--no-default-features",
                "--features",
                "app",
                "-e",
                "normal",
                "--prefix",
                "none",
            ],
        ),
        banned_backend_packages,
    )
    _assert_tree_excludes(
        "bootstrap: fret-bootstrap --no-default-features",
        _run_captured(
            "bootstrap: fret-bootstrap --no-default-features tree",
            [
                "cargo",
                "tree",
                "-p",
                "fret-bootstrap",
                "--locked",
                "--no-default-features",
                "-e",
                "normal",
                "--prefix",
                "none",
            ],
        ),
        banned_backend_packages,
    )
    _run_checked(
        "app-authoring: fret app check",
        ["cargo", "check", "-p", "fret", "--locked", "--no-default-features", "--features", "app"],
    )
    _run_checked(
        "app-authoring: fret default facade check",
        ["cargo", "check", "-p", "fret", "--locked"],
    )
    _run_checked(
        "app-authoring: fret batteries facade check",
        ["cargo", "check", "-p", "fret", "--locked", "--features", "batteries"],
    )
    _run_checked(
        "app-authoring: fret app authoring spec test check",
        [
            "cargo",
            "check",
            "-p",
            "fret",
            "--locked",
            "--no-default-features",
            "--features",
            "app",
            "--test",
            "backend_free_app_authoring_profile",
        ],
    )
    _run_checked(
        "bootstrap: fret-bootstrap no-default check",
        ["cargo", "check", "-p", "fret-bootstrap", "--locked", "--no-default-features"],
    )
    _run_checked(
        "bootstrap: fret-bootstrap no-default planning test check",
        [
            "cargo",
            "check",
            "-p",
            "fret-bootstrap",
            "--locked",
            "--no-default-features",
            "--test",
            "backend_free_bootstrap_profile",
        ],
    )

    # Profile D — launcher facade
    _run_checked("launch: fret-launch", ["cargo", "check", "-p", "fret-launch", "--locked"])

    print("[profiles] done")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
