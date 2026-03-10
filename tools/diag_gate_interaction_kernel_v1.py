#!/usr/bin/env python3
"""Run a small, CI-friendly gate matrix for the interaction-kernel v1 workstream."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _exe_name(stem: str) -> str:
    return f"{stem}.exe" if os.name == "nt" else stem


def _run_checked(name: str, argv: list[str], *, cwd: Path) -> None:
    print(f"[diag-gate-interaction-kernel-v1] {name}")
    proc = subprocess.run(argv, cwd=str(cwd), check=False)
    if proc.returncode != 0:
        raise SystemExit(f"Step failed: {name} (exit code: {proc.returncode})")


def _run_nextest_or_test(
    *,
    package: str,
    cwd: Path,
    features: list[str] | None = None,
    filters: list[str] | None = None,
) -> None:
    feature_args: list[str] = []
    if features:
        feature_args = ["--features", ",".join(features)]

    if shutil.which("cargo-nextest") is not None:
        cmd = ["cargo", "nextest", "run", "-p", package, *feature_args]
        if filters:
            cmd.extend(filters)
        _run_checked(" ".join(cmd), cmd, cwd=cwd)
        return

    if not filters:
        _run_checked(
            f"cargo test -p {package}",
            ["cargo", "test", "-p", package, *feature_args],
            cwd=cwd,
        )
        return

    for test_filter in filters:
        _run_checked(
            f"cargo test -p {package} {test_filter}",
            ["cargo", "test", "-p", package, *feature_args, test_filter],
            cwd=cwd,
        )


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", default="target/fret-diag-interaction-kernel-v1")
    parser.add_argument("--timeout-ms", type=int, default=180000)
    parser.add_argument("--poll-ms", type=int, default=50)
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--weak-dock-hover", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    run_out_dir = repo_root / args.out_dir / str(int(time.time() * 1000))
    profile_dir = "release" if args.release else "debug"

    _run_nextest_or_test(
        package="fret-interaction",
        cwd=repo_root,
        features=["runtime"],
    )
    _run_nextest_or_test(
        package="fret-ui-kit",
        cwd=repo_root,
        features=["imui"],
    )
    _run_nextest_or_test(
        package="fret-node",
        cwd=repo_root,
        filters=[
            "viewport_helper_conformance",
            "viewport_animation_conformance",
            "threshold_zoom_conformance",
            "translate_extent_conformance",
        ],
    )

    fretboard_build = ["cargo", "build", "-j", "1", "-p", "fretboard"]
    if args.release:
        fretboard_build.append("--release")
    _run_checked("cargo build -p fretboard", fretboard_build, cwd=repo_root)

    demo_build = ["cargo", "build", "-j", "1", "-p", "fret-demo", "--bin", "imui_floating_windows_demo"]
    if args.release:
        demo_build.append("--release")
    _run_checked("cargo build -p fret-demo --bin imui_floating_windows_demo", demo_build, cwd=repo_root)

    demo_exe = repo_root / "target" / profile_dir / _exe_name("imui_floating_windows_demo")
    if not demo_exe.exists():
        raise SystemExit(f"imui floating windows demo exe not found: {demo_exe}")

    fretboard_exe = repo_root / "target" / profile_dir / _exe_name("fretboard")
    if not fretboard_exe.exists():
        raise SystemExit(f"fretboard exe not found: {fretboard_exe}")

    scripts = [
        {
            "path": "tools/diag-scripts/imui-float-window-titlebar-drag-screenshots.json",
            "extra_args": [
                "--check-stale-paint",
                "imui-float-demo.a.activate",
                "--check-stale-paint-eps",
                "0.5",
                "--env",
                "FRET_DIAG_GPU_SCREENSHOTS=1",
                "--env",
                "FRET_DIAG_REDACT_TEXT=0",
            ],
        },
        {
            "path": "tools/diag-scripts/imui-float-window-text-wrap-no-overlap-150.json",
            "extra_args": [],
        },
    ]

    for script in scripts:
        script_path = str(script["path"])
        script_name = Path(script_path).stem
        script_out_dir = run_out_dir / script_name
        argv_run = [
            str(fretboard_exe),
            "diag",
            "run",
            script_path,
            "--dir",
            str(script_out_dir),
            "--timeout-ms",
            str(args.timeout_ms),
            "--poll-ms",
            str(args.poll_ms),
            *script["extra_args"],
            "--pack",
            "--env",
            "FRET_DIAG_SEMANTICS=1",
            "--launch",
            "--",
            str(demo_exe),
        ]
        _run_checked(f"fretboard diag run {script_name}", argv_run, cwd=repo_root)

    editor_build = ["cargo", "build", "-j", "1", "-p", "fret-demo", "--bin", "imui_editor_proof_demo"]
    if args.release:
        editor_build.append("--release")
    _run_checked("cargo build -p fret-demo --bin imui_editor_proof_demo", editor_build, cwd=repo_root)

    editor_exe = repo_root / "target" / profile_dir / _exe_name("imui_editor_proof_demo")
    if not editor_exe.exists():
        raise SystemExit(f"imui editor proof demo exe not found: {editor_exe}")

    m3_extra: list[str] = []
    if not args.weak_dock_hover:
        m3_extra += ["--check-dock-drag-current-windows-min", "2"]

    _run_checked(
        "fretboard diag run imui-editor-proof-multiwindow-overlap-topmost-hover",
        [
            str(fretboard_exe),
            "diag",
            "run",
            "tools/diag-scripts/imui-editor-proof-multiwindow-overlap-topmost-hover.json",
            "--dir",
            str(run_out_dir / "imui-editor-proof-multiwindow-overlap-topmost-hover"),
            "--timeout-ms",
            str(args.timeout_ms),
            "--poll-ms",
            str(args.poll_ms),
            "--check-dock-drag-min",
            "1",
            *m3_extra,
            "--pack",
            "--env",
            "FRET_DIAG_SEMANTICS=1",
            "--launch",
            "--",
            str(editor_exe),
        ],
        cwd=repo_root,
    )

    print(f"[diag-gate-interaction-kernel-v1] done (out_dir={run_out_dir})")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
