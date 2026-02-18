#!/usr/bin/env python3
"""
Perf gates for EXTV2 external texture import suites (no jq/bash).

This intentionally uses `--launch` (filesystem transport) so it works without DevTools WS.

Baselines are selected via a platform tag (default: `windows-local` on Windows, `macos-local`
on macOS, `linux-local` otherwise).
"""

from __future__ import annotations

import argparse
import json
import platform
import subprocess
import sys
import time
from pathlib import Path


def _workspace_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _resolve_workspace_path(workspace_root: Path, p: str) -> Path:
    path = Path(p)
    if path.is_absolute():
        return path
    return workspace_root / path


def _run(cmd: list[str], cwd: Path, stdout_path: Path, stderr_path: Path) -> int:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        p = subprocess.run(cmd, cwd=str(cwd), stdout=out, stderr=err)
        return int(p.returncode)


def _assert_perf_check_clean(out_dir: Path, stderr_path: Path) -> None:
    check_path = out_dir / "check.perf_thresholds.json"
    if not check_path.is_file():
        raise RuntimeError(f"missing {check_path} (see: {stderr_path})")

    doc = json.loads(check_path.read_text(encoding="utf-8"))
    failures = doc.get("failures", [])
    if isinstance(failures, list) and len(failures) != 0:
        raise RuntimeError(
            f"perf threshold failures={len(failures)} (see: {check_path})"
        )


def _run_suite(
    *,
    workspace_root: Path,
    suite_label: str,
    script: Path,
    baseline: Path,
    launch_bin: Path,
    out_dir_root: Path,
    timeout_ms: int,
    repeat: int,
    warmup_frames: int,
) -> None:
    if not script.is_file():
        raise RuntimeError(f"script not found: {script}")
    if not baseline.is_file():
        raise RuntimeError(f"baseline not found: {baseline}")
    if not launch_bin.is_file():
        raise RuntimeError(f"launch bin not found: {launch_bin}")

    out_dir = out_dir_root / suite_label
    out_dir.mkdir(parents=True, exist_ok=True)

    cmd = [
        "cargo",
        "run",
        "-q",
        "-p",
        "fretboard",
        "--",
        "diag",
        "perf",
        str(script),
        "--dir",
        str(out_dir),
        "--timeout-ms",
        str(int(timeout_ms)),
        "--reuse-launch",
        "--repeat",
        str(int(repeat)),
        "--warmup-frames",
        str(int(warmup_frames)),
        "--sort",
        "time",
        "--top",
        "15",
        "--json",
        "--perf-baseline",
        str(baseline),
        "--env",
        "FRET_DIAG_SCRIPT_AUTO_DUMP=0",
        "--env",
        "FRET_DIAG_SEMANTICS=0",
        "--launch",
        "--",
        str(launch_bin),
    ]

    print(f"[gate] {suite_label} -> {out_dir}")
    print(f"[gate] baseline: {baseline}")
    print(f"[gate] script: {script}")
    print(f"[gate] launch-bin: {launch_bin}")
    print("[gate] cmd:", " ".join(cmd))

    stdout_path = out_dir / "stdout.json"
    stderr_path = out_dir / "stderr.log"
    rc = _run(cmd, workspace_root, stdout_path, stderr_path)
    if rc != 0:
        raise RuntimeError(f"rc={rc} (see: {stderr_path})")

    _assert_perf_check_clean(out_dir, stderr_path)
    print(f"PASS ({suite_label})")


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run EXTV2 external texture import perf suites and enforce perf baselines.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--out-dir", default="")
    ap.add_argument("--timeout-ms", type=int, default=300_000)
    ap.add_argument("--repeat", type=int, default=7)
    ap.add_argument("--warmup-frames", type=int, default=60)
    ap.add_argument(
        "--baseline-platform-tag",
        default="",
        help="Perf baseline platform tag (e.g. windows-local, macos-m4, web-local).",
    )
    ap.add_argument(
        "--launch-bin",
        default="target/release/external_texture_imports_demo",
        help="Path to the demo binary used for the launch transport.",
    )
    ap.add_argument(
        "--suite",
        action="append",
        default=[],
        help="Optional: restrict to a specific suite label (repeatable).",
    )

    args = ap.parse_args()

    workspace_root = _workspace_root()
    launch_bin = _resolve_workspace_path(workspace_root, args.launch_bin)

    if not launch_bin.is_file() and platform.system().lower() == "windows":
        exe = launch_bin.with_suffix(".exe")
        if exe.is_file():
            launch_bin = exe

    baseline_platform_tag = args.baseline_platform_tag.strip()
    if not baseline_platform_tag:
        system = platform.system().lower()
        if system == "windows":
            baseline_platform_tag = "windows-local"
        elif system == "darwin":
            baseline_platform_tag = "macos-local"
        else:
            baseline_platform_tag = "linux-local"

    out_dir = args.out_dir.strip()
    if not out_dir:
        out_dir = f"target/fret-diag-perf/extv2-external-texture-imports.{int(time.time())}"
    out_dir_root = _resolve_workspace_path(workspace_root, out_dir)
    out_dir_root.mkdir(parents=True, exist_ok=True)

    suites = [
        (
            "external-texture-imports-contract-path-perf-steady",
            "tools/diag-scripts/external-texture-imports-contract-path-perf-steady.json",
            "docs/workstreams/perf-baselines/external-texture-imports-contract-path",
        ),
        (
            "external-texture-imports-decoded-png-cpu-copy-perf-steady",
            "tools/diag-scripts/external-texture-imports-decoded-png-cpu-copy-perf-steady.json",
            "docs/workstreams/perf-baselines/external-texture-imports-decoded-png-cpu-copy",
        ),
    ]

    restrict = {s.strip() for s in args.suite if s.strip()}
    if restrict:
        suites = [t for t in suites if t[0] in restrict]
        unknown = restrict - {t[0] for t in suites}
        if unknown:
            print(f"error: unknown suite(s): {sorted(unknown)}", file=sys.stderr)
            return 2

    build_rc = subprocess.run(
        [
            "cargo",
            "build",
            "-q",
            "-p",
            "fret-demo",
            "--release",
            "--bin",
            "external_texture_imports_demo",
        ],
        cwd=str(workspace_root),
    ).returncode
    if build_rc != 0:
        print("FAIL (failed to build external_texture_imports_demo)", file=sys.stderr)
        return int(build_rc)

    try:
        for label, script, baseline_base in suites:
            baseline = f"{baseline_base}.{baseline_platform_tag}.v1.json"
            _run_suite(
                workspace_root=workspace_root,
                suite_label=label,
                script=_resolve_workspace_path(workspace_root, script),
                baseline=_resolve_workspace_path(workspace_root, baseline),
                launch_bin=launch_bin,
                out_dir_root=out_dir_root,
                timeout_ms=int(args.timeout_ms),
                repeat=int(args.repeat),
                warmup_frames=int(args.warmup_frames),
            )
    except RuntimeError as e:
        print(f"FAIL ({e})", file=sys.stderr)
        return 1

    print("PASS (extv2 external texture imports suites)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
