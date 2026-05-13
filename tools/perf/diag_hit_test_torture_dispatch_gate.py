#!/usr/bin/env python3
"""
Perf gate for the UI Gallery hit-test torture pointer-move contract.

This helper wraps the promoted `perf-ui-gallery-hit-test-torture-steady`
threshold gate. It is intentionally a direct threshold gate instead of a
baseline-selected gate: the architectural invariant is that stable
tree/layer topology must reuse dispatch snapshots rather than rebuilding the
large routing forest on every pointer move.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


def _workspace_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _resolve_workspace_path(workspace_root: Path, p: str) -> Path:
    path = Path(p)
    if path.is_absolute():
        return path
    return workspace_root / path


def _maybe_with_exe_suffix(path: Path) -> Path:
    if sys.platform.startswith("win") and not path.suffix:
        candidate = path.with_suffix(".exe")
        if candidate.is_file():
            return candidate
    if path.is_file():
        return path
    if path.suffix:
        return path
    candidate = path.with_suffix(".exe")
    if candidate.is_file():
        return candidate
    return path


def _read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def _failures_count(check_path: Path) -> int | None:
    if not check_path.is_file():
        return None
    try:
        doc = _read_json(check_path)
    except Exception:
        return None
    if not isinstance(doc, dict):
        return None
    failures = doc.get("failures")
    if not isinstance(failures, list):
        return None
    return len(failures)


def _row_summary(check_path: Path) -> dict[str, Any] | None:
    if not check_path.is_file():
        return None
    try:
        doc = _read_json(check_path)
    except Exception:
        return None
    if not isinstance(doc, dict):
        return None
    rows = doc.get("rows")
    if not isinstance(rows, list) or not rows:
        return None
    row = rows[0]
    if not isinstance(row, dict):
        return None
    return {
        "script": row.get("script"),
        "repeat": row.get("repeat"),
        "thresholds": row.get("thresholds"),
        "max": row.get("max"),
        "p50": row.get("p50"),
        "p95": row.get("p95"),
        "worst_run": row.get("worst_run"),
    }


def _run(cmd: list[str], cwd: Path, stdout_path: Path, stderr_path: Path) -> int:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        p = subprocess.run(cmd, cwd=str(cwd), stdout=out, stderr=err)
        return int(p.returncode)


def main() -> int:
    ap = argparse.ArgumentParser(
        description=(
            "Run the UI Gallery hit-test torture pointer-move dispatch gate. "
            "Defaults match the promoted Windows RTX4090 repeat=7 contract."
        ),
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--suite", default="perf-ui-gallery-hit-test-torture-steady")
    ap.add_argument("--out-dir", default="")
    ap.add_argument("--fretboard-dev-bin", default="target/release/fretboard-dev")
    ap.add_argument("--launch-bin", default="target/release/fret-ui-gallery")
    ap.add_argument("--timeout-ms", type=int, default=300_000)
    ap.add_argument("--repeat", type=int, default=7)
    ap.add_argument("--warmup-frames", type=int, default=5)
    ap.add_argument("--top", type=int, default=5)
    ap.add_argument("--max-pointer-move-dispatch-us", type=int, default=250)
    ap.add_argument("--max-pointer-move-hit-test-us", type=int, default=100)
    ap.add_argument("--max-pointer-move-global-changes", type=int, default=0)
    ap.add_argument("--stripes", type=int, default=256)
    ap.add_argument("--noise", type=int, default=20_000)
    ap.add_argument("--max-snapshots", type=int, default=240)
    ap.add_argument(
        "--no-reuse-launch",
        action="store_true",
        default=False,
        help="Do not forward `--reuse-launch` to `diag perf`.",
    )

    args = ap.parse_args()

    if int(args.repeat) < 1:
        print("error: --repeat must be >= 1", file=sys.stderr)
        return 2

    workspace_root = _workspace_root()

    out_dir = str(args.out_dir).strip()
    if not out_dir:
        out_dir = f"target/fret-diag-hit-test-torture-dispatch-gate-{int(time.time())}"
    out_dir_path = _resolve_workspace_path(workspace_root, out_dir)
    out_dir_path.mkdir(parents=True, exist_ok=True)

    fretboard_dev_bin = _maybe_with_exe_suffix(
        _resolve_workspace_path(workspace_root, str(args.fretboard_dev_bin))
    )
    launch_bin = _maybe_with_exe_suffix(_resolve_workspace_path(workspace_root, str(args.launch_bin)))

    if not fretboard_dev_bin.is_file():
        print(
            f"error: fretboard-dev binary not found: {fretboard_dev_bin} "
            "(build it via `cargo build -p fretboard-dev --release`)",
            file=sys.stderr,
        )
        return 2
    if not launch_bin.is_file():
        print(
            f"error: launch bin not found: {launch_bin} "
            "(build it via `cargo build -p fret-ui-gallery --release --features gallery-dev`)",
            file=sys.stderr,
        )
        return 2

    cmd: list[str] = [
        str(fretboard_dev_bin),
        "diag",
        "perf",
        str(args.suite),
        "--dir",
        str(out_dir_path),
        "--timeout-ms",
        str(int(args.timeout_ms)),
    ]
    if not bool(args.no_reuse_launch):
        cmd.append("--reuse-launch")
    cmd += [
        "--repeat",
        str(int(args.repeat)),
        "--warmup-frames",
        str(int(args.warmup_frames)),
        "--sort",
        "dispatch",
        "--top",
        str(int(args.top)),
        "--json",
        "--max-pointer-move-dispatch-us",
        str(int(args.max_pointer_move_dispatch_us)),
        "--max-pointer-move-hit-test-us",
        str(int(args.max_pointer_move_hit_test_us)),
        "--max-pointer-move-global-changes",
        str(int(args.max_pointer_move_global_changes)),
        "--env",
        f"FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES={int(args.stripes)}",
        "--env",
        f"FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE={int(args.noise)}",
        "--env",
        "FRET_DIAG_SCRIPT_AUTO_DUMP=0",
        "--env",
        f"FRET_DIAG_MAX_SNAPSHOTS={int(args.max_snapshots)}",
        "--launch",
        "--",
        str(launch_bin),
    ]

    stdout_path = out_dir_path / "stdout.json"
    stderr_path = out_dir_path / "stderr.log"

    print(f"[gate] {args.suite} -> {out_dir_path}")
    print(f"[gate] fretboard-dev: {fretboard_dev_bin}")
    print(f"[gate] launch-bin: {launch_bin}")
    print("[gate] cmd:", " ".join(cmd))

    rc = _run(cmd, workspace_root, stdout_path, stderr_path)
    check_path = out_dir_path / "check.perf_thresholds.json"
    failures = _failures_count(check_path)
    row_summary = _row_summary(check_path)

    summary = {
        "kind": "hit_test_torture_dispatch_gate_summary",
        "pass": rc == 0 and failures == 0,
        "out_dir": str(out_dir_path),
        "suite": str(args.suite),
        "fretboard_dev_bin": str(fretboard_dev_bin),
        "launch_bin": str(launch_bin),
        "cmd": cmd,
        "rc": int(rc),
        "repeat": int(args.repeat),
        "warmup_frames": int(args.warmup_frames),
        "thresholds": {
            "max_pointer_move_dispatch_us": int(args.max_pointer_move_dispatch_us),
            "max_pointer_move_hit_test_us": int(args.max_pointer_move_hit_test_us),
            "max_pointer_move_global_changes": int(args.max_pointer_move_global_changes),
        },
        "env": {
            "FRET_UI_GALLERY_HIT_TEST_TORTURE_STRIPES": int(args.stripes),
            "FRET_UI_GALLERY_HIT_TEST_TORTURE_NOISE": int(args.noise),
            "FRET_DIAG_SCRIPT_AUTO_DUMP": 0,
            "FRET_DIAG_MAX_SNAPSHOTS": int(args.max_snapshots),
        },
        "check": {
            "perf_thresholds": str(check_path),
            "failures": failures,
            "row_summary": row_summary,
        },
        "stdout": str(stdout_path),
        "stderr": str(stderr_path),
    }
    summary_path = out_dir_path / "summary.json"
    _write_json(summary_path, summary)

    # Keep the helper shape friendly to older automation that expects gate.summary.json.
    try:
        shutil.copyfile(summary_path, out_dir_path / "gate.summary.json")
    except Exception:
        pass

    if rc != 0:
        print(f"[gate] FAIL: fretboard-dev returned {rc}. Summary: {summary_path}", file=sys.stderr)
        return 1
    if failures is None:
        print(f"[gate] FAIL: missing or unreadable {check_path}. Summary: {summary_path}", file=sys.stderr)
        return 1
    if failures != 0:
        print(f"[gate] FAIL: perf thresholds failed ({failures}). Summary: {summary_path}", file=sys.stderr)
        return 1

    print(f"[gate] PASS. Summary: {summary_path}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
