#!/usr/bin/env python3
"""
VirtualList window-boundary crossing gate (cross-platform, no jq/bash).

Python alternative to:
  - tools/perf/diag_vlist_boundary_gate.sh (requires bash + jq)
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any


def _workspace_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _resolve_workspace_path(workspace_root: Path, p: str) -> Path:
    path = Path(p)
    if path.is_absolute():
        return path
    return workspace_root / path


def _read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _write_json(path: Path, v: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(v, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def _run(cmd: list[str], cwd: Path, stdout_path: Path, stderr_path: Path) -> int:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        p = subprocess.run(cmd, cwd=str(cwd), stdout=out, stderr=err)
        return int(p.returncode)


def _failures_len(check_path: Path) -> int | None:
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


def _get_int_field(doc_path: Path, field: str, default: int = 0) -> int:
    if not doc_path.is_file():
        return default
    try:
        doc = _read_json(doc_path)
    except Exception:
        return default
    if not isinstance(doc, dict):
        return default
    v = doc.get(field, default)
    try:
        return int(v or 0)
    except Exception:
        return default


def _max_snapshot_stat(bundle_json: Path, key: str) -> int:
    if not bundle_json.is_file():
        return 0
    try:
        bundle = _read_json(bundle_json)
    except Exception:
        return 0
    if not isinstance(bundle, dict):
        return 0
    windows = bundle.get("windows")
    if not isinstance(windows, list):
        return 0

    best = 0
    for w in windows:
        if not isinstance(w, dict):
            continue
        snaps = w.get("snapshots")
        if not isinstance(snaps, list):
            continue
        for s in snaps:
            if not isinstance(s, dict):
                continue
            debug = s.get("debug")
            if not isinstance(debug, dict):
                continue
            stats = debug.get("stats")
            if not isinstance(stats, dict):
                continue
            try:
                best = max(best, int(stats.get(key, 0) or 0))
            except Exception:
                continue
    return best


@dataclass(frozen=True)
class Thresholds:
    prefetch_max: int
    escape_max: int
    non_retained_max: int
    max_cache_key_mismatch: int
    max_needs_rerender: int


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run VirtualList window-boundary crossing gate repeatedly and enforce invariants.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--out-dir", default="")
    ap.add_argument(
        "--script",
        dest="script_path",
        default="tools/diag-scripts/ui-gallery-virtual-list-window-boundary-crossing-steady.json",
    )
    ap.add_argument("--launch-bin", default="target/release/fret-ui-gallery")
    ap.add_argument("--timeout-ms", type=int, default=300_000)
    ap.add_argument("--retained", type=int, choices=[0, 1], default=1)
    ap.add_argument("--prefetch-max", type=int, default=3)
    ap.add_argument("--escape-max", type=int, default=0)
    ap.add_argument("--non-retained-max", type=int, default=0)
    ap.add_argument("--max-cache-key-mismatch", type=int, default=0)
    ap.add_argument("--max-needs-rerender", type=int, default=0)

    args = ap.parse_args()

    if args.runs < 1:
        print("error: --runs must be >= 1", file=sys.stderr)
        return 2

    workspace_root = _workspace_root()

    out_dir = args.out_dir.strip()
    if not out_dir:
        out_dir = f"target/fret-diag-vlist-boundary-gate-{int(time.time())}"
    out_dir_path = _resolve_workspace_path(workspace_root, out_dir)
    out_dir_path.mkdir(parents=True, exist_ok=True)

    script_path = _resolve_workspace_path(workspace_root, str(args.script_path))
    launch_bin = _resolve_workspace_path(workspace_root, str(args.launch_bin))

    if not script_path.is_file():
        print(f"error: script not found: {script_path}", file=sys.stderr)
        return 2

    thresholds = Thresholds(
        prefetch_max=int(args.prefetch_max),
        escape_max=int(args.escape_max),
        non_retained_max=int(args.non_retained_max),
        max_cache_key_mismatch=int(args.max_cache_key_mismatch),
        max_needs_rerender=int(args.max_needs_rerender),
    )

    results: list[dict[str, object]] = []
    run_failures = 0

    for i in range(1, int(args.runs) + 1):
        run_dir = out_dir_path / f"run-{i}"
        run_dir.mkdir(parents=True, exist_ok=True)

        cmd: list[str] = [
            "cargo",
            "run",
            "-q",
            "-p",
            "fretboard",
            "--",
            "diag",
            "run",
            str(script_path),
            "--dir",
            str(run_dir),
            "--timeout-ms",
            str(int(args.timeout_ms)),
            "--check-vlist-window-shifts-explainable",
            "--check-vlist-window-shifts-have-prepaint-actions",
            "--check-vlist-window-shifts-non-retained-max",
            str(thresholds.non_retained_max),
            "--check-vlist-window-shifts-prefetch-max",
            str(thresholds.prefetch_max),
            "--check-vlist-window-shifts-escape-max",
            str(thresholds.escape_max),
            "--env",
            "FRET_UI_GALLERY_VIEW_CACHE=1",
            "--env",
            "FRET_UI_GALLERY_VIEW_CACHE_SHELL=1",
            "--env",
            "FRET_UI_GALLERY_VLIST_MINIMAL=1",
            "--env",
            "FRET_DIAG_SCRIPT_AUTO_DUMP=0",
        ]

        if int(args.retained) == 0:
            cmd += ["--env", "FRET_UI_GALLERY_VLIST_RETAINED=0"]

        cmd += ["--launch", "--", str(launch_bin)]

        print(f"[run] {i}/{int(args.runs)} -> {run_dir} (retained={int(args.retained)})")
        rc = _run(cmd, workspace_root, run_dir / "stdout.log", run_dir / "stderr.log")

        explainable_file = run_dir / "check.vlist_window_shifts_explainable.json"
        prepaint_file = run_dir / "check.vlist_window_shifts_have_prepaint_actions.json"
        prefetch_file = run_dir / "check.vlist_window_shifts_prefetch_max.json"
        escape_file = run_dir / "check.vlist_window_shifts_escape_max.json"
        non_retained_file = run_dir / "check.vlist_window_shifts_non_retained_max.json"

        total_shifts = _get_int_field(explainable_file, "total_shifts", default=0)
        prefetch = _get_int_field(prefetch_file, "total_kind_shifts", default=0)
        escape = _get_int_field(escape_file, "total_kind_shifts", default=0)
        non_retained = _get_int_field(non_retained_file, "total_non_retained_shifts", default=0)
        explainable_failures = _failures_len(explainable_file)
        prepaint_failures = _failures_len(prepaint_file)

        if explainable_failures is None:
            explainable_failures = 1
        if prepaint_failures is None:
            prepaint_failures = 1

        cache_key_mismatch_max = 0
        needs_rerender_max = 0

        latest_txt = run_dir / "latest.txt"
        if latest_txt.is_file():
            try:
                latest_dir_raw = latest_txt.read_text(encoding="utf-8", errors="replace").strip()
                bundle_path = run_dir / latest_dir_raw / "bundle.json"
                cache_key_mismatch_max = _max_snapshot_stat(bundle_path, "view_cache_roots_cache_key_mismatch")
                needs_rerender_max = _max_snapshot_stat(bundle_path, "view_cache_roots_needs_rerender")
            except Exception:
                pass

        cache_key_budget_ok = cache_key_mismatch_max <= thresholds.max_cache_key_mismatch
        needs_rerender_budget_ok = needs_rerender_max <= thresholds.max_needs_rerender

        check_failures = int(explainable_failures) + int(prepaint_failures)
        gate_pass = (
            rc == 0
            and check_failures == 0
            and cache_key_budget_ok
            and needs_rerender_budget_ok
        )

        if not gate_pass:
            run_failures += 1

        print(
            "  rc={rc} shifts={shifts} prefetch={prefetch} escape={escape} non_retained={non_retained} "
            "cache_key_mismatch_max={ck} needs_rerender_max={nr} check_failures={cf}".format(
                rc=rc,
                shifts=total_shifts,
                prefetch=prefetch,
                escape=escape,
                non_retained=non_retained,
                ck=cache_key_mismatch_max,
                nr=needs_rerender_max,
                cf=check_failures,
            )
        )

        results.append(
            {
                "run_dir": str(run_dir),
                "exit_code": int(rc),
                "gate_pass": bool(gate_pass),
                "total_shifts": int(total_shifts),
                "prefetch": int(prefetch),
                "escape": int(escape),
                "non_retained": int(non_retained),
                "explainable_failures": int(explainable_failures),
                "prepaint_failures": int(prepaint_failures),
                "cache_key_mismatch_max": int(cache_key_mismatch_max),
                "needs_rerender_max": int(needs_rerender_max),
                "cache_key_budget_ok": bool(cache_key_budget_ok),
                "needs_rerender_budget_ok": bool(needs_rerender_budget_ok),
            }
        )

    summary = {
        "schema_version": 1,
        "script": str(script_path),
        "launch_bin": str(launch_bin),
        "profile": {"retained": bool(int(args.retained) == 1)},
        "thresholds": {
            "prefetch_max": thresholds.prefetch_max,
            "escape_max": thresholds.escape_max,
            "non_retained_max": thresholds.non_retained_max,
            "max_cache_key_mismatch": thresholds.max_cache_key_mismatch,
            "max_needs_rerender": thresholds.max_needs_rerender,
        },
        "runs": int(args.runs),
        "run_failures": int(run_failures),
        "pass": bool(run_failures == 0),
        "results": results,
    }

    summary_path = out_dir_path / "summary.json"
    _write_json(summary_path, summary)
    print(f"[summary] {summary_path}")

    # Print a compact view (similar to the bash script).
    compact = {
        "pass": summary["pass"],
        "run_failures": summary["run_failures"],
        "profile": summary["profile"],
        "thresholds": summary["thresholds"],
        "runs": summary["runs"],
        "sample": [
            {
                "exit_code": r["exit_code"],
                "total_shifts": r["total_shifts"],
                "prefetch": r["prefetch"],
                "escape": r["escape"],
                "non_retained": r["non_retained"],
                "cache_key_mismatch_max": r["cache_key_mismatch_max"],
                "needs_rerender_max": r["needs_rerender_max"],
                "gate_pass": r["gate_pass"],
            }
            for r in results
        ],
    }
    print(json.dumps(compact, indent=2, sort_keys=False))

    if run_failures != 0:
        return 1
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)

