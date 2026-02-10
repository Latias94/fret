#!/usr/bin/env python3
"""
Select a stable `diag perf` baseline from multiple candidates (cross-platform, no jq/bash).

This mirrors the intent of `tools/perf/diag_perf_baseline_select.sh`:
  - Generate N candidate baselines (via `--perf-baseline-out`)
  - Validate each candidate M times (via `--perf-baseline`)
  - Pick a winner with priority:
      1) fewer validation failures
      2) lower suite p90 sum (rows[].measured_p90.top_total_time_us)
      3) lower sum of max_top_total_us thresholds

Example:
  python tools/perf/diag_perf_baseline_select.py \
    --suite extras-marquee-steady \
    --baseline-out docs/workstreams/perf-baselines/extras-marquee-steady.windows-rtx4090.v1.json \
    --preset docs/workstreams/perf-baselines/policies/extras-marquee-steady.v1.json \
    --candidates 2 --validate-runs 3 --repeat 7 --warmup-frames 5 \
    --work-dir target/fret-diag-baseline-select-extras-marquee-steady-v1 \
    --launch-bin target/release/extras_marquee_perf_demo \
    --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
    --env FRET_DIAG_SEMANTICS=0
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any


def _workspace_root() -> Path:
    # tools/perf/<this file> -> repo root
    return Path(__file__).resolve().parents[2]


def _resolve_workspace_path(workspace_root: Path, p: str) -> Path:
    path = Path(p)
    if path.is_absolute():
        return path
    return workspace_root / path


def _split_env_specs(env_specs: list[str]) -> list[str]:
    out: list[str] = []
    for spec in env_specs:
        s = spec.strip()
        if not s:
            continue
        # Convenience: allow comma-separated env specs, e.g. "A=1,B=2".
        if "," in s and " " not in s:
            parts = [p.strip() for p in s.split(",") if p.strip()]
            if all("=" in p for p in parts):
                out.extend(parts)
                continue
        out.append(s)
    return out


def _run(
    *,
    cmd: list[str],
    cwd: Path,
    stdout_path: Path,
    stderr_path: Path,
) -> int:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        p = subprocess.run(cmd, cwd=str(cwd), stdout=out, stderr=err)
        return int(p.returncode)


def _load_json(path: Path) -> Any:
    if not path.is_file():
        raise FileNotFoundError(str(path))

    # Occasionally, a writer may truncate/replace a JSON artifact while we read it (or the process
    # may exit early, leaving a 0-byte file). A short retry loop makes this helper less flaky.
    last_err: Exception | None = None
    for attempt in range(5):
        try:
            data = path.read_bytes()
            if not data:
                raise json.JSONDecodeError("empty JSON file", doc="", pos=0)
            return json.loads(data)
        except json.JSONDecodeError as e:
            last_err = e
            time.sleep(min(0.5, 0.05 * (2**attempt)))
            continue
    raise RuntimeError(f"failed to parse JSON after retries: {path}: {last_err}") from last_err


def _count_failures(check_path: Path) -> int:
    try:
        doc = _load_json(check_path)
    except (FileNotFoundError, RuntimeError) as e:
        # Treat missing/invalid artifacts as a hard failure signal for baseline selection, but
        # keep scanning other candidates to avoid aborting the whole selection run.
        print(f"warning: invalid validation report: {check_path}: {e}", file=sys.stderr)
        return 10_000

    failures = doc.get("failures", [])
    return len(failures) if isinstance(failures, list) else 0


@dataclass(frozen=True)
class BaselineMetrics:
    p90_sum_top_total_us: int
    threshold_sum_max_top_total_us: int


def _baseline_metrics(path: Path) -> BaselineMetrics:
    doc = _load_json(path)
    rows = doc.get("rows", []) or []

    p90_sum = 0
    thr_sum = 0
    for row in rows:
        measured_p90 = (row or {}).get("measured_p90") or {}
        p90 = int(measured_p90.get("top_total_time_us") or 0)
        p90_sum += p90

        thresholds = (row or {}).get("thresholds") or {}
        thr = int(thresholds.get("max_top_total_us") or 0)
        thr_sum += thr

    return BaselineMetrics(
        p90_sum_top_total_us=p90_sum,
        threshold_sum_max_top_total_us=thr_sum,
    )


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Select a stable perf baseline from multiple `fretboard diag perf` candidates.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--suite", default="ui-gallery-steady")
    ap.add_argument("--baseline-out", required=True)
    ap.add_argument("--preset", action="append", default=[], help="Seed policy preset JSON (repeatable).")
    ap.add_argument("--candidates", type=int, default=2)
    ap.add_argument("--validate-runs", type=int, default=3)
    ap.add_argument("--repeat", type=int, default=7)
    ap.add_argument("--warmup-frames", type=int, default=5)
    ap.add_argument("--headroom-pct", type=int, default=20)
    ap.add_argument("--work-dir", default="")
    ap.add_argument("--launch-bin", default="target/release/fret-ui-gallery")
    ap.add_argument("--timeout-ms", type=int, default=300_000)
    ap.add_argument(
        "--env",
        action="append",
        default=[],
        help="Forwarded to `fretboard diag perf --env KEY=VALUE` (repeatable).",
    )

    args = ap.parse_args()

    workspace_root = _workspace_root()
    suite = str(args.suite)

    baseline_out = _resolve_workspace_path(workspace_root, args.baseline_out)
    preset_paths = [_resolve_workspace_path(workspace_root, p) for p in args.preset]
    launch_bin = _resolve_workspace_path(workspace_root, args.launch_bin)

    work_dir = args.work_dir.strip()
    if not work_dir:
        work_dir = f"target/fret-diag-baseline-select-{suite}.{int(time.time())}"
    work_dir_path = _resolve_workspace_path(workspace_root, work_dir)
    work_dir_path.mkdir(parents=True, exist_ok=True)

    baseline_out.parent.mkdir(parents=True, exist_ok=True)

    env_specs = _split_env_specs(list(args.env))

    candidate_results: list[dict[str, Any]] = []
    best: tuple[int, int, int, str] | None = None

    def diag_cmd_common(out_dir: Path) -> list[str]:
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
            str(out_dir),
            "--timeout-ms",
            str(int(args.timeout_ms)),
            "--reuse-launch",
            "--sort",
            "time",
            "--json",
        ]
        return cmd

    def diag_cmd_with_env_and_launch(cmd: list[str]) -> list[str]:
        for env in env_specs:
            cmd += ["--env", env]
        cmd += ["--launch", "--", str(launch_bin)]
        return cmd

    for i in range(1, int(args.candidates) + 1):
        candidate_name = f"candidate-{i}"
        candidate_baseline = work_dir_path / f"{candidate_name}.baseline.json"
        candidate_out_dir = work_dir_path / f"{candidate_name}-baseline"
        candidate_out_dir.mkdir(parents=True, exist_ok=True)

        print(f"[baseline] candidate={i} out={candidate_baseline}")
        cmd = diag_cmd_common(candidate_out_dir)
        cmd += [
            "--repeat",
            str(int(args.repeat)),
            "--warmup-frames",
            str(int(args.warmup_frames)),
            "--top",
            "5",
            "--perf-baseline-out",
            str(candidate_baseline),
            "--perf-baseline-headroom-pct",
            str(int(args.headroom_pct)),
        ]
        for preset in preset_paths:
            cmd += ["--perf-baseline-seed-preset", str(preset)]
        cmd = diag_cmd_with_env_and_launch(cmd)

        stdout_path = candidate_out_dir / "stdout.json"
        stderr_path = candidate_out_dir / "stderr.log"
        print("[diag] cmd:", " ".join(cmd))
        rc = _run(cmd=cmd, cwd=workspace_root, stdout_path=stdout_path, stderr_path=stderr_path)
        if rc != 0:
            print(f"error: baseline run failed (rc={rc}). See: {stderr_path}", file=sys.stderr)
            return rc

        # Validate that the baseline JSON artifact is readable before spending time on validations.
        try:
            _ = _baseline_metrics(candidate_baseline)
        except Exception as e:
            print(
                f"warning: invalid baseline JSON: {candidate_baseline}: {e}. See: {stderr_path}",
                file=sys.stderr,
            )
            candidate_results.append(
                {
                    "name": candidate_name,
                    "baseline": str(candidate_baseline),
                    "fail_total": 10_000,
                    "suite_p90_total_time_us_sum": 2**31 - 1,
                    "threshold_sum_max_top_total_us": 2**31 - 1,
                    "validate_runs": [],
                }
            )
            continue

        fail_total = 0
        validate_runs: list[dict[str, Any]] = []
        for j in range(1, int(args.validate_runs) + 1):
            validation_out_dir = work_dir_path / f"{candidate_name}-validate-{j}"
            validation_out_dir.mkdir(parents=True, exist_ok=True)
            print(f"[validate] candidate={i} run={j}")
            vcmd = diag_cmd_common(validation_out_dir)
            vcmd += [
                "--repeat",
                "3",
                "--warmup-frames",
                str(int(args.warmup_frames)),
                "--top",
                "3",
                "--perf-baseline",
                str(candidate_baseline),
            ]
            vcmd = diag_cmd_with_env_and_launch(vcmd)

            vstdout = validation_out_dir / "stdout.json"
            vstderr = validation_out_dir / "stderr.log"
            print("[diag] cmd:", " ".join(vcmd))
            vrc = _run(cmd=vcmd, cwd=workspace_root, stdout_path=vstdout, stderr_path=vstderr)

            check_path = validation_out_dir / "check.perf_thresholds.json"
            failures = _count_failures(check_path)
            fail_total += failures
            validate_runs.append(
                {
                    "out_dir": str(validation_out_dir),
                    "exit_code": int(vrc),
                    "failures": int(failures),
                }
            )

        metrics = _baseline_metrics(candidate_baseline)
        p90_sum = int(metrics.p90_sum_top_total_us)
        thr_sum = int(metrics.threshold_sum_max_top_total_us)

        print(
            f"[candidate] name={candidate_name} fail_total={fail_total} "
            f"suite_p90_total_time_us_sum={p90_sum} threshold_sum={thr_sum}"
        )

        candidate_results.append(
            {
                "name": candidate_name,
                "baseline": str(candidate_baseline),
                "fail_total": int(fail_total),
                "suite_p90_total_time_us_sum": int(p90_sum),
                "threshold_sum_max_top_total_us": int(thr_sum),
                "validate_runs": validate_runs,
            }
        )

        key = (int(fail_total), int(p90_sum), int(thr_sum), str(candidate_baseline))
        if best is None or key < best:
            best = key

    if best is None:
        print("error: no candidate selected", file=sys.stderr)
        return 3

    selected_baseline_path = Path(best[3])
    shutil.copyfile(selected_baseline_path, baseline_out)

    summary = {
        "schema_version": 1,
        "kind": "perf_baseline_selection",
        "suite": suite,
        "baseline_out": str(baseline_out),
        "best_candidate": {
            "path": str(selected_baseline_path),
            "fail_total": int(best[0]),
            "suite_p90_total_time_us_sum": int(best[1]),
            "threshold_sum_max_top_total_us": int(best[2]),
        },
        "candidates": candidate_results,
    }

    summary_path = work_dir_path / "selection-summary.json"
    summary_path.write_text(json.dumps(summary, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(f"[done] baseline_out={baseline_out}")
    print(f"[done] summary={summary_path}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
