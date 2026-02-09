#!/usr/bin/env python3
"""
Perf gate for the `extras-marquee-steady` suite (cross-platform, no jq/bash).

This is a Python alternative to:
  - tools/perf/diag_extras_marquee_gate.sh (requires jq)
  - tools/perf/diag_extras_marquee_gate.ps1 (Windows-only)
"""

from __future__ import annotations

import argparse
import json
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


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run the extras-marquee-steady perf suite and enforce perf baseline thresholds.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--baseline", required=True)
    ap.add_argument("--out-dir", default="")
    ap.add_argument("--launch-bin", default="target/release/extras_marquee_perf_demo")
    ap.add_argument("--timeout-ms", type=int, default=300_000)
    ap.add_argument("--repeat", type=int, default=7)
    ap.add_argument("--warmup-frames", type=int, default=60)

    args = ap.parse_args()

    workspace_root = _workspace_root()
    baseline = _resolve_workspace_path(workspace_root, args.baseline)
    launch_bin = _resolve_workspace_path(workspace_root, args.launch_bin)

    if not baseline.is_file():
        print(f"error: baseline not found: {baseline}", file=sys.stderr)
        return 2

    out_dir = args.out_dir.strip()
    if not out_dir:
        out_dir = f"target/fret-diag-perf/extras-marquee-steady.{int(time.time())}"
    out_dir_path = _resolve_workspace_path(workspace_root, out_dir)
    out_dir_path.mkdir(parents=True, exist_ok=True)

    print(f"[gate] extras-marquee-steady -> {out_dir_path}")
    print(f"[gate] baseline: {baseline}")
    print(f"[gate] launch-bin: {launch_bin}")

    # Ensure the demo binary exists (fast no-op if already built).
    build_rc = subprocess.run(
        ["cargo", "build", "-q", "-p", "fret-demo", "--release", "--bin", "extras_marquee_perf_demo"],
        cwd=str(workspace_root),
    ).returncode
    if build_rc != 0:
        print("FAIL (failed to build extras_marquee_perf_demo)", file=sys.stderr)
        return int(build_rc)

    cmd = [
        "cargo",
        "run",
        "-q",
        "-p",
        "fretboard",
        "--",
        "diag",
        "perf",
        "extras-marquee-steady",
        "--dir",
        str(out_dir_path),
        "--timeout-ms",
        str(int(args.timeout_ms)),
        "--reuse-launch",
        "--repeat",
        str(int(args.repeat)),
        "--warmup-frames",
        str(int(args.warmup_frames)),
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

    print("[gate] cmd:", " ".join(cmd))
    stdout_path = out_dir_path / "stdout.json"
    stderr_path = out_dir_path / "stderr.log"
    rc = _run(cmd, workspace_root, stdout_path, stderr_path)
    if rc != 0:
        print(f"FAIL (rc={rc}). See: {stderr_path}", file=sys.stderr)
        return rc

    check_path = out_dir_path / "check.perf_thresholds.json"
    if not check_path.is_file():
        print(f"FAIL (missing {check_path}). See: {stderr_path}", file=sys.stderr)
        return 1

    doc = json.loads(check_path.read_text(encoding="utf-8"))
    failures = doc.get("failures", [])
    failures_count = len(failures) if isinstance(failures, list) else 0
    if failures_count != 0:
        print(
            f"FAIL (perf threshold failures={failures_count}). See: {check_path}",
            file=sys.stderr,
        )
        return 1

    print("PASS (extras-marquee-steady)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
