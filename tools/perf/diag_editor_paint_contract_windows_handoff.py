#!/usr/bin/env python3
"""
Run the Windows RTX4090 editor paint contract handoff.

This helper does not replace the target-machine validation itself. It packages
the exact sequence needed for the formal closeout:

1. preflight
2. baseline validation
3. attribution validation
4. artifact verification
5. local closeout gates

Use ``--dry-run`` on a non-target machine to inspect the command plan without
producing misleading validation artifacts.
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

import diag_editor_paint_contract_validate as validate


def _workspace_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _resolve_workspace_path(workspace_root: Path, p: str) -> Path:
    path = Path(p)
    if path.is_absolute():
        return path
    return workspace_root / path


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


def _default_date_tag() -> str:
    return time.strftime("%Y%m%d-%H%M%S")


def _default_out_dir(date_tag: str) -> str:
    return f"target/fret-diag/editor-paint-contract-windows-handoff-{date_tag}"


def _validation_dir(date_tag: str) -> str:
    return validate._default_out_dir(date_tag)


def _attribution_dir(date_tag: str) -> str:
    return validate._default_out_dir(f"{date_tag}-attrib")


def build_plan(
    *,
    python_bin: str,
    date_tag: str,
    out_dir: str,
    skip_preflight: bool,
) -> list[dict[str, Any]]:
    validation_dir = _validation_dir(date_tag)
    attribution_dir = _attribution_dir(date_tag)

    plan: list[dict[str, Any]] = []
    if not skip_preflight:
        plan.append(
            {
                "name": "preflight",
                "out_dir": f"{out_dir}/preflight",
                "cmd": [
                    python_bin,
                    "tools/perf/diag_editor_paint_contract_preflight.py",
                    "--out-summary",
                    f"{out_dir}/preflight/summary.json",
                ],
            }
        )
    plan.extend(
        [
            {
                "name": "baseline-validation",
                "out_dir": validation_dir,
                "cmd": [
                    python_bin,
                    "tools/perf/diag_editor_paint_contract_validate.py",
                    "--date-tag",
                    date_tag,
                    "--skip-preflight",
                ],
            },
            {
                "name": "attribution-validation",
                "out_dir": attribution_dir,
                "cmd": [
                    python_bin,
                    "tools/perf/diag_editor_paint_contract_validate.py",
                    "--date-tag",
                    f"{date_tag}-attrib",
                    "--skip-preflight",
                    "--with-paint-perf",
                ],
            },
            {
                "name": "verify-artifacts",
                "out_dir": f"{out_dir}/verify",
                "cmd": [
                    python_bin,
                    "tools/perf/diag_editor_paint_contract_verify_artifacts.py",
                    validation_dir,
                    "--attribution-dir",
                    attribution_dir,
                    "--out-report",
                    f"{out_dir}/verify/artifact-verification.summary.json",
                ],
            },
            {
                "name": "closeout",
                "out_dir": f"{out_dir}/closeout",
                "cmd": [
                    python_bin,
                    "tools/perf/diag_editor_paint_contract_closeout.py",
                    validation_dir,
                    "--attribution-dir",
                    attribution_dir,
                    "--out-report",
                    f"{out_dir}/closeout/editor-paint-contract-closeout.summary.json",
                ],
            },
        ]
    )
    return plan


def _validate_inputs(workspace_root: Path) -> list[str]:
    missing: list[str] = []
    for rel in [
        "tools/perf/diag_editor_paint_contract_preflight.py",
        "tools/perf/diag_editor_paint_contract_validate.py",
        "tools/perf/diag_editor_paint_contract_verify_artifacts.py",
        "tools/perf/diag_editor_paint_contract_closeout.py",
    ]:
        if not _resolve_workspace_path(workspace_root, rel).is_file():
            missing.append(rel)
    return missing


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run the Windows RTX4090 editor paint contract handoff.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--date-tag", default=_default_date_tag())
    ap.add_argument("--out-dir", default="")
    ap.add_argument("--python-bin", default=sys.executable)
    ap.add_argument("--skip-preflight", action="store_true", default=False)
    ap.add_argument("--dry-run", action="store_true", default=False)
    ap.add_argument(
        "--allow-non-windows",
        action="store_true",
        default=False,
        help="Allow executing the handoff from a non-Windows host for explicit debugging.",
    )
    args = ap.parse_args()

    workspace_root = _workspace_root()
    out_dir = str(args.out_dir).strip() or _default_out_dir(str(args.date_tag))
    out_dir_path = _resolve_workspace_path(workspace_root, out_dir)

    if (
        not bool(args.dry_run)
        and not sys.platform.startswith("win")
        and not bool(args.allow_non_windows)
    ):
        print(
            "error: the editor paint contract handoff must run on the target Windows host "
            "(use --dry-run to inspect the plan)",
            file=sys.stderr,
        )
        return 2

    missing = _validate_inputs(workspace_root)
    if missing:
        print("error: required handoff inputs are missing:", file=sys.stderr)
        for item in missing:
            print(f"  - {item}", file=sys.stderr)
        return 2

    plan = build_plan(
        python_bin=str(args.python_bin),
        date_tag=str(args.date_tag),
        out_dir=out_dir,
        skip_preflight=bool(args.skip_preflight),
    )

    if bool(args.dry_run):
        print(f"[handoff] out-dir={out_dir}")
        for step in plan:
            print(f"[handoff] {step['name']}: {shlex.join(step['cmd'])}")
        summary = {
            "kind": "editor_paint_contract_windows_handoff_plan",
            "dry_run": True,
            "date_tag": str(args.date_tag),
            "out_dir": out_dir,
            "steps": plan,
        }
        _write_json(out_dir_path / "handoff-plan.json", summary)
        print(f"[handoff] wrote plan: {out_dir_path / 'handoff-plan.json'}")
        return 0

    out_dir_path.mkdir(parents=True, exist_ok=True)
    print(f"[handoff] out-dir={out_dir_path}")
    step_results: list[dict[str, Any]] = []
    pass_all = True
    for step in plan:
        name = str(step["name"])
        cmd = list(step["cmd"])
        step_dir = out_dir_path / "runner-logs" / name
        stdout_path = step_dir / "stdout.log"
        stderr_path = step_dir / "stderr.log"
        _write_json(step_dir / "cmd.json", {"cmd": cmd})
        print(f"[handoff] running {name}: {shlex.join(cmd)}")
        result = _run(cmd, workspace_root, stdout_path, stderr_path)
        step_results.append(result)
        pass_all = pass_all and result["rc"] == 0

    summary = {
        "kind": "editor_paint_contract_windows_handoff_summary",
        "ok": pass_all,
        "date_tag": str(args.date_tag),
        "out_dir": out_dir,
        "validation_dir": _validation_dir(str(args.date_tag)),
        "attribution_dir": _attribution_dir(str(args.date_tag)),
        "steps": step_results,
    }
    _write_json(out_dir_path / "summary.json", summary)

    if not pass_all:
        print(f"[handoff] FAIL. Summary: {out_dir_path / 'summary.json'}", file=sys.stderr)
        return 1
    print(f"[handoff] PASS. Summary: {out_dir_path / 'summary.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
