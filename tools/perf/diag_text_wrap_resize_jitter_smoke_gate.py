#!/usr/bin/env python3
"""
Smoke gate for editor-grade text wrapping under resize jitter.

This is intentionally a *catastrophic regression* guard, not a calibrated baseline gate:

- Runs a text-wrap focused diag perf script (UI gallery).
- Enforces coarse top-frame time caps (configurable).
- Produces a portable out-dir with the worst-bundle evidence for attribution.

If you need calibrated thresholds, prefer a suite baseline under:
  docs/workstreams/perf-baselines/
and run:
  tools/perf/diag_perf_baseline_select.py
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


def _maybe_with_exe_suffix(path: Path) -> Path:
    if path.is_file():
        return path
    if path.suffix:
        return path
    candidate = path.with_suffix(".exe")
    if candidate.is_file():
        return candidate
    return path


def _read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def _write_json(path: Path, v: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(v, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(
        description=(
            "Run a text-wrap focused `fretboard diag perf` scenario and enforce coarse time caps. "
            "Intended as a catastrophic regression guard for resize jitter + wrapping."
        ),
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument(
        "--script",
        default="tools/diag-scripts/ui-gallery-text-measure-overlay-window-resize-drag-jitter-steady.json",
    )
    ap.add_argument("--out-dir", default="")
    ap.add_argument("--launch-bin", default="target/release/fret-ui-gallery")
    ap.add_argument("--timeout-ms", type=int, default=300_000)
    ap.add_argument("--repeat", type=int, default=5)
    ap.add_argument("--warmup-frames", type=int, default=5)
    ap.add_argument(
        "--wgpu-backend",
        default="",
        help="Optional `FRET_WGPU_BACKEND` override passed to the launched app (e.g. dx12, vulkan).",
    )

    # Coarse caps (microseconds). Keep these intentionally generous.
    ap.add_argument("--max-top-total-us", type=int, default=250_000)  # 250ms
    ap.add_argument("--max-top-layout-us", type=int, default=200_000)  # 200ms
    ap.add_argument("--max-top-solve-us", type=int, default=150_000)  # 150ms

    args = ap.parse_args()

    workspace_root = _workspace_root()

    script_path = _resolve_workspace_path(workspace_root, str(args.script))
    if not script_path.is_file():
        print(f"error: script not found: {script_path}", file=sys.stderr)
        return 2

    out_dir = args.out_dir.strip()
    if not out_dir:
        out_dir = f"target/fret-diag-text-wrap-resize-jitter-smoke-{int(time.time())}"
    out_dir_path = _resolve_workspace_path(workspace_root, out_dir)
    out_dir_path.mkdir(parents=True, exist_ok=True)

    launch_bin_path = _resolve_workspace_path(workspace_root, str(args.launch_bin))
    launch_bin_path = _maybe_with_exe_suffix(launch_bin_path)
    if not launch_bin_path.is_file():
        print(
            f"error: launch bin not found: {launch_bin_path} (build it via `cargo build -p fret-ui-gallery --release`)",
            file=sys.stderr,
        )
        return 2

    cmd = [
        "cargo",
        "run",
        "-q",
        "-p",
        "fretboard",
        "--",
        "diag",
        "perf",
        str(script_path),
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
        "--max-top-total-us",
        str(int(args.max_top_total_us)),
        "--max-top-layout-us",
        str(int(args.max_top_layout_us)),
        "--max-top-solve-us",
        str(int(args.max_top_solve_us)),
        "--env",
        "FRET_UI_GALLERY_VIEW_CACHE=1",
        "--env",
        "FRET_UI_GALLERY_VIEW_CACHE_SHELL=1",
        "--env",
        "FRET_DIAG_SCRIPT_AUTO_DUMP=0",
        "--env",
        "FRET_DIAG_SEMANTICS=0",
    ]

    if str(args.wgpu_backend).strip():
        cmd.extend(["--env", f"FRET_WGPU_BACKEND={str(args.wgpu_backend).strip()}"])

    cmd.extend(
        [
        "--launch",
        "--",
        str(launch_bin_path),
        ]
    )

    stdout_path = out_dir_path / "stdout.json"
    stderr_path = out_dir_path / "stderr.log"
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        rc = int(
            subprocess.run(cmd, cwd=str(workspace_root), stdout=out, stderr=err).returncode
        )

    check_path = out_dir_path / "check.perf_thresholds.json"
    failures: list[object] | None = None
    if check_path.is_file():
        doc = _read_json(check_path)
        if isinstance(doc, dict) and isinstance(doc.get("failures"), list):
            failures = doc["failures"]

    summary = {
        "script": str(script_path),
        "out_dir": str(out_dir_path),
        "launch_bin": str(launch_bin_path),
        "cmd": cmd,
        "rc": rc,
        "check": {
            "path": str(check_path),
            "failures": 0 if failures is None else len(failures),
        },
        "stdout": str(stdout_path),
        "stderr": str(stderr_path),
    }
    _write_json(out_dir_path / "gate.summary.json", summary)

    if rc != 0:
        print(f"FAIL: fretboard returned non-zero ({rc})", file=sys.stderr)
        print(f"out-dir: {out_dir_path}", file=sys.stderr)
        return 1

    if failures is None:
        print("FAIL: missing or unreadable check.perf_thresholds.json", file=sys.stderr)
        print(f"out-dir: {out_dir_path}", file=sys.stderr)
        return 1

    if len(failures) != 0:
        print(f"FAIL: perf thresholds failed ({len(failures)} failures)", file=sys.stderr)
        print(f"out-dir: {out_dir_path}", file=sys.stderr)
        return 1

    print(f"PASS: {script_path} (out-dir: {out_dir_path})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
