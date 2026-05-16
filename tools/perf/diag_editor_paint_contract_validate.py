#!/usr/bin/env python3
"""
Run the target-machine editor paint contract validation plan.

This is intentionally narrower than the preflight helper: it owns the long
Windows RTX4090 validation pass described by the editor paint stabilization
runbook. Use ``--dry-run`` on non-target machines to inspect the exact commands.
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


TARGET_PROFILE = "windows-rtx4090"

PREWARM_SCRIPT = "tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json"
PRELUDE_SCRIPT = "tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json"

RESIZE_SUITE = "ui-code-editor-resize-probes"
RESIZE_BASELINE = "docs/workstreams/perf-baselines/ui-code-editor-resize-probes.windows-rtx4090.v2.json"

TYPICAL_SCRIPT = (
    "tools/diag-scripts/ui-gallery/code-editor/"
    "ui-gallery-code-editor-torture-autoscroll-typical.json"
)
TYPICAL_BASELINE = (
    "docs/workstreams/perf-baselines/"
    "ui-gallery-code-editor-torture-autoscroll-typical.windows-rtx4090.v2.json"
)

COMPLEX_WHEEL_SCRIPT = (
    "tools/diag-scripts/ui-gallery/code-editor/"
    "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.json"
)
COMPLEX_WHEEL_BASELINE = (
    "docs/workstreams/perf-baselines/"
    "ui-gallery-code-editor-torture-decorations-soft-wrap-inline-preedit-composed-wheel-steady.windows-rtx4090.v1.json"
)

COMMON_ENVS = [
    "FRET_A11Y_DISABLE=1",
    "FRET_UI_GALLERY_BOOTSTRAP_FONTS=1",
    "FRET_UI_GALLERY_VIEW_CACHE=1",
    "FRET_UI_GALLERY_VIEW_CACHE_SHELL=1",
    "FRET_UI_GALLERY_CODE_EDITOR_TORTURE_OVERLAY=0",
    "FRET_DIAG_SCRIPT_AUTO_DUMP=0",
    "FRET_DIAG_SEMANTICS=0",
]


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


def _read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def _default_date_tag() -> str:
    return time.strftime("%Y%m%d-%H%M%S")


def _default_out_dir(date_tag: str) -> str:
    return f"target/fret-diag/editor-paint-contract-validate-{date_tag}"


def _default_fretboard_bin() -> str:
    return "target/release/fretboard-dev.exe"


def _default_launch_bin() -> str:
    return "target/release/fret-ui-gallery.exe"


def _default_launch_cmd() -> list[str]:
    return [
        "cargo",
        "run",
        "-p",
        "fret-ui-gallery",
        "--release",
        "--features",
        "gallery-full",
    ]


def _launch_cmd_from_args(workspace_root: Path, args: argparse.Namespace) -> tuple[list[str], Path | None]:
    launch_cmd_arg = getattr(args, "launch_cmd", None)
    if isinstance(launch_cmd_arg, list):
        launch_cmd = [str(token) for token in launch_cmd_arg if str(token)]
        if not launch_cmd:
            raise ValueError("--launch-cmd requires at least one token")
        return launch_cmd, None
    if isinstance(launch_cmd_arg, str) and launch_cmd_arg.strip():
        try:
            launch_cmd = shlex.split(launch_cmd_arg)
        except ValueError as exc:
            raise ValueError(f"invalid --launch-cmd: {exc}") from exc
        if not launch_cmd:
            raise ValueError("--launch-cmd requires at least one token")
        return launch_cmd, None

    launch_bin_raw = str(getattr(args, "launch_bin", "")).strip()
    if launch_bin_raw:
        launch_bin_path = _resolve_workspace_path(workspace_root, launch_bin_raw)
        return [str(launch_bin_path)], launch_bin_path

    return _default_launch_cmd(), None


def _diag_perf_command(
    *,
    fretboard_bin: str,
    launch_cmd: list[str],
    script: str,
    out_dir: str,
    repeat: int,
    warmup_frames: int,
    baseline: str,
    with_paint_perf: bool,
) -> list[str]:
    cmd = [
        fretboard_bin,
        "diag",
        "perf",
        script,
        "--dir",
        out_dir,
        "--repeat",
        str(repeat),
        "--warmup-frames",
        str(warmup_frames),
        "--reuse-launch",
        "--prewarm-script",
        PREWARM_SCRIPT,
        "--prelude-script",
        PRELUDE_SCRIPT,
        "--sort",
        "time",
        "--top",
        "15",
        "--json",
        "--perf-baseline",
        baseline,
    ]
    envs = [*COMMON_ENVS]
    if with_paint_perf:
        envs.append("FRET_CODE_EDITOR_DIAG_PAINT_PERF=1")
    for env in envs:
        cmd += ["--env", env]
    cmd += ["--launch", "--", *launch_cmd]
    return cmd


def build_plan(
    *,
    python_bin: str,
    fretboard_bin: str,
    launch_cmd: list[str],
    out_dir: str,
    resize_attempts: int,
    resize_repeat: int,
    typical_repeat: int,
    complex_repeat: int,
    warmup_frames: int,
    skip_preflight: bool,
    with_paint_perf: bool,
) -> list[dict[str, Any]]:
    steps: list[dict[str, Any]] = []
    if not skip_preflight:
        steps.append(
            {
                "name": "preflight",
                "out_dir": f"{out_dir}/preflight",
                "wants_stats": False,
                "cmd": [
                    python_bin,
                    "tools/perf/diag_editor_paint_contract_preflight.py",
                    "--out-summary",
                    f"{out_dir}/preflight/summary.json",
                ],
            }
        )

    steps.append(
        {
            "name": "resize-jitter",
            "out_dir": f"{out_dir}/resize-jitter",
            "wants_stats": True,
            "cmd": [
                python_bin,
                "tools/perf/diag_resize_probes_gate.py",
                "--suite",
                RESIZE_SUITE,
                "--out-dir",
                f"{out_dir}/resize-jitter",
                "--baseline",
                RESIZE_BASELINE,
                "--fretboard-bin",
                fretboard_bin,
                "--attempts",
                str(resize_attempts),
                "--repeat",
                str(resize_repeat),
                "--warmup-frames",
                str(warmup_frames),
                "--launch-cmd",
                shlex.join(launch_cmd),
            ],
        }
    )
    steps.append(
        {
            "name": "typical-autoscroll",
            "out_dir": f"{out_dir}/typical-autoscroll",
            "wants_stats": True,
            "cmd": _diag_perf_command(
                fretboard_bin=fretboard_bin,
                launch_cmd=launch_cmd,
                script=TYPICAL_SCRIPT,
                out_dir=f"{out_dir}/typical-autoscroll",
                repeat=typical_repeat,
                warmup_frames=warmup_frames,
                baseline=TYPICAL_BASELINE,
                with_paint_perf=with_paint_perf,
            ),
        }
    )
    steps.append(
        {
            "name": "complex-wheel",
            "out_dir": f"{out_dir}/complex-wheel",
            "wants_stats": True,
            "cmd": _diag_perf_command(
                fretboard_bin=fretboard_bin,
                launch_cmd=launch_cmd,
                script=COMPLEX_WHEEL_SCRIPT,
                out_dir=f"{out_dir}/complex-wheel",
                repeat=complex_repeat,
                warmup_frames=warmup_frames,
                baseline=COMPLEX_WHEEL_BASELINE,
                with_paint_perf=with_paint_perf,
            ),
        }
    )
    return steps


def _run_step(cmd: list[str], cwd: Path, stdout_path: Path, stderr_path: Path) -> int:
    stdout_path.parent.mkdir(parents=True, exist_ok=True)
    stderr_path.parent.mkdir(parents=True, exist_ok=True)
    with stdout_path.open("wb") as out, stderr_path.open("wb") as err:
        p = subprocess.run(cmd, cwd=str(cwd), stdout=out, stderr=err)
        return int(p.returncode)


def _validate_inputs(workspace_root: Path, args: argparse.Namespace) -> list[str]:
    missing: list[str] = []
    for rel in [
        PREWARM_SCRIPT,
        PRELUDE_SCRIPT,
        RESIZE_BASELINE,
        TYPICAL_SCRIPT,
        TYPICAL_BASELINE,
        COMPLEX_WHEEL_SCRIPT,
        COMPLEX_WHEEL_BASELINE,
    ]:
        if not _resolve_workspace_path(workspace_root, rel).is_file():
            missing.append(rel)
    if not bool(args.dry_run):
        for rel in [str(args.fretboard_bin)]:
            if not _resolve_workspace_path(workspace_root, rel).is_file():
                missing.append(rel)
        launch_bin_raw = str(getattr(args, "launch_bin", "")).strip()
        if launch_bin_raw and not _resolve_workspace_path(workspace_root, launch_bin_raw).is_file():
            missing.append(launch_bin_raw)
    return missing


def _bundle_from_regression_summary(path: Path) -> str | None:
    try:
        doc = _read_json(path)
    except Exception:
        return None
    if not isinstance(doc, dict):
        return None
    items = doc.get("items")
    if not isinstance(items, list):
        return None
    for item in items:
        if not isinstance(item, dict):
            continue
        evidence = item.get("evidence")
        if not isinstance(evidence, dict):
            continue
        bundle = evidence.get("bundle_artifact")
        if isinstance(bundle, str) and bundle:
            return bundle
    return None


def artifact_summary_for_step(step_out_dir: Path) -> dict[str, Any]:
    check_path = step_out_dir / "check.perf_thresholds.json"
    regression_path = step_out_dir / "regression.summary.json"

    failures_count: int | None = None
    bundle: str | None = None

    if check_path.is_file():
        try:
            check = _read_json(check_path)
        except Exception:
            check = None
        if isinstance(check, dict):
            failures = check.get("failures")
            if isinstance(failures, list):
                failures_count = len(failures)
            layout_summary = check.get("layout_perf_summary")
            if isinstance(layout_summary, dict):
                bundle_artifact = layout_summary.get("bundle_artifact")
                if isinstance(bundle_artifact, str) and bundle_artifact:
                    bundle = bundle_artifact

    if bundle is None and regression_path.is_file():
        bundle = _bundle_from_regression_summary(regression_path)

    return {
        "step_out_dir": str(step_out_dir),
        "check_perf_thresholds": str(check_path) if check_path.is_file() else None,
        "check_perf_thresholds_failures": failures_count,
        "regression_summary": str(regression_path) if regression_path.is_file() else None,
        "worst_bundle": bundle,
    }


def _has_metric(doc: dict[str, Any], metric: str) -> bool:
    for group in ["p95", "max", "sum"]:
        values = doc.get(group)
        if isinstance(values, dict) and metric in values:
            return True
    return False


def _code_editor_max_metric_is_zero(doc: dict[str, Any], metric: str) -> bool:
    code_editor = doc.get("code_editor_paint_perf")
    if not isinstance(code_editor, dict):
        return False
    max_values = code_editor.get("max")
    if not isinstance(max_values, dict):
        return False
    return max_values.get(metric) == 0


def stats_coverage_for_doc(doc: object) -> dict[str, bool]:
    if not isinstance(doc, dict):
        return {
            "paint_widget": False,
            "renderer_text_encode_upload": False,
            "code_editor_paint_perf": False,
            "code_editor_torture_overlay_zero": False,
        }
    code_editor = doc.get("code_editor_paint_perf")
    return {
        "paint_widget": _has_metric(doc, "paint_widget_time_us"),
        "renderer_text_encode_upload": all(
            _has_metric(doc, metric)
            for metric in [
                "renderer_prepare_text_us",
                "renderer_encode_scene_us",
                "renderer_upload_us",
            ]
        ),
        "code_editor_paint_perf": isinstance(code_editor, dict) and isinstance(code_editor.get("p95"), dict),
        "code_editor_torture_overlay_zero": _code_editor_max_metric_is_zero(doc, "us_torture_overlay"),
    }


def thresholds_pass_for_artifacts(artifacts: dict[str, Any]) -> bool:
    return artifacts.get("check_perf_thresholds_failures") == 0


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Run the Windows RTX4090 editor paint contract validation plan.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("--target-profile", default=TARGET_PROFILE, choices=[TARGET_PROFILE])
    ap.add_argument("--date-tag", default=_default_date_tag())
    ap.add_argument("--out-dir", default="")
    ap.add_argument("--python-bin", default=sys.executable)
    ap.add_argument("--fretboard-bin", default=_default_fretboard_bin())
    ap.add_argument(
        "--launch-bin",
        default="",
        help=(
            "Legacy direct binary launch. Prefer the default cargo launch, or pass --launch-cmd as "
            "the final option, so fret-diag can prove required_launch_features."
        ),
    )
    ap.add_argument(
        "--launch-cmd",
        default="",
        help=(
            "Shell-like launch command forwarded after `diag perf --launch --`. Quote the whole "
            "value when passing spaces. Defaults to `cargo run -p fret-ui-gallery --release "
            "--features gallery-full`."
        ),
    )
    ap.add_argument("--resize-attempts", type=int, default=3)
    ap.add_argument("--resize-repeat", type=int, default=7)
    ap.add_argument("--typical-repeat", type=int, default=15)
    ap.add_argument("--complex-repeat", type=int, default=7)
    ap.add_argument("--warmup-frames", type=int, default=5)
    ap.add_argument("--skip-preflight", action="store_true", default=False)
    ap.add_argument(
        "--with-paint-perf",
        action="store_true",
        default=False,
        help=(
            "Also add FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 to the direct diag perf probes. "
            "The resize helper already enables that env for ui-code-editor-resize-probes. "
            "Use this for attribution evidence, not silent baseline loosening."
        ),
    )
    ap.add_argument("--dry-run", action="store_true", default=False)
    ap.add_argument(
        "--skip-stats",
        action="store_true",
        default=False,
        help="Do not run `diag stats` for the worst bundle collected from each validation probe.",
    )
    ap.add_argument(
        "--allow-non-windows",
        action="store_true",
        default=False,
        help="Allow executing the Windows-profile command plan from a non-Windows host.",
    )
    ap.add_argument(
        "--allow-existing-out-dir",
        action="store_true",
        default=False,
        help=(
            "Allow a non-dry-run validation to write into an existing non-empty output directory. "
            "Use only when intentionally resuming/debugging a run; the default protects closeout "
            "evidence from stale dry-run or failed-run artifacts."
        ),
    )
    ap.add_argument("--keep-going", action="store_true", default=False)
    args = ap.parse_args()

    if int(args.resize_attempts) < 1:
        print("error: --resize-attempts must be >= 1", file=sys.stderr)
        return 2
    if min(int(args.resize_repeat), int(args.typical_repeat), int(args.complex_repeat), int(args.warmup_frames)) < 1:
        print("error: repeat and warmup values must be >= 1", file=sys.stderr)
        return 2

    workspace_root = _workspace_root()
    out_dir = str(args.out_dir).strip() or _default_out_dir(str(args.date_tag))
    out_dir_path = _resolve_workspace_path(workspace_root, out_dir)
    try:
        launch_cmd, launch_bin_path = _launch_cmd_from_args(workspace_root, args)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    if (
        str(args.target_profile) == TARGET_PROFILE
        and not sys.platform.startswith("win")
        and not bool(args.dry_run)
        and not bool(args.allow_non_windows)
    ):
        print(
            "error: windows-rtx4090 validation must run on the target Windows host "
            "(use --dry-run to inspect the plan)",
            file=sys.stderr,
        )
        return 2

    missing = _validate_inputs(workspace_root, args)
    if missing:
        print("error: required validation inputs are missing:", file=sys.stderr)
        for item in missing:
            print(f"  - {item}", file=sys.stderr)
        return 2

    if (
        not bool(args.dry_run)
        and out_dir_path.is_dir()
        and any(out_dir_path.iterdir())
        and not bool(args.allow_existing_out_dir)
    ):
        print(
            "error: validation output directory already exists and is not empty "
            f"({out_dir_path}); choose a fresh --date-tag/--out-dir or pass --allow-existing-out-dir",
            file=sys.stderr,
        )
        return 2

    plan = build_plan(
        python_bin=str(args.python_bin),
        fretboard_bin=str(args.fretboard_bin),
        launch_cmd=launch_cmd,
        out_dir=out_dir,
        resize_attempts=int(args.resize_attempts),
        resize_repeat=int(args.resize_repeat),
        typical_repeat=int(args.typical_repeat),
        complex_repeat=int(args.complex_repeat),
        warmup_frames=int(args.warmup_frames),
        skip_preflight=bool(args.skip_preflight),
        with_paint_perf=bool(args.with_paint_perf),
    )

    step_results: list[dict[str, Any]] = []
    if bool(args.dry_run):
        print(f"[validate] target-profile={args.target_profile} out-dir={out_dir}")
        for step in plan:
            print(f"[validate] {step['name']}: {shlex.join(step['cmd'])}")
        summary = {
            "kind": "editor_paint_contract_validate_plan",
            "dry_run": True,
            "target_profile": str(args.target_profile),
            "date_tag": str(args.date_tag),
            "out_dir": out_dir,
            "launch_bin": str(launch_bin_path) if launch_bin_path is not None else None,
            "launch_cmd": launch_cmd,
            "with_paint_perf": bool(args.with_paint_perf),
            "stats_enabled": not bool(args.skip_stats),
            "steps": plan,
        }
        _write_json(out_dir_path / "validation-plan.json", summary)
        print(f"[validate] wrote plan: {out_dir_path / 'validation-plan.json'}")
        return 0

    out_dir_path.mkdir(parents=True, exist_ok=True)
    print(f"[validate] target-profile={args.target_profile} out-dir={out_dir_path}")
    pass_all = True
    for step in plan:
        name = str(step["name"])
        cmd = list(step["cmd"])
        step_dir = out_dir_path / "runner-logs" / name
        stdout_path = step_dir / "stdout.log"
        stderr_path = step_dir / "stderr.log"
        _write_json(step_dir / "cmd.json", {"cmd": cmd})
        print(f"[validate] running {name}: {shlex.join(cmd)}")
        started = time.time()
        rc = _run_step(cmd, workspace_root, stdout_path, stderr_path)
        elapsed_ms = int((time.time() - started) * 1000.0)
        ok = rc == 0
        pass_all = pass_all and ok
        artifacts = artifact_summary_for_step(_resolve_workspace_path(workspace_root, str(step.get("out_dir", ""))))
        thresholds_ok: bool | None = None
        if bool(step.get("wants_stats")):
            thresholds_ok = thresholds_pass_for_artifacts(artifacts)
            pass_all = pass_all and thresholds_ok
        stats_result: dict[str, Any] | None = None
        if bool(step.get("wants_stats")) and not bool(args.skip_stats):
            bundle = artifacts.get("worst_bundle")
            if isinstance(bundle, str) and bundle:
                stats_cmd = [
                    str(args.fretboard_bin),
                    "diag",
                    "stats",
                    bundle,
                    "--sort",
                    "cpu_cycles",
                    "--top",
                    "15",
                    "--json",
                ]
                stats_stdout = step_dir / "stats.stdout.json"
                stats_stderr = step_dir / "stats.stderr.log"
                _write_json(step_dir / "stats.cmd.json", {"cmd": stats_cmd})
                stats_started = time.time()
                stats_rc = _run_step(stats_cmd, workspace_root, stats_stdout, stats_stderr)
                coverage: dict[str, bool] | None = None
                missing_coverage: list[str] = []
                if stats_rc == 0:
                    try:
                        coverage = stats_coverage_for_doc(_read_json(stats_stdout))
                    except Exception:
                        coverage = stats_coverage_for_doc(None)
                    required_coverage = ["paint_widget", "renderer_text_encode_upload"]
                    if bool(args.with_paint_perf):
                        required_coverage.append("code_editor_paint_perf")
                        required_coverage.append("code_editor_torture_overlay_zero")
                    missing_coverage = [name for name in required_coverage if not bool(coverage.get(name))]
                stats_ok = stats_rc == 0 and not missing_coverage
                pass_all = pass_all and stats_ok
                stats_result = {
                    "ok": stats_ok,
                    "rc": stats_rc,
                    "elapsed_ms": int((time.time() - stats_started) * 1000.0),
                    "cmd": stats_cmd,
                    "stdout": str(stats_stdout),
                    "stderr": str(stats_stderr),
                    "coverage": coverage,
                    "missing_coverage": missing_coverage,
                }
            else:
                pass_all = False
                stats_result = {
                    "ok": False,
                    "rc": None,
                    "error": "worst bundle not found for stats collection",
                }
        step_results.append(
            {
                "name": name,
                "ok": ok,
                "rc": rc,
                "elapsed_ms": elapsed_ms,
                "cmd": cmd,
                "stdout": str(stdout_path),
                "stderr": str(stderr_path),
                "artifacts": artifacts,
                "thresholds_ok": thresholds_ok,
                "stats": stats_result,
            }
        )
        if (
            not ok
            or thresholds_ok is False
            or (stats_result is not None and not bool(stats_result.get("ok")))
        ) and not bool(args.keep_going):
            break

    summary = {
        "kind": "editor_paint_contract_validate_summary",
        "ok": pass_all,
        "target_profile": str(args.target_profile),
        "date_tag": str(args.date_tag),
        "out_dir": str(out_dir_path),
        "launch_bin": str(launch_bin_path) if launch_bin_path is not None else None,
        "launch_cmd": launch_cmd,
        "with_paint_perf": bool(args.with_paint_perf),
        "stats_enabled": not bool(args.skip_stats),
        "steps": step_results,
    }
    summary_path = out_dir_path / "summary.json"
    _write_json(summary_path, summary)

    if not pass_all:
        print(f"[validate] FAIL. Summary: {summary_path}", file=sys.stderr)
        return 1
    print(f"[validate] PASS. Summary: {summary_path}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        raise SystemExit(130)
