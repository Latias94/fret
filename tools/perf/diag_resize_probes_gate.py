#!/usr/bin/env python3
"""
Perf gate for resize-focused suites (cross-platform, no jq/bash).

Python alternative to:
  - tools/perf/diag_resize_probes_gate.sh (requires bash + jq)
"""

from __future__ import annotations

import argparse
import json
import shutil
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


def _read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def _write_json(path: Path, v: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(v, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def _default_baseline_for_suite(suite: str) -> str:
    # Keep behavior consistent with the bash gate for now.
    if suite == "ui-resize-probes":
        return "docs/workstreams/perf-baselines/ui-resize-probes.macos-m4.v3.json"
    if suite == "ui-code-editor-resize-probes":
        return "docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json"
    raise KeyError(suite)


def _run(cmd: list[str], cwd: Path, stdout_path: Path, stderr_path: Path) -> int:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        p = subprocess.run(cmd, cwd=str(cwd), stdout=out, stderr=err)
        return int(p.returncode)


def _failures_count(check_path: Path) -> int | None:
    if not check_path.is_file():
        return None
    try:
        doc = _read_json(check_path)
    except Exception:
        return None
    failures = None
    if isinstance(doc, dict):
        failures = doc.get("failures")
    if not isinstance(failures, list):
        return None
    return len(failures)


def main() -> int:
    ap = argparse.ArgumentParser(
        description=(
            "Run a resize-focused `fretboard diag perf` suite and enforce perf baseline thresholds. "
            "Intended as a lightweight 'P0 resize must not regress' gate."
        ),
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--suite", default="ui-resize-probes")
    ap.add_argument("--out-dir", default="")
    ap.add_argument("--baseline", default="")
    ap.add_argument("--launch-bin", default="target/release/fret-ui-gallery")
    ap.add_argument("--timeout-ms", type=int, default=300_000)
    ap.add_argument("--attempts", type=int, default=1)
    ap.add_argument("--repeat", type=int, default=7)
    ap.add_argument("--warmup-frames", type=int, default=5)

    args = ap.parse_args()

    if args.attempts < 1:
        print("error: --attempts must be >= 1", file=sys.stderr)
        return 2

    workspace_root = _workspace_root()

    suite = str(args.suite)

    out_dir = args.out_dir.strip()
    if not out_dir:
        out_dir = f"target/fret-diag-resize-probes-gate-{int(time.time())}"
    out_dir_path = _resolve_workspace_path(workspace_root, out_dir)
    out_dir_path.mkdir(parents=True, exist_ok=True)

    baseline_raw = args.baseline.strip()
    if not baseline_raw:
        try:
            baseline_raw = _default_baseline_for_suite(suite)
        except KeyError:
            print(f"error: unknown --suite {suite!r} (provide --baseline explicitly)", file=sys.stderr)
            return 2
    baseline_path = _resolve_workspace_path(workspace_root, baseline_raw)

    launch_bin_path = _resolve_workspace_path(workspace_root, str(args.launch_bin))

    if not baseline_path.is_file():
        print(f"error: baseline not found: {baseline_path}", file=sys.stderr)
        return 2

    print(f"[gate] {suite} -> {out_dir_path} (attempts={int(args.attempts)})")
    print(f"[gate] baseline: {baseline_path}")
    print(f"[gate] launch-bin: {launch_bin_path}")

    passes = 0
    fails = 0
    selected_attempt_dir: Path | None = None
    attempt_summaries: list[dict[str, object]] = []

    for i in range(1, int(args.attempts) + 1):
        attempt_dir = out_dir_path / f"attempt-{i}"
        attempt_dir.mkdir(parents=True, exist_ok=True)

        cmd = [
            "cargo",
            "run",
            "-q",
            "-p",
            "fretboard",
            "--",
            "diag",
            "perf",
            suite,
            "--dir",
            str(attempt_dir),
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
            str(baseline_path),
            "--env",
            "FRET_UI_GALLERY_VIEW_CACHE=1",
            "--env",
            "FRET_UI_GALLERY_VIEW_CACHE_SHELL=1",
            "--env",
            "FRET_DIAG_SCRIPT_AUTO_DUMP=0",
            "--env",
            "FRET_DIAG_SEMANTICS=0",
            "--launch",
            "--",
            str(launch_bin_path),
        ]

        print(f"[gate] attempt {i}/{int(args.attempts)} -> {attempt_dir}")
        print("[gate] cmd:", " ".join(cmd))

        stdout_path = attempt_dir / "stdout.json"
        stderr_path = attempt_dir / "stderr.log"
        rc = _run(cmd, workspace_root, stdout_path, stderr_path)

        check_path = attempt_dir / "check.perf_thresholds.json"
        failures_count = _failures_count(check_path)

        attempt_pass = True
        if rc != 0:
            attempt_pass = False
        if failures_count is None:
            attempt_pass = False
        elif failures_count != 0:
            attempt_pass = False

        if attempt_pass:
            passes += 1
            if selected_attempt_dir is None:
                selected_attempt_dir = attempt_dir
        else:
            fails += 1

        attempt_summaries.append(
            {
                "attempt_dir": str(attempt_dir),
                "pass": attempt_pass,
                "rc": int(rc),
                "check": {
                    "perf_thresholds": str(check_path),
                    "failures": failures_count,
                },
                "stdout": str(stdout_path),
                "stderr": str(stderr_path),
            }
        )

    majority_required = int(args.attempts) // 2 + 1
    pass_gate = passes >= majority_required

    if selected_attempt_dir is None:
        selected_attempt_dir = out_dir_path / f"attempt-{int(args.attempts)}"

    # Preserve compatibility with downstream tooling by copying one attempt to the top-level paths.
    for name in ["stdout.json", "stderr.log", "check.perf_thresholds.json"]:
        src = selected_attempt_dir / name
        dst = out_dir_path / name
        try:
            if src.is_file():
                shutil.copyfile(src, dst)
        except Exception:
            pass

    summary = {
        "kind": "resize_probes_gate_summary",
        "pass": pass_gate,
        "out_dir": str(out_dir_path),
        "suite": suite,
        "baseline": str(baseline_path),
        "launch_bin": str(launch_bin_path),
        "attempts": int(args.attempts),
        "pass_attempts": passes,
        "fail_attempts": fails,
        "majority_required": majority_required,
        "selected_attempt_dir": str(selected_attempt_dir),
        "repeat": int(args.repeat),
        "warmup_frames": int(args.warmup_frames),
        "check": {
            "perf_thresholds": str(out_dir_path / "check.perf_thresholds.json"),
            "failures": None,
        },
        "stdout": str(out_dir_path / "stdout.json"),
        "stderr": str(out_dir_path / "stderr.log"),
        "attempt_summaries": attempt_summaries,
    }
    summary_path = out_dir_path / "summary.json"
    _write_json(summary_path, summary)

    if not pass_gate:
        print(
            f"[gate] FAIL (passes={passes}/{int(args.attempts)}; required={majority_required}). See: {summary_path}",
            file=sys.stderr,
        )
        return 1

    print(f"[gate] PASS (passes={passes}/{int(args.attempts)}; required={majority_required}). Summary: {summary_path}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)

