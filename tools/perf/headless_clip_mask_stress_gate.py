#!/usr/bin/env python3
"""
Headless renderer perf gate for clip/mask stacks.

Why:
- Mask/clip semantics are core UI primitives and sit on hot fragment paths.
- WebGPU/WGSL portability constraints can force shader shape changes (uniformity-safe code).
- This gate keeps counters (draws/pipelines/binds/bytes/cache) within a stable envelope.

Timing is recorded for visibility but is not enforced by default.
"""

from __future__ import annotations

import argparse
import json
import os
import re
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


def _read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def _git_short_sha(workspace_root: Path) -> str:
    try:
        out = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"], cwd=workspace_root)
        return out.decode("utf-8").strip() or "UNKNOWN"
    except Exception:
        return "UNKNOWN"


_PERF_RE = re.compile(
    r"headless_renderer_perf:\s+"
    r"frames=(?P<frames>\d+)\s+"
    r"encode=(?P<encode_ms>[\d\.]+)ms\s+"
    r"prepare_svg=(?P<prepare_svg_ms>[\d\.]+)ms\s+"
    r"prepare_text=(?P<prepare_text_ms>[\d\.]+)ms\s+"
    r"draws=(?P<draw_calls>\d+)\s+"
    r"\(quad=(?P<quad_draw_calls>\d+)\s+"
    r"viewport=(?P<viewport_draw_calls>\d+)\s+"
    r"image=(?P<image_draw_calls>\d+)\s+"
    r"text=(?P<text_draw_calls>\d+)\s+"
    r"path=(?P<path_draw_calls>\d+)\s+"
    r"mask=(?P<mask_draw_calls>\d+)\s+"
    r"fs=(?P<fullscreen_draw_calls>\d+)\s+"
    r"clipmask=(?P<clip_mask_draw_calls>\d+)\)\s+"
    r"pipelines=(?P<pipeline_switches>\d+)\s+"
    r"binds=(?P<bind_group_switches>\d+)\s+"
    r"\(ubinds=(?P<uniform_bind_group_switches>\d+)\s+"
    r"tbinds=(?P<texture_bind_group_switches>\d+)\)\s+"
    r"scissor=(?P<scissor_sets>\d+)\s+"
    r"uniform=(?P<uniform_kb>\d+)KB\s+"
    r"instance=(?P<instance_kb>\d+)KB\s+"
    r"vertex=(?P<vertex_kb>\d+)KB\s+"
    r"cache_hits=(?P<cache_hits>\d+)\s+"
    r"cache_misses=(?P<cache_misses>\d+)"
)

_PIPELINES_RE = re.compile(
    r"headless_renderer_perf_pipelines:\s+"
    r"quad=(?P<pipeline_switches_quad>\d+)\s+"
    r"viewport=(?P<pipeline_switches_viewport>\d+)\s+"
    r"mask=(?P<pipeline_switches_mask>\d+)\s+"
    r"text_mask=(?P<pipeline_switches_text_mask>\d+)\s+"
    r"text_color=(?P<pipeline_switches_text_color>\d+)\s+"
    r"path=(?P<pipeline_switches_path>\d+)\s+"
    r"path_msaa=(?P<pipeline_switches_path_msaa>\d+)\s+"
    r"composite=(?P<pipeline_switches_composite>\d+)\s+"
    r"fullscreen=(?P<pipeline_switches_fullscreen>\d+)\s+"
    r"clip_mask=(?P<pipeline_switches_clip_mask>\d+)"
)


def _extract_last_match(text: str, rx: re.Pattern[str]) -> dict[str, str] | None:
    last: dict[str, str] | None = None
    for m in rx.finditer(text):
        last = dict(m.groupdict())
    return last


def _run(cmd: list[str], cwd: Path, env: dict[str, str], stdout_path: Path, stderr_path: Path) -> int:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        p = subprocess.run(cmd, cwd=str(cwd), env=env, stdout=out, stderr=err)
        return int(p.returncode)


def _to_ints(d: dict[str, str]) -> dict[str, int]:
    out: dict[str, int] = {}
    for k, v in d.items():
        if k.endswith("_ms"):
            continue
        out[k] = int(v)
    return out


def main() -> int:
    ap = argparse.ArgumentParser(
        description=(
            "Run a deterministic headless clip/mask workload and enforce stable counter thresholds "
            "(pipelines/draws/binds/bytes/cache)."
        ),
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--out-dir", default="")
    ap.add_argument(
        "--baseline",
        default="docs/workstreams/perf-baselines/clip-mask-stress-headless.windows-local.v1.json",
        help="Path to a JSON baseline with max thresholds.",
    )
    ap.add_argument("--frames", type=int, default=300)
    ap.add_argument("--group-n", type=int, default=32)
    ap.add_argument("--attempts", type=int, default=1)
    ap.add_argument("--release", action="store_true", default=True)
    ap.add_argument("--no-release", action="store_false", dest="release")
    ap.add_argument(
        "--cargo-target-dir",
        default="",
        help="Optional override for CARGO_TARGET_DIR (leave empty to use the workspace default).",
    )
    ap.add_argument("--headroom-pct", type=int, default=0, help="Optional extra headroom on top of baseline thresholds.")

    args = ap.parse_args()

    if int(args.attempts) < 1:
        print("error: --attempts must be >= 1", file=sys.stderr)
        return 2
    if int(args.frames) < 1:
        print("error: --frames must be >= 1", file=sys.stderr)
        return 2
    if int(args.group_n) < 1:
        print("error: --group-n must be >= 1", file=sys.stderr)
        return 2

    workspace_root = _workspace_root()
    sha = _git_short_sha(workspace_root)

    out_dir = str(args.out_dir).strip()
    if not out_dir:
        out_dir = f"target/perf-gates/clip-mask-stress-headless.{sha}.{time.strftime('%Y%m%d-%H%M%S')}"
    out_dir_path = _resolve_workspace_path(workspace_root, out_dir)
    out_dir_path.mkdir(parents=True, exist_ok=True)

    baseline_path = _resolve_workspace_path(workspace_root, str(args.baseline).strip())
    if not baseline_path.is_file():
        print(f"error: baseline not found: {baseline_path}", file=sys.stderr)
        return 2
    baseline = _read_json(baseline_path)
    if not isinstance(baseline, dict) or int(baseline.get("schema_version", 0) or 0) != 1:
        print(f"error: invalid baseline schema: {baseline_path}", file=sys.stderr)
        return 2
    thresholds = baseline.get("thresholds")
    if not isinstance(thresholds, dict):
        print(f"error: baseline missing thresholds: {baseline_path}", file=sys.stderr)
        return 2

    cmd = ["cargo", "run", "-q", "-p", "fret-clip-mask-stress"]
    if bool(args.release):
        cmd.append("--release")
    cmd += ["--", "--headless", "--frames", str(int(args.frames)), "--group-n", str(int(args.group_n))]

    gate_env = dict(os.environ)
    gate_env["FRET_RENDERER_PERF_PIPELINES"] = "1"
    cargo_target_dir = str(args.cargo_target_dir).strip()
    if cargo_target_dir:
        gate_env["CARGO_TARGET_DIR"] = cargo_target_dir

    print(f"[gate] clip-mask-stress-headless -> {out_dir_path} (attempts={int(args.attempts)})")
    print(f"[gate] baseline: {baseline_path}")
    print("[gate] cmd:", " ".join(cmd))

    passes = 0
    fails = 0
    selected_attempt_dir: Path | None = None
    selected_metrics: dict[str, int] | None = None
    attempt_summaries: list[dict[str, Any]] = []

    for i in range(1, int(args.attempts) + 1):
        attempt_dir = out_dir_path / f"attempt-{i}"
        attempt_dir.mkdir(parents=True, exist_ok=True)

        stdout_path = attempt_dir / "stdout.log"
        stderr_path = attempt_dir / "stderr.log"
        rc = _run(cmd, workspace_root, gate_env, stdout_path, stderr_path)

        stdout_text = stdout_path.read_text(encoding="utf-8", errors="replace")
        perf = _extract_last_match(stdout_text, _PERF_RE)
        pipes = _extract_last_match(stdout_text, _PIPELINES_RE)
        if rc != 0 or perf is None:
            fails += 1
            attempt_summaries.append(
                {
                    "attempt": i,
                    "rc": rc,
                    "status": "fail",
                    "reason": "missing perf snapshot" if perf is None else "non-zero exit",
                }
            )
            continue

        metrics = _to_ints(perf)
        if pipes is not None:
            metrics.update(_to_ints(pipes))

        extra_headroom = max(0, int(args.headroom_pct))
        ok = True
        violations: list[str] = []
        for k, v in thresholds.items():
            if not isinstance(v, int):
                ok = False
                violations.append(f"{k}: invalid threshold type ({type(v)})")
                continue
            if k not in metrics:
                ok = False
                violations.append(f"{k}: missing metric in output")
                continue
            limit = int(v) * (100 + extra_headroom) // 100
            if int(metrics[k]) > limit:
                ok = False
                violations.append(f"{k}: got={metrics[k]} limit={limit}")

        if ok:
            passes += 1
            selected_attempt_dir = attempt_dir
            selected_metrics = metrics
            attempt_summaries.append({"attempt": i, "rc": rc, "status": "pass"})
            break

        fails += 1
        attempt_summaries.append(
            {"attempt": i, "rc": rc, "status": "fail", "violations": violations[:16]}
        )

    summary_path = out_dir_path / "summary.json"
    summary_path.write_text(
        json.dumps(
            {
                "suite": "clip-mask-stress-headless",
                "baseline": str(baseline_path),
                "cmd": cmd,
                "passes": passes,
                "fails": fails,
                "selected_attempt": str(selected_attempt_dir) if selected_attempt_dir else "",
                "selected_metrics": selected_metrics or {},
                "attempts": attempt_summaries,
            },
            indent=2,
            sort_keys=False,
        )
        + "\n",
        encoding="utf-8",
    )

    if passes > 0:
        print(f"[gate] PASS (summary: {summary_path})")
        return 0

    print(f"[gate] FAIL (summary: {summary_path})", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())

