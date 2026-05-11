#!/usr/bin/env python3
"""
Select a stable `diag perf` baseline from multiple candidates (cross-platform, no jq/bash).

This mirrors the intent of `tools/perf/diag_perf_baseline_select.sh`:
  - Generate N candidate baselines (via `--perf-baseline-out`)
  - Validate each candidate M times (via `--perf-baseline`)
  - Validation uses the same repeat count as baseline generation by default so the selected
    contract matches the intended gate surface.
  - Generated baselines record p50/p90/p95/max; selection still ranks by p90 to favor
    stable typical performance before threshold size.
  - By default, generated baselines use the UI threshold surface. Renderer timings remain
    recorded under measured_* but are not hard thresholds unless explicitly requested.
    Use ui-renderer-payload for UI thresholds plus renderer payload counters without gating
    renderer micro-timings.
  - Pick a winner with priority:
      1) fewer validation failures
      2) no threshold loosening compared with the existing --baseline-out file
      3) lower suite p90 sum (rows[].measured_p90.top_total_time_us)
      4) lower sum of max_top_total_us thresholds
  - The selected candidate must have zero validation failures unless --allow-failures is passed.
  - The selected candidate must not loosen existing numeric thresholds unless
    --allow-threshold-loosening is passed.
  - Use --clamp-threshold-loosening to validate candidates with the existing stricter thresholds
    preserved whenever their measured values still fit that older contract.

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

DEFAULT_PREWARM_SCRIPT = "tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json"
DEFAULT_PRELUDE_SCRIPT = "tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json"
THRESHOLD_TO_MEASURED_METRIC = {
    "max_frame_p95_layout_us": "frame_p95_layout_time_us",
    "max_frame_p95_solve_us": "frame_p95_layout_engine_solve_time_us",
    "max_frame_p95_total_us": "frame_p95_total_time_us",
    "max_pointer_move_dispatch_us": "pointer_move_max_dispatch_time_us",
    "max_pointer_move_global_changes": "pointer_move_snapshots_with_global_changes",
    "max_pointer_move_hit_test_us": "pointer_move_max_hit_test_time_us",
    "max_renderer_encode_scene_text_ops": "renderer_encode_scene_text_ops",
    "max_renderer_encode_scene_us": "renderer_encode_scene_us",
    "max_renderer_encoder_finish_us": "renderer_encoder_finish_us",
    "max_renderer_instance_bytes": "renderer_instance_bytes",
    "max_renderer_prepare_svg_us": "renderer_prepare_svg_us",
    "max_renderer_prepare_text_us": "renderer_prepare_text_us",
    "max_renderer_record_passes_us": "renderer_record_passes_us",
    "max_renderer_upload_us": "renderer_upload_us",
    "max_run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max": (
        "run_paint_cache_hit_test_only_replay_rejected_key_mismatch_max"
    ),
    "max_top_layout_us": "top_layout_time_us",
    "max_top_solve_us": "top_layout_engine_solve_time_us",
    "max_top_total_us": "top_total_time_us",
    "min_run_paint_cache_hit_test_only_replay_allowed_max": (
        "run_paint_cache_hit_test_only_replay_allowed_max"
    ),
}


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
        thr = int(thresholds.get("max_top_total_us") or thresholds.get("max_frame_p95_total_us") or 0)
        thr_sum += thr

    return BaselineMetrics(
        p90_sum_top_total_us=p90_sum,
        threshold_sum_max_top_total_us=thr_sum,
    )


def _is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def _threshold_loosening_report(old_path: Path, new_path: Path) -> list[dict[str, Any]]:
    if not old_path.is_file():
        return []

    old_doc = _load_json(old_path)
    new_doc = _load_json(new_path)
    old_rows = {
        str((row or {}).get("script") or ""): row
        for row in (old_doc.get("rows", []) or [])
        if isinstance(row, dict)
    }
    new_rows = {
        str((row or {}).get("script") or ""): row
        for row in (new_doc.get("rows", []) or [])
        if isinstance(row, dict)
    }

    loosening: list[dict[str, Any]] = []
    for script, old_row in old_rows.items():
        new_row = new_rows.get(script)
        if new_row is None:
            loosening.append(
                {
                    "script": script,
                    "threshold": "*",
                    "old_value": "present",
                    "new_value": "missing",
                    "reason": "row_removed",
                }
            )
            continue

        old_thresholds = (old_row or {}).get("thresholds") or {}
        new_thresholds = (new_row or {}).get("thresholds") or {}
        for threshold_name, old_value in old_thresholds.items():
            if old_value is None:
                continue
            if not _is_number(old_value):
                continue

            new_present = threshold_name in new_thresholds
            new_value = new_thresholds.get(threshold_name)
            if not new_present or new_value is None:
                loosening.append(
                    {
                        "script": script,
                        "threshold": threshold_name,
                        "old_value": old_value,
                        "new_value": None,
                        "reason": "threshold_removed",
                    }
                )
                continue
            if not _is_number(new_value):
                loosening.append(
                    {
                        "script": script,
                        "threshold": threshold_name,
                        "old_value": old_value,
                        "new_value": new_value,
                        "reason": "threshold_non_numeric",
                    }
                )
                continue

            if threshold_name.startswith("min_"):
                is_looser = float(new_value) < float(old_value)
                reason = "min_decreased"
            else:
                is_looser = float(new_value) > float(old_value)
                reason = "max_increased"
            if is_looser:
                loosening.append(
                    {
                        "script": script,
                        "threshold": threshold_name,
                        "old_value": old_value,
                        "new_value": new_value,
                        "reason": reason,
                    }
                )

    return loosening


def _measured_value_for_threshold(row: dict[str, Any], threshold_name: str) -> Any:
    metric = THRESHOLD_TO_MEASURED_METRIC.get(threshold_name)
    if metric is None:
        return None
    for section_name in ("measured_max", "threshold_seed", "measured_p95", "measured_p90"):
        section = row.get(section_name) or {}
        if not isinstance(section, dict):
            continue
        value = section.get(metric)
        if _is_number(value):
            return value
    return None


def _clamp_threshold_loosening(
    *,
    old_path: Path,
    new_path: Path,
    source_baseline: str,
) -> list[dict[str, Any]]:
    if not old_path.is_file():
        return []

    old_doc = _load_json(old_path)
    new_doc = _load_json(new_path)
    old_rows = {
        str((row or {}).get("script") or ""): row
        for row in (old_doc.get("rows", []) or [])
        if isinstance(row, dict)
    }
    new_rows = {
        str((row or {}).get("script") or ""): row
        for row in (new_doc.get("rows", []) or [])
        if isinstance(row, dict)
    }

    clamps: list[dict[str, Any]] = []
    for script, old_row in old_rows.items():
        new_row = new_rows.get(script)
        if new_row is None:
            continue
        old_thresholds = (old_row or {}).get("thresholds") or {}
        new_thresholds = new_row.setdefault("thresholds", {})
        if not isinstance(new_thresholds, dict):
            continue

        for threshold_name, old_value in old_thresholds.items():
            if old_value is None or not _is_number(old_value):
                continue
            new_value = new_thresholds.get(threshold_name)
            if threshold_name.startswith("min_"):
                should_clamp = (not _is_number(new_value)) or float(new_value) < float(old_value)
                reason = "min_clamped_to_existing"
            else:
                should_clamp = (not _is_number(new_value)) or float(new_value) > float(old_value)
                reason = "max_clamped_to_existing"
                measured_value = _measured_value_for_threshold(new_row, threshold_name)
                if _is_number(measured_value) and float(measured_value) > float(old_value):
                    should_clamp = False
            if not should_clamp:
                continue

            new_thresholds[threshold_name] = old_value
            clamps.append(
                {
                    "script": script,
                    "threshold": threshold_name,
                    "old_value": old_value,
                    "new_value_before_clamp": new_value,
                    "new_value_after_clamp": old_value,
                    "reason": reason,
                }
            )

    if clamps:
        new_doc["threshold_clamp_policy"] = {
            "schema_version": 1,
            "mode": "no_threshold_loosening",
            "source_baseline": source_baseline,
            "clamps": clamps,
        }
        new_path.write_text(json.dumps(new_doc, indent=2, sort_keys=False) + "\n", encoding="utf-8")

    return clamps


def _rewrite_checked_in_out_path(path: Path, out_path_value: str) -> None:
    doc = _load_json(path)
    if not isinstance(doc, dict):
        return
    if doc.get("kind") != "perf_baseline":
        return
    doc["out_path"] = out_path_value
    path.write_text(json.dumps(doc, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Select a stable perf baseline from multiple `fretboard-dev diag perf` candidates.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--suite", default="ui-gallery-steady")
    ap.add_argument("--baseline-out", required=True)
    ap.add_argument("--preset", action="append", default=[], help="Seed policy preset JSON (repeatable).")
    ap.add_argument("--candidates", type=int, default=2)
    ap.add_argument("--validate-runs", type=int, default=3)
    ap.add_argument(
        "--validate-repeat",
        type=int,
        default=0,
        help="Repeat count for validation runs. Defaults to --repeat when unset/0.",
    )
    ap.add_argument("--repeat", type=int, default=7)
    ap.add_argument("--warmup-frames", type=int, default=5)
    ap.add_argument("--headroom-pct", type=int, default=20)
    ap.add_argument(
        "--threshold-surface",
        default="ui",
        choices=["ui", "ui-renderer-payload", "renderer-payload", "renderer", "all"],
        help=(
            "Forwarded to `diag perf --perf-baseline-threshold-surface`. "
            "Use 'ui' for resize/layout contracts; 'ui-renderer-payload' when UI thresholds plus renderer payload "
            "metrics should be gated; 'renderer-payload' for payload-only gates; 'renderer' or 'all' for "
            "renderer-focused gates."
        ),
    )
    ap.add_argument(
        "--ui-threshold-mode",
        default="",
        choices=["", "top", "frame_p95", "frame-p95", "top_and_frame_p95", "top-and-frame-p95"],
        help=(
            "Forwarded to `diag perf --perf-baseline-ui-threshold-mode` when set. "
            "Use frame_p95 for typical-frame contracts, top for tail contracts, and "
            "top_and_frame_p95 when a contract intentionally gates both."
        ),
    )
    ap.add_argument("--work-dir", default="")
    ap.add_argument("--launch-bin", default="target/release/fret-ui-gallery")
    ap.add_argument("--timeout-ms", type=int, default=300_000)
    ap.add_argument(
        "--prewarm-script",
        action="append",
        default=[],
        help="Forwarded to `fretboard-dev diag perf --prewarm-script <script.json>` (repeatable).",
    )
    ap.add_argument(
        "--prelude-script",
        action="append",
        default=[],
        help="Forwarded to `fretboard-dev diag perf --prelude-script <script.json>` (repeatable).",
    )
    ap.add_argument(
        "--prelude-each-run",
        action="store_true",
        default=False,
        help="Forwarded to `fretboard-dev diag perf --prelude-each-run`.",
    )
    ap.add_argument(
        "--reuse-launch-per-script",
        action="store_true",
        default=False,
        help=(
            "Forwarded to `fretboard-dev diag perf --reuse-launch-per-script`. "
            "Use this for suites whose scripts declare conflicting launch env defaults."
        ),
    )
    ap.add_argument(
        "--no-default-suite-hooks",
        action="store_true",
        default=False,
        help="Do not add the default font prewarm and reset-diagnostics prelude scripts.",
    )
    ap.add_argument(
        "--env",
        action="append",
        default=[],
        help="Forwarded to `fretboard-dev diag perf --env KEY=VALUE` (repeatable).",
    )
    ap.add_argument(
        "--allow-failures",
        action="store_true",
        default=False,
        help="Copy the best candidate even if validation failures remain.",
    )
    ap.add_argument(
        "--allow-threshold-loosening",
        action="store_true",
        default=False,
        help=(
            "Allow the selected candidate to loosen thresholds compared with an existing "
            "--baseline-out file. Without this, threshold increases/removals fail selection."
        ),
    )
    ap.add_argument(
        "--clamp-threshold-loosening",
        action="store_true",
        default=False,
        help=(
            "Before validating each candidate, clamp generated thresholds to the existing "
            "--baseline-out values when the candidate's measured value still fits the existing threshold."
        ),
    )

    args = ap.parse_args()

    workspace_root = _workspace_root()
    suite = str(args.suite)

    baseline_out = _resolve_workspace_path(workspace_root, args.baseline_out)
    preset_paths = [_resolve_workspace_path(workspace_root, p) for p in args.preset]
    launch_bin = _resolve_workspace_path(workspace_root, args.launch_bin)
    validate_repeat = int(args.validate_repeat) if int(args.validate_repeat) > 0 else int(args.repeat)

    work_dir = args.work_dir.strip()
    if not work_dir:
        work_dir = f"target/fret-diag-baseline-select-{suite}.{int(time.time())}"
    work_dir_path = _resolve_workspace_path(workspace_root, work_dir)
    work_dir_path.mkdir(parents=True, exist_ok=True)

    baseline_out.parent.mkdir(parents=True, exist_ok=True)

    env_specs = _split_env_specs(list(args.env))
    prewarm_script_specs = list(args.prewarm_script)
    prelude_script_specs = list(args.prelude_script)
    if not bool(args.no_default_suite_hooks):
        prewarm_script_specs.insert(0, DEFAULT_PREWARM_SCRIPT)
        prelude_script_specs.insert(0, DEFAULT_PRELUDE_SCRIPT)
    prewarm_script_paths = [_resolve_workspace_path(workspace_root, p) for p in prewarm_script_specs]
    prelude_script_paths = [_resolve_workspace_path(workspace_root, p) for p in prelude_script_specs]
    prewarm_scripts = [str(p) for p in prewarm_script_paths]
    prelude_scripts = [str(p) for p in prelude_script_paths]

    for hook_path in [*prewarm_script_paths, *prelude_script_paths]:
        if not hook_path.is_file():
            print(f"error: suite hook script not found: {hook_path}", file=sys.stderr)
            return 2

    print(f"[select] prewarm: {prewarm_scripts}")
    print(f"[select] prelude: {prelude_scripts}")

    candidate_results: list[dict[str, Any]] = []
    compare_existing_thresholds = baseline_out.is_file() and not bool(args.allow_threshold_loosening)
    clamp_existing_thresholds = baseline_out.is_file() and bool(args.clamp_threshold_loosening)
    best: tuple[int, int, int, int, str] | None = None

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
        ]
        for script in prewarm_scripts:
            cmd += ["--prewarm-script", script]
        for script in prelude_scripts:
            cmd += ["--prelude-script", script]
        if bool(args.prelude_each_run):
            cmd += ["--prelude-each-run"]
        cmd += [
            "--reuse-launch",
            "--sort",
            "time",
            "--json",
        ]
        if bool(args.reuse_launch_per_script):
            cmd += ["--reuse-launch-per-script"]
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
            "--perf-baseline-threshold-surface",
            str(args.threshold_surface),
        ]
        if str(args.ui_threshold_mode).strip():
            cmd += ["--perf-baseline-ui-threshold-mode", str(args.ui_threshold_mode)]
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
                    "threshold_clamp_count": 0,
                    "threshold_clamps": [],
                    "threshold_loosening_count": 10_000 if compare_existing_thresholds else 0,
                    "threshold_loosening": [],
                    "suite_p90_total_time_us_sum": 2**31 - 1,
                    "threshold_sum_max_top_total_us": 2**31 - 1,
                    "validate_runs": [],
                }
            )
            continue

        threshold_clamps = (
            _clamp_threshold_loosening(
                old_path=baseline_out,
                new_path=candidate_baseline,
                source_baseline=str(args.baseline_out),
            )
            if clamp_existing_thresholds
            else []
        )
        if threshold_clamps:
            print(f"[candidate] name={candidate_name} threshold_clamp_count={len(threshold_clamps)}")

        fail_total = 0
        validate_runs: list[dict[str, Any]] = []
        for j in range(1, int(args.validate_runs) + 1):
            validation_out_dir = work_dir_path / f"{candidate_name}-validate-{j}"
            validation_out_dir.mkdir(parents=True, exist_ok=True)
            print(f"[validate] candidate={i} run={j}")
            vcmd = diag_cmd_common(validation_out_dir)
            vcmd += [
                "--repeat",
                str(validate_repeat),
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
        threshold_loosening = (
            _threshold_loosening_report(baseline_out, candidate_baseline)
            if compare_existing_thresholds
            else []
        )
        threshold_loosening_count = len(threshold_loosening)

        print(
            f"[candidate] name={candidate_name} fail_total={fail_total} "
            f"threshold_loosening_count={threshold_loosening_count} "
            f"suite_p90_total_time_us_sum={p90_sum} threshold_sum={thr_sum}"
        )

        candidate_results.append(
            {
                "name": candidate_name,
                "baseline": str(candidate_baseline),
                "fail_total": int(fail_total),
                "threshold_clamp_count": int(len(threshold_clamps)),
                "threshold_clamps": threshold_clamps[:50],
                "threshold_loosening_count": int(threshold_loosening_count),
                "threshold_loosening": threshold_loosening[:50],
                "suite_p90_total_time_us_sum": int(p90_sum),
                "threshold_sum_max_top_total_us": int(thr_sum),
                "validate_runs": validate_runs,
            }
        )

        key = (
            int(fail_total),
            int(threshold_loosening_count),
            int(p90_sum),
            int(thr_sum),
            str(candidate_baseline),
        )
        if best is None or key < best:
            best = key

    if best is None:
        print("error: no candidate selected", file=sys.stderr)
        return 3

    selected_baseline_path = Path(best[4])
    if int(best[0]) != 0 and not bool(args.allow_failures):
        summary = {
            "schema_version": 1,
            "kind": "perf_baseline_selection",
            "suite": suite,
            "baseline_out": str(baseline_out),
            "threshold_surface": str(args.threshold_surface),
            "ui_threshold_mode": str(args.ui_threshold_mode or ""),
            "validate_repeat": int(validate_repeat),
            "allow_failures": False,
            "allow_threshold_loosening": bool(args.allow_threshold_loosening),
            "clamp_threshold_loosening": bool(args.clamp_threshold_loosening),
            "selected_candidate": str(selected_baseline_path),
            "selected_fail_total": int(best[0]),
            "selected_threshold_loosening_count": int(best[1]),
            "candidates": candidate_results,
        }
        summary_path = work_dir_path / "selection-summary.json"
        summary_path.write_text(
            json.dumps(summary, indent=2, sort_keys=False) + "\n",
            encoding="utf-8",
        )
        print(
            f"error: selected candidate still has validation failures "
            f"(fail_total={int(best[0])}). See: {summary_path}",
            file=sys.stderr,
        )
        return 4

    selected_threshold_loosening = (
        _threshold_loosening_report(baseline_out, selected_baseline_path)
        if compare_existing_thresholds
        else []
    )
    if selected_threshold_loosening and not bool(args.allow_threshold_loosening):
        summary = {
            "schema_version": 1,
            "kind": "perf_baseline_selection",
            "suite": suite,
            "baseline_out": str(baseline_out),
            "threshold_surface": str(args.threshold_surface),
            "ui_threshold_mode": str(args.ui_threshold_mode or ""),
            "validate_repeat": int(validate_repeat),
            "allow_failures": bool(args.allow_failures),
            "allow_threshold_loosening": False,
            "clamp_threshold_loosening": bool(args.clamp_threshold_loosening),
            "selected_candidate": str(selected_baseline_path),
            "selected_fail_total": int(best[0]),
            "selected_threshold_loosening_count": len(selected_threshold_loosening),
            "selected_threshold_loosening": selected_threshold_loosening[:50],
            "candidates": candidate_results,
        }
        summary_path = work_dir_path / "selection-summary.json"
        summary_path.write_text(
            json.dumps(summary, indent=2, sort_keys=False) + "\n",
            encoding="utf-8",
        )
        print(
            f"error: selected candidate loosens existing thresholds "
            f"(count={len(selected_threshold_loosening)}). See: {summary_path}",
            file=sys.stderr,
        )
        return 5

    shutil.copyfile(selected_baseline_path, baseline_out)
    _rewrite_checked_in_out_path(baseline_out, str(args.baseline_out))

    summary = {
        "schema_version": 1,
        "kind": "perf_baseline_selection",
        "suite": suite,
        "baseline_out": str(baseline_out),
        "suite_hooks": {
            "prewarm": prewarm_scripts,
            "prelude": prelude_scripts,
            "prelude_each_run": bool(args.prelude_each_run),
            "reuse_launch_per_script": bool(args.reuse_launch_per_script),
            "default_suite_hooks": not bool(args.no_default_suite_hooks),
        },
        "threshold_surface": str(args.threshold_surface),
        "ui_threshold_mode": str(args.ui_threshold_mode or ""),
        "validate_repeat": int(validate_repeat),
        "allow_failures": bool(args.allow_failures),
        "allow_threshold_loosening": bool(args.allow_threshold_loosening),
        "clamp_threshold_loosening": bool(args.clamp_threshold_loosening),
        "best_candidate": {
            "path": str(selected_baseline_path),
            "fail_total": int(best[0]),
            "threshold_loosening_count": int(best[1]),
            "suite_p90_total_time_us_sum": int(best[2]),
            "threshold_sum_max_top_total_us": int(best[3]),
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
