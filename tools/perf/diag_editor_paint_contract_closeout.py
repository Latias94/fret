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
            "steps": plan,
        }
        _write_json(out_report, summary)
        print(f"[closeout] dry-run report: {out_report}")
        return 0

    verifier = verify.verify_artifact_dirs(validation_dir, attribution_dir)

    if not bool(verifier.get("ok")):
        summary = {
            "kind": "editor_paint_contract_closeout_summary",
            "ok": False,
            "validation_dir": str(validation_dir),
            "attribution_dir": str(attribution_dir) if attribution_dir is not None else None,
            "verifier": verifier,
            "steps": [],
        }
        _write_json(out_report, summary)
        print(f"[closeout] FAIL. Verifier report: {out_report}", file=sys.stderr)
        return 1

    print(f"[closeout] verifier ok. running local gates from {workspace_root}")
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
        "verifier": verifier,
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
