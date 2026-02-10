#!/usr/bin/env python3
"""
Run a small, CI-friendly imui v3 gate matrix.

Cross-platform replacement for `tools/diag_gate_imui_v3.ps1`.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _run(cmd: list[str], *, cwd: Path) -> int:
    proc = subprocess.run(cmd, cwd=str(cwd))
    return proc.returncode


def _demo_exe_path(*, repo_root: Path, release: bool) -> Path:
    is_windows = os.name == "nt"
    name = "imui_floating_windows_demo.exe" if is_windows else "imui_floating_windows_demo"
    profile = "release" if release else "debug"
    return repo_root / "target" / profile / name


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", default="target/fret-diag-imui-v3")
    parser.add_argument("--timeout-ms", type=int, default=180000)
    parser.add_argument("--poll-ms", type=int, default=50)
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()

    demo_build = ["cargo", "build", "-j", "1", "-p", "fret-demo", "--bin", "imui_floating_windows_demo"]
    if args.release:
        demo_build.append("--release")
    code = _run(demo_build, cwd=repo_root)
    if code != 0:
        return code

    demo_exe = _demo_exe_path(repo_root=repo_root, release=args.release)
    if not demo_exe.exists():
        print(f"imui demo exe not found: {demo_exe}", file=sys.stderr)
        return 2

    code = _run(["cargo", "nextest", "run", "-p", "fret-imui"], cwd=repo_root)
    if code != 0:
        return code

    code = _run(
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "fret-ui-kit",
            "--features",
            "imui",
            "--test",
            "imui_response_contract_smoke",
            "--test",
            "imui_adapter_seam_smoke",
            "--test",
            "imui_perf_guard_smoke",
        ],
        cwd=repo_root,
    )
    if code != 0:
        return code

    code = _run(
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "fret-docking",
            "--features",
            "imui",
            "--test",
            "imui_handshake_smoke",
        ],
        cwd=repo_root,
    )
    if code != 0:
        return code

    scripts = [
        "tools/diag-scripts/imui-float-window-drag-resize-context-menu.json",
        "tools/diag-scripts/imui-float-window-select-popup-coexistence.json",
        "tools/diag-scripts/imui-float-window-activate-on-content-bring-to-front.json",
    ]

    for script in scripts:
        cmd = [
            "cargo",
            "run",
            "-j",
            "1",
            "-p",
            "fretboard",
            "--",
            "diag",
            "run",
            script,
            "--dir",
            args.out_dir,
            "--timeout-ms",
            str(args.timeout_ms),
            "--poll-ms",
            str(args.poll_ms),
            "--pack",
            "--env",
            "FRET_DIAG_SEMANTICS=1",
            "--launch",
            "--",
            str(demo_exe),
        ]
        code = _run(cmd, cwd=repo_root)
        if code != 0:
            return code

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
