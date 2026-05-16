#!/usr/bin/env python3
"""
Verify synced target-machine editor paint contract validation artifacts.

This helper does not run perf probes. It checks the summary produced by
diag_editor_paint_contract_validate.py after the Windows RTX4090 output
directories have been copied back into the workspace.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

import diag_editor_paint_contract_validate as validate


EXPECTED_STATS_STEPS = {
    "resize-jitter": {"repeat": 7, "warmup_frames": 5, "attempts": 3},
    "typical-autoscroll": {"repeat": 15, "warmup_frames": 5},
    "complex-wheel": {"repeat": 7, "warmup_frames": 5},
}


def _read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def _write_json(path: Path, v: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(v, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def _as_path(value: object) -> Path | None:
    if not isinstance(value, str) or not value:
        return None
    return Path(value)


def _artifact_path(value: object, fallback: Path) -> Path:
    path = _as_path(value)
    if path is not None and path.is_file():
        return path
    if fallback.is_file():
        return fallback
    return path if path is not None else fallback


def _cmd_value(cmd: object, flag: str) -> str | None:
    if not isinstance(cmd, list):
        return None
    for i, item in enumerate(cmd):
        if item == flag and i + 1 < len(cmd):
            value = cmd[i + 1]
            if isinstance(value, str):
                return value
    return None


def _cmd_contains(cmd: object, needle: str) -> bool:
    return isinstance(cmd, list) and any(item == needle for item in cmd)


def _cmd_has_flag_value(cmd: object, flag: str, expected: str) -> bool:
    return _cmd_value(cmd, flag) == expected


def _cmd_env_values(cmd: object) -> set[str]:
    if not isinstance(cmd, list):
        return set()
    envs: set[str] = set()
    for i, item in enumerate(cmd):
        if item == "--env" and i + 1 < len(cmd) and isinstance(cmd[i + 1], str):
            envs.add(str(cmd[i + 1]))
    return envs


def _check_direct_diag_perf_cmd(
    *,
    cmd: object,
    errors: list[str],
    prefix: str,
    expect_with_paint_perf: bool,
) -> None:
    if not _cmd_contains(cmd, "--reuse-launch"):
        errors.append(f"{prefix}: direct diag perf command must use --reuse-launch")
    if not _cmd_has_flag_value(cmd, "--prewarm-script", validate.PREWARM_SCRIPT):
        errors.append(f"{prefix}: direct diag perf command must use the standard prewarm script")
    if not _cmd_has_flag_value(cmd, "--prelude-script", validate.PRELUDE_SCRIPT):
        errors.append(f"{prefix}: direct diag perf command must use the standard prelude script")
    if not _cmd_contains(cmd, "--json"):
        errors.append(f"{prefix}: direct diag perf command must emit --json")
    if not _cmd_contains(cmd, "--launch"):
        errors.append(f"{prefix}: direct diag perf command must launch the target binary")
    if not _cmd_contains(cmd, validate._default_launch_bin()):
        errors.append(f"{prefix}: direct diag perf command must launch {validate._default_launch_bin()}")

    envs = _cmd_env_values(cmd)
    required_envs = set(validate.COMMON_ENVS)
    missing_envs = sorted(required_envs - envs)
    if missing_envs:
        errors.append(f"{prefix}: direct diag perf command missing required envs {missing_envs}")

    paint_perf_env = "FRET_CODE_EDITOR_DIAG_PAINT_PERF=1"
    has_paint_perf = paint_perf_env in envs
    if expect_with_paint_perf and not has_paint_perf:
        errors.append(f"{prefix}: attribution direct diag perf command must set {paint_perf_env}")
    if not expect_with_paint_perf and has_paint_perf:
        errors.append(f"{prefix}: baseline-validation direct diag perf command must not set {paint_perf_env}")


def _check_threshold_file(path: Path, errors: list[str], prefix: str) -> None:
    if not path.is_file():
        errors.append(f"{prefix}: missing threshold report: {path}")
        return
    try:
        doc = _read_json(path)
    except Exception as exc:
        errors.append(f"{prefix}: cannot read threshold report {path}: {exc}")
        return
    if not isinstance(doc, dict):
        errors.append(f"{prefix}: threshold report is not an object: {path}")
        return
    failures = doc.get("failures")
    if failures != []:
        errors.append(f"{prefix}: threshold failures must be [] in {path}")


def _metric_group(doc: dict[str, Any], group: str) -> dict[str, Any]:
    values = doc.get(group)
    return values if isinstance(values, dict) else {}


def _nested_metric(doc: dict[str, Any], *keys: str) -> Any:
    value: Any = doc
    for key in keys:
        if not isinstance(value, dict):
            return None
        value = value.get(key)
    return value


def _decision_inputs_for_doc(doc: object) -> dict[str, Any]:
    if not isinstance(doc, dict):
        return {}
    p95 = _metric_group(doc, "p95")
    max_values = _metric_group(doc, "max")
    code_editor = doc.get("code_editor_paint_perf")
    if not isinstance(code_editor, dict):
        code_editor = {}
    paint_hotspots = doc.get("paint_widget_hotspot_summary")
    if not isinstance(paint_hotspots, dict):
        paint_hotspots = {}

    return {
        "paint_widget_p95_us": p95.get("paint_widget_time_us"),
        "paint_widget_max_us": max_values.get("paint_widget_time_us"),
        "renderer_prepare_text_p95_us": p95.get("renderer_prepare_text_us"),
        "renderer_prepare_text_max_us": max_values.get("renderer_prepare_text_us"),
        "renderer_encode_scene_p95_us": p95.get("renderer_encode_scene_us"),
        "renderer_upload_p95_us": p95.get("renderer_upload_us"),
        "code_editor_paint_perf_frames": code_editor.get("frames"),
        "code_editor_total_p95_us": _nested_metric(code_editor, "p95", "us_total"),
        "code_editor_windowed_surface_callback_p95_us": _nested_metric(
            code_editor,
            "p95",
            "us_windowed_surface_paint_callback",
        ),
        "code_editor_windowed_surface_row_paint_p95_us": _nested_metric(
            code_editor,
            "p95",
            "us_windowed_surface_row_paint",
        ),
        "code_editor_torture_overlay_max_us": _nested_metric(
            code_editor,
            "max",
            "us_torture_overlay",
        ),
        "paint_widget_hotspot_summary": {
            "frames_with_hotspots": paint_hotspots.get("frames_with_hotspots"),
            "canvas_exclusive_p95_us": _nested_metric(
                paint_hotspots,
                "canvas",
                "exclusive_us",
                "p95",
            ),
            "non_canvas_exclusive_p95_us": _nested_metric(
                paint_hotspots,
                "non_canvas",
                "exclusive_us",
                "p95",
            ),
            "gap_to_code_editor_p95": paint_hotspots.get("gap_to_code_editor_p95"),
            "code_editor_windowed_surface_p95": paint_hotspots.get(
                "code_editor_windowed_surface_p95"
            ),
        },
    }


def _stats_coverage_from_step(
    step: dict[str, Any],
    errors: list[str],
    prefix: str,
    fallback_stats_path: Path,
) -> tuple[dict[str, bool], dict[str, Any]]:
    stats = step.get("stats")
    if not isinstance(stats, dict):
        errors.append(f"{prefix}: missing stats result")
        return validate.stats_coverage_for_doc(None), {}
    if stats.get("ok") is not True:
        errors.append(f"{prefix}: stats result is not ok")
    missing_coverage = stats.get("missing_coverage")
    if isinstance(missing_coverage, list) and missing_coverage:
        errors.append(f"{prefix}: stats summary reports missing coverage: {missing_coverage}")

    stdout_path = _artifact_path(stats.get("stdout"), fallback_stats_path)
    if not stdout_path.is_file():
        errors.append(f"{prefix}: stats stdout JSON missing: {stdout_path}")
        return validate.stats_coverage_for_doc(None), {}
    try:
        doc = _read_json(stdout_path)
        return validate.stats_coverage_for_doc(doc), _decision_inputs_for_doc(doc)
    except Exception as exc:
        errors.append(f"{prefix}: cannot read stats stdout JSON {stdout_path}: {exc}")
        return validate.stats_coverage_for_doc(None), {}


def verify_summary_dir(path: Path, *, expect_with_paint_perf: bool) -> dict[str, Any]:
    summary_path = path / "summary.json"
    errors: list[str] = []
    step_reports: dict[str, Any] = {}

    if not summary_path.is_file():
        return {
            "ok": False,
            "summary": str(summary_path),
            "expect_with_paint_perf": expect_with_paint_perf,
            "errors": [f"missing summary: {summary_path}"],
            "steps": step_reports,
        }

    try:
        summary = _read_json(summary_path)
    except Exception as exc:
        return {
            "ok": False,
            "summary": str(summary_path),
            "expect_with_paint_perf": expect_with_paint_perf,
            "errors": [f"cannot read summary {summary_path}: {exc}"],
            "steps": step_reports,
        }

    if not isinstance(summary, dict):
        return {
            "ok": False,
            "summary": str(summary_path),
            "expect_with_paint_perf": expect_with_paint_perf,
            "errors": [f"summary is not an object: {summary_path}"],
            "steps": step_reports,
        }

    if summary.get("kind") != "editor_paint_contract_validate_summary":
        errors.append("summary kind must be editor_paint_contract_validate_summary")
    if summary.get("ok") is not True:
        errors.append("validation summary ok must be true")
    if summary.get("target_profile") != validate.TARGET_PROFILE:
        errors.append(f"target_profile must be {validate.TARGET_PROFILE}")
    if not isinstance(summary.get("date_tag"), str) or not str(summary.get("date_tag")).strip():
        errors.append("summary date_tag must be a non-empty string")
    if summary.get("with_paint_perf") is not expect_with_paint_perf:
        errors.append(f"with_paint_perf must be {expect_with_paint_perf}")
    if summary.get("stats_enabled") is not True:
        errors.append("stats_enabled must be true")

    steps_obj = summary.get("steps")
    steps: dict[str, dict[str, Any]] = {}
    if isinstance(steps_obj, list):
        for step in steps_obj:
            if isinstance(step, dict) and isinstance(step.get("name"), str):
                steps[str(step["name"])] = step
    else:
        errors.append("summary steps must be a list")

    for name, expected in EXPECTED_STATS_STEPS.items():
        prefix = f"{summary_path}:{name}"
        step = steps.get(name)
        if step is None:
            errors.append(f"{prefix}: missing step")
            continue

        cmd = step.get("cmd")
        if step.get("ok") is not True:
            errors.append(f"{prefix}: step ok must be true")
        if step.get("thresholds_ok") is not True:
            errors.append(f"{prefix}: thresholds_ok must be true")
        if str(expected["repeat"]) != _cmd_value(cmd, "--repeat"):
            errors.append(f"{prefix}: --repeat must be {expected['repeat']}")
        if str(expected["warmup_frames"]) != _cmd_value(cmd, "--warmup-frames"):
            errors.append(f"{prefix}: --warmup-frames must be {expected['warmup_frames']}")
        if "attempts" in expected and str(expected["attempts"]) != _cmd_value(cmd, "--attempts"):
            errors.append(f"{prefix}: --attempts must be {expected['attempts']}")

        if name == "resize-jitter":
            if not _cmd_has_flag_value(cmd, "--suite", validate.RESIZE_SUITE):
                errors.append(f"{prefix}: resize suite must be {validate.RESIZE_SUITE}")
            if not _cmd_contains(cmd, validate.RESIZE_BASELINE):
                errors.append(f"{prefix}: resize baseline missing from command")
            if not _cmd_has_flag_value(cmd, "--fretboard-bin", validate._default_fretboard_bin()):
                errors.append(f"{prefix}: resize command must use release fretboard-dev.exe")
            if not _cmd_has_flag_value(cmd, "--launch-bin", validate._default_launch_bin()):
                errors.append(f"{prefix}: resize command must use release fret-ui-gallery.exe")
        elif name == "typical-autoscroll":
            if not _cmd_contains(cmd, validate.TYPICAL_BASELINE):
                errors.append(f"{prefix}: typical baseline missing from command")
            _check_direct_diag_perf_cmd(
                cmd=cmd,
                errors=errors,
                prefix=prefix,
                expect_with_paint_perf=expect_with_paint_perf,
            )
        elif name == "complex-wheel":
            if not _cmd_contains(cmd, validate.COMPLEX_WHEEL_BASELINE):
                errors.append(f"{prefix}: complex wheel baseline missing from command")
            _check_direct_diag_perf_cmd(
                cmd=cmd,
                errors=errors,
                prefix=prefix,
                expect_with_paint_perf=expect_with_paint_perf,
            )

        artifacts = step.get("artifacts")
        if not isinstance(artifacts, dict):
            errors.append(f"{prefix}: missing artifacts")
            continue
        if artifacts.get("check_perf_thresholds_failures") != 0:
            errors.append(f"{prefix}: check_perf_thresholds_failures must be 0")
        check_path = _artifact_path(
            artifacts.get("check_perf_thresholds"),
            summary_path.parent / name / "check.perf_thresholds.json",
        )
        _check_threshold_file(check_path, errors, prefix)

        coverage, decision_inputs = _stats_coverage_from_step(
            step,
            errors,
            prefix,
            summary_path.parent / "runner-logs" / name / "stats.stdout.json",
        )
        required = ["paint_widget", "renderer_text_encode_upload"]
        if expect_with_paint_perf:
            required.append("code_editor_paint_perf")
            required.append("code_editor_torture_overlay_zero")
        missing = [key for key in required if not bool(coverage.get(key))]
        if missing:
            errors.append(f"{prefix}: missing stats coverage {missing}")

        step_reports[name] = {
            "thresholds_ok": step.get("thresholds_ok"),
            "check_perf_thresholds_failures": artifacts.get("check_perf_thresholds_failures"),
            "stats_coverage": coverage,
            "decision_inputs": decision_inputs,
        }

    return {
        "ok": not errors,
        "summary": str(summary_path),
        "date_tag": summary.get("date_tag") if isinstance(summary.get("date_tag"), str) else None,
        "expect_with_paint_perf": expect_with_paint_perf,
        "errors": errors,
        "steps": step_reports,
    }


def verify_artifact_dirs(validation_dir: Path, attribution_dir: Path | None) -> dict[str, Any]:
    validation = verify_summary_dir(validation_dir, expect_with_paint_perf=False)
    attribution = None
    if attribution_dir is not None:
        attribution = verify_summary_dir(attribution_dir, expect_with_paint_perf=True)
    ok = bool(validation.get("ok")) and (attribution is None or bool(attribution.get("ok")))
    return {
        "kind": "editor_paint_contract_artifacts_verify_summary",
        "ok": ok,
        "validation": validation,
        "attribution": attribution,
    }


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Verify synced editor paint contract validation artifacts without rerunning perf probes.",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    ap.add_argument("validation_dir", help="Directory produced by diag_editor_paint_contract_validate.py.")
    ap.add_argument(
        "--attribution-dir",
        default="",
        help="Optional second directory produced with --with-paint-perf.",
    )
    ap.add_argument(
        "--out-report",
        default="",
        help="Path for a JSON verification report. Defaults to <validation_dir>/artifact-verification.summary.json.",
    )
    args = ap.parse_args()

    validation_dir = Path(str(args.validation_dir))
    attribution_dir = Path(str(args.attribution_dir)) if str(args.attribution_dir).strip() else None
    report = verify_artifact_dirs(validation_dir, attribution_dir)

    out_report = Path(str(args.out_report)) if str(args.out_report).strip() else validation_dir / "artifact-verification.summary.json"
    _write_json(out_report, report)

    if not report["ok"]:
        print(f"FAIL: editor paint contract artifacts verify. Report: {out_report}")
        return 1
    print(f"PASS: editor paint contract artifacts verify. Report: {out_report}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
