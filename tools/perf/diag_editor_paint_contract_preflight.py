#!/usr/bin/env python3
"""
Preflight checks for the editor paint contract stabilization runbook.

This helper intentionally avoids running the long perf validation passes. It only verifies that the
script registry, required editor probe JSON files, and baseline matrix are ready before a target
Windows machine spends time on the formal contract run.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path


EDITOR_PROBE_SCRIPTS = [
    "tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-autoscroll-typical.json",
    "tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.json",
    "tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json",
]

REQUIRED_SCRIPT_ENV_DEFAULTS = {
    "FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY": "0",
}

BASELINE_MATRIX = "docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md"


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


def _run(cmd: list[str], cwd: Path) -> dict[str, object]:
    started = time.time()
    p = subprocess.run(cmd, cwd=str(cwd), text=True, capture_output=True)
    return {
        "cmd": cmd,
        "rc": int(p.returncode),
        "elapsed_ms": int((time.time() - started) * 1000.0),
        "stdout": p.stdout,
        "stderr": p.stderr,
    }


def check_script_contract(path: Path) -> dict[str, object]:
    started = time.time()
    errors: list[str] = []
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        errors.append(f"cannot read script JSON: {exc}")
        doc = None

    meta = doc.get("meta") if isinstance(doc, dict) else None
    env_defaults = meta.get("env_defaults") if isinstance(meta, dict) else None
    for key, expected in REQUIRED_SCRIPT_ENV_DEFAULTS.items():
        actual = env_defaults.get(key) if isinstance(env_defaults, dict) else None
        if actual != expected:
            errors.append(f"meta.env_defaults.{key} must be {expected!r}, got {actual!r}")

    return {
        "cmd": ["check-script-contract", str(path)],
        "rc": 1 if errors else 0,
        "elapsed_ms": int((time.time() - started) * 1000.0),
        "stdout": "",
        "stderr": "\n".join(errors),
    }


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run fast preflight checks for the editor paint contract stabilization runbook.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument(
        "--out-summary",
        default="target/fret-diag/editor-paint-contract-preflight/summary.json",
        help="Path for the JSON summary artifact.",
    )
    args = ap.parse_args()

    workspace_root = _workspace_root()
    checks: list[dict[str, object]] = []

    for script in EDITOR_PROBE_SCRIPTS:
        path = _resolve_workspace_path(workspace_root, script)
        if not path.is_file():
            checks.append(
                {
                    "cmd": [sys.executable, "-m", "json.tool", script],
                    "rc": 2,
                    "elapsed_ms": 0,
                    "stdout": "",
                    "stderr": f"script not found: {path}",
                }
            )
            continue
        checks.append(_run([sys.executable, "-m", "json.tool", str(path)], workspace_root))
        checks.append(check_script_contract(path))

    checks.append(_run([sys.executable, "tools/check_diag_scripts_registry.py"], workspace_root))
    checks.append(
        _run(
            [
                sys.executable,
                "tools/perf/audit_perf_baselines.py",
                "--matrix",
                BASELINE_MATRIX,
                "--strict",
            ],
            workspace_root,
        )
    )

    failures = [check for check in checks if int(check.get("rc", 1)) != 0]
    summary = {
        "kind": "editor_paint_contract_preflight",
        "ok": not failures,
        "workspace_root": str(workspace_root),
        "checks": checks,
        "failures": failures,
    }

    out_summary = _resolve_workspace_path(workspace_root, str(args.out_summary))
    _write_json(out_summary, summary)

    if failures:
        print(f"FAIL: editor paint contract preflight ({len(failures)} failures)")
        print(f"summary: {out_summary}")
        return 1

    print(f"PASS: editor paint contract preflight ({len(checks)} checks)")
    print(f"summary: {out_summary}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
