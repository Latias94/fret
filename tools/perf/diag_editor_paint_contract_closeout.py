#!/usr/bin/env python3
"""
Run the local closeout gates for synced editor paint contract artifacts.

This helper does not rerun perf probes. It verifies the copied Windows RTX4090
validation directories, then runs the local repo gates that must still be green
before P1.5 can close.
"""

from __future__ import annotations

import argparse
import json
import shlex
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

import diag_editor_paint_contract_verify_artifacts as verify


DEFAULT_MATRIX = "docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md"
DEFAULT_WORKSTREAM_JSON = "docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json"
OWNER_DECISION_LOW_P95_US = 150
OWNER_DECISION_DOMINANCE_RATIO = 1.25


def _workspace_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _write_json(path: Path, v: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(v, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def _run(cmd: list[str], cwd: Path, stdout_path: Path, stderr_path: Path) -> dict[str, Any]:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    started = time.time()
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        p = subprocess.run(cmd, cwd=str(cwd), stdout=out, stderr=err)
    return {
        "cmd": cmd,
        "rc": int(p.returncode),
        "elapsed_ms": int((time.time() - started) * 1000.0),
        "stdout": str(stdout_path),
        "stderr": str(stderr_path),
    }


def build_plan(
    *,
    python_bin: str,
    matrix: str,
    workstream_json: str,
    skip_diff_check: bool,
) -> list[dict[str, Any]]:
    plan = [
        {
            "name": "perf-baseline-matrix-audit",
            "cmd": [
                python_bin,
                "tools/perf/audit_perf_baselines.py",
                "--matrix",
                matrix,
                "--strict",
            ],
        },
        {
            "name": "workstream-json-valid",
            "cmd": [
                python_bin,
                "-m",
                "json.tool",
                workstream_json,
            ],
        },
        {
            "name": "workstream-catalog",
            "cmd": [
                python_bin,
                "tools/check_workstream_catalog.py",
            ],
        },
    ]
    if not skip_diff_check:
        plan.append(
            {
                "name": "git-diff-check",
                "cmd": [
                    "git",
                    "diff",
                    "--check",
                ],
            }
        )
    return plan


def verifier_date_tag(verifier: dict[str, Any], section: str) -> str | None:
    value = verifier.get(section)
    if not isinstance(value, dict):
        return None
    date_tag = value.get("date_tag")
    return date_tag if isinstance(date_tag, str) and date_tag else None


def _number(value: Any) -> float | None:
    if isinstance(value, bool):
        return None
    if isinstance(value, (int, float)):
        return float(value)
    return None


def _max_metric(rows: list[dict[str, Any]], key: str) -> tuple[float | None, str | None]:
    best_value: float | None = None
    best_probe: str | None = None
    for row in rows:
        value = _number(row.get(key))
        if value is None:
            continue
        if best_value is None or value > best_value:
            best_value = value
            probe = row.get("probe")
            best_probe = probe if isinstance(probe, str) else None
    return best_value, best_probe


def _decision_rows_from_verifier(verifier: dict[str, Any]) -> list[dict[str, Any]]:
    attribution = verifier.get("attribution")
    if not isinstance(attribution, dict):
        return []
    steps = attribution.get("steps")
    if not isinstance(steps, dict):
        return []

    rows: list[dict[str, Any]] = []
    for probe, report in steps.items():
        if not isinstance(report, dict):
            continue
        inputs = report.get("decision_inputs")
        if not isinstance(inputs, dict):
            continue
        hotspots = inputs.get("paint_widget_hotspot_summary")
        if not isinstance(hotspots, dict):
            hotspots = {}
        rows.append(
            {
                "probe": probe,
                "paint_widget_p95_us": inputs.get("paint_widget_p95_us"),
                "canvas_exclusive_p95_us": hotspots.get("canvas_exclusive_p95_us"),
                "renderer_prepare_text_p95_us": inputs.get("renderer_prepare_text_p95_us"),
                "renderer_encode_scene_p95_us": inputs.get("renderer_encode_scene_p95_us"),
                "renderer_upload_p95_us": inputs.get("renderer_upload_p95_us"),
                "code_editor_total_p95_us": inputs.get("code_editor_total_p95_us"),
            }
        )
    return rows


def decide_next_owner(verifier: dict[str, Any]) -> dict[str, Any]:
    if not bool(verifier.get("ok")):
        return {
            "status": "incomplete",
            "owner": None,
            "action": "wait-for-valid-artifacts",
            "reason": "artifact verifier is not ok",
            "probes": [],
        }

    rows = _decision_rows_from_verifier(verifier)
    if not rows:
        return {
            "status": "incomplete",
            "owner": None,
            "action": "wait-for-attribution-decision-inputs",
            "reason": "verified attribution report has no decision_inputs",
            "probes": [],
        }

    paint_widget_p95, paint_probe = _max_metric(rows, "paint_widget_p95_us")
    canvas_p95, canvas_probe = _max_metric(rows, "canvas_exclusive_p95_us")
    renderer_text_p95, renderer_probe = _max_metric(rows, "renderer_prepare_text_p95_us")

    paint_score = paint_widget_p95 or 0.0
    renderer_score = renderer_text_p95 or 0.0

    if paint_score < OWNER_DECISION_LOW_P95_US and renderer_score < OWNER_DECISION_LOW_P95_US:
        owner = "no-code-change"
        action = "lock-gates-and-docs"
        reason = (
            f"paint.widget and renderer text p95 are both below "
            f"{OWNER_DECISION_LOW_P95_US}us on the verified attribution artifact"
        )
    elif renderer_score >= paint_score * OWNER_DECISION_DOMINANCE_RATIO:
        owner = "renderer-text-prepare"
        action = "open-glyph-text-index-atlas-residency-slice"
        reason = "renderer text prepare is the dominant verified attribution owner"
    else:
        owner = "canvas-paint-replay"
        action = "open-canvas-paint-replay-slice"
        reason = "paint.widget / Canvas remains the dominant verified attribution owner"

    return {
        "status": "decided",
        "owner": owner,
        "action": action,
        "reason": reason,
        "thresholds": {
            "low_owner_p95_us": OWNER_DECISION_LOW_P95_US,
            "dominance_ratio": OWNER_DECISION_DOMINANCE_RATIO,
        },
        "scores": {
            "paint_widget_p95_us": paint_widget_p95,
            "paint_widget_probe": paint_probe,
            "canvas_exclusive_p95_us": canvas_p95,
            "canvas_probe": canvas_probe,
            "renderer_prepare_text_p95_us": renderer_text_p95,
            "renderer_probe": renderer_probe,
        },
        "probes": rows,
    }


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run the local closeout gates for synced editor paint contract artifacts.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("validation_dir", help="Directory produced by diag_editor_paint_contract_validate.py.")
    ap.add_argument(
        "--attribution-dir",
        default="",
        help="Second directory produced with --with-paint-perf. Required unless --dry-run is used.",
    )
    ap.add_argument("--matrix", default=DEFAULT_MATRIX)
    ap.add_argument("--workstream-json", default=DEFAULT_WORKSTREAM_JSON)
    ap.add_argument("--python-bin", default=sys.executable)
    ap.add_argument("--skip-diff-check", action="store_true", default=False)
    ap.add_argument("--dry-run", action="store_true", default=False)
    ap.add_argument(
        "--out-report",
        default="",
        help="Path for a JSON report. Defaults to <validation_dir>/editor-paint-contract-closeout.summary.json.",
    )
    args = ap.parse_args()

    workspace_root = _workspace_root()
    validation_dir = Path(str(args.validation_dir))
    attribution_arg = str(args.attribution_dir).strip()
    if not attribution_arg and not bool(args.dry_run):
        print("error: --attribution-dir is required for non-dry-run closeout", file=sys.stderr)
        return 2
    attribution_dir = Path(attribution_arg) if attribution_arg else None
    out_report = (
        Path(str(args.out_report))
        if str(args.out_report).strip()
        else validation_dir / "editor-paint-contract-closeout.summary.json"
    )

    plan = build_plan(
        python_bin=str(args.python_bin),
        matrix=str(args.matrix),
        workstream_json=str(args.workstream_json),
        skip_diff_check=bool(args.skip_diff_check),
    )

    if bool(args.dry_run):
        summary = {
            "kind": "editor_paint_contract_closeout_plan",
            "dry_run": True,
            "ok": True,
            "validation_dir": str(validation_dir),
            "attribution_dir": str(attribution_dir) if attribution_dir is not None else None,
            "verifier": {
                "skipped": True,
                "reason": "dry-run",
            },
            "owner_decision": {
                "status": "skipped",
                "reason": "dry-run",
            },
            "steps": plan,
        }
        _write_json(out_report, summary)
        print(f"[closeout] dry-run report: {out_report}")
        return 0

    verifier = verify.verify_artifact_dirs(validation_dir, attribution_dir)

    if not bool(verifier.get("ok")):
        owner_decision = decide_next_owner(verifier)
        summary = {
            "kind": "editor_paint_contract_closeout_summary",
            "ok": False,
            "validation_dir": str(validation_dir),
            "attribution_dir": str(attribution_dir) if attribution_dir is not None else None,
            "validation_date_tag": verifier_date_tag(verifier, "validation"),
            "attribution_date_tag": verifier_date_tag(verifier, "attribution"),
            "verifier": verifier,
            "owner_decision": owner_decision,
            "steps": [],
        }
        _write_json(out_report, summary)
        print(f"[closeout] FAIL. Verifier report: {out_report}", file=sys.stderr)
        return 1

    print(f"[closeout] verifier ok. running local gates from {workspace_root}")
    owner_decision = decide_next_owner(verifier)
    step_results: list[dict[str, Any]] = []
    pass_all = True
    for step in plan:
        name = str(step["name"])
        cmd = list(step["cmd"])
        step_dir = out_report.parent / "closeout-logs" / name
        stdout_path = step_dir / "stdout.log"
        stderr_path = step_dir / "stderr.log"
        print(f"[closeout] running {name}: {shlex.join(cmd)}")
        result = _run(cmd, workspace_root, stdout_path, stderr_path)
        step_results.append(result)
        pass_all = pass_all and result["rc"] == 0
        if result["rc"] != 0:
            break

    summary = {
        "kind": "editor_paint_contract_closeout_summary",
        "ok": pass_all,
        "validation_dir": str(validation_dir),
        "attribution_dir": str(attribution_dir) if attribution_dir is not None else None,
        "validation_date_tag": verifier_date_tag(verifier, "validation"),
        "attribution_date_tag": verifier_date_tag(verifier, "attribution"),
        "verifier": verifier,
        "owner_decision": owner_decision,
        "steps": step_results,
    }
    _write_json(out_report, summary)

    if not pass_all:
        print(f"[closeout] FAIL. Summary: {out_report}", file=sys.stderr)
        return 1
    print(f"[closeout] PASS. Summary: {out_report}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
