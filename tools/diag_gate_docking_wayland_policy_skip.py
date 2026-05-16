#!/usr/bin/env python3
"""Gate the local non-Wayland policy-skip path for the Wayland docking campaign."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


GATE_NAME = "diag-gate-docking-wayland-policy-skip"
CAMPAIGN_ID = "imui-p3-wayland-real-host"
OUT_ROOT = Path("target/fret-diag/docking-multiwindow-imgui-parity/wayland-policy-skip-local")


@dataclass(frozen=True)
class ProbeCase:
    name: str
    platform: str
    ui: dict[str, Any]
    reason_code: str


PROBE_CASES = [
    ProbeCase(
        name="windows-platform-mismatch",
        platform="windows",
        ui={
            "multi_window": True,
            "window_tear_off": True,
            "window_hover_detection": "platform_win32",
            "window_set_outer_position": "best_effort",
            "window_z_level": "reliable",
        },
        reason_code="environment.platform_capabilities.platform_ne",
    ),
    ProbeCase(
        name="linux-wayland-multi-window-mismatch",
        platform="linux",
        ui={
            "multi_window": False,
            "window_tear_off": False,
            "window_hover_detection": "none",
            "window_set_outer_position": "none",
            "window_z_level": "none",
        },
        reason_code="environment.platform_capabilities.ui_multi_window_ne",
    ),
    ProbeCase(
        name="linux-x11-tear-off-mismatch",
        platform="linux",
        ui={
            "multi_window": True,
            "window_tear_off": True,
            "window_hover_detection": "best_effort",
            "window_set_outer_position": "best_effort",
            "window_z_level": "best_effort",
        },
        reason_code="environment.platform_capabilities.ui_window_tear_off_ne",
    ),
    ProbeCase(
        name="linux-wayland-hover-detection-mismatch",
        platform="linux",
        ui={
            "multi_window": True,
            "window_tear_off": False,
            "window_hover_detection": "best_effort",
            "window_set_outer_position": "none",
            "window_z_level": "none",
        },
        reason_code="environment.platform_capabilities.ui_window_hover_detection_ne",
    ),
    ProbeCase(
        name="linux-wayland-z-level-mismatch",
        platform="linux",
        ui={
            "multi_window": True,
            "window_tear_off": False,
            "window_hover_detection": "none",
            "window_set_outer_position": "none",
            "window_z_level": "best_effort",
        },
        reason_code="environment.platform_capabilities.ui_window_z_level_ne",
    ),
]


def _repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _exe_name(stem: str) -> str:
    return f"{stem}.exe" if os.name == "nt" else stem


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise SystemExit(f"[{GATE_NAME}] failed to read {path}: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"[{GATE_NAME}] failed to parse {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise SystemExit(f"[{GATE_NAME}] expected JSON object in {path}")
    return value


def _prepare_probe_dir(repo_root: Path, probe: ProbeCase) -> Path:
    out_dir = repo_root / OUT_ROOT / f"{int(time.time() * 1000)}-{probe.name}"
    out_dir.mkdir(parents=True, exist_ok=True)

    _write_json(
        out_dir / "capabilities.json",
        {
            "schema_version": 1,
            "capabilities": ["diag.script_v2"],
            "runner_kind": GATE_NAME,
            "runner_version": "1",
        },
    )
    _write_json(
        out_dir / "environment.sources.json",
        {
            "schema_version": 1,
            "sources": [
                {
                    "source_id": "platform.capabilities",
                    "availability": "launch_time",
                }
            ],
            "runner_kind": GATE_NAME,
            "runner_version": "1",
        },
    )
    _write_json(
        out_dir / "environment.source.platform.capabilities.json",
        {
            "schema_version": 1,
            "source_id": "platform.capabilities",
            "platform": probe.platform,
            "ui": probe.ui,
        },
    )
    return out_dir


def _campaign_argv(repo_root: Path, out_dir: Path, *, reuse_built: bool) -> list[str]:
    if reuse_built:
        exe = repo_root / "target" / "debug" / _exe_name("fretboard-dev")
        if not exe.exists():
            raise SystemExit(
                f"[{GATE_NAME}] --reuse-built requested but {exe} does not exist"
            )
        return [
            str(exe),
            "diag",
            "campaign",
            "run",
            CAMPAIGN_ID,
            "--dir",
            str(out_dir),
            "--json",
        ]
    return [
        "cargo",
        "run",
        "-p",
        "fretboard-dev",
        "--",
        "diag",
        "campaign",
        "run",
        CAMPAIGN_ID,
        "--dir",
        str(out_dir),
        "--json",
    ]


def _extract_json_object(stdout: str) -> dict[str, Any]:
    decoder = json.JSONDecoder()
    for index, char in enumerate(stdout):
        if char != "{":
            continue
        try:
            value, _end = decoder.raw_decode(stdout[index:])
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict):
            return value
    raise SystemExit(f"[{GATE_NAME}] campaign stdout did not contain a JSON object")


def _expect(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"[{GATE_NAME}] {message}")


def _path_from_json(value: Any, *, field: str) -> Path:
    _expect(isinstance(value, str) and bool(value), f"missing path field {field}")
    return Path(value)


def _validate_check_environment(path: Path, probe: ProbeCase) -> None:
    check = _read_json(path)
    _expect(check.get("schema_version") == 1, "check.environment.json schema_version drifted")
    _expect(check.get("status") == "failed", "check.environment.json must be failed")
    _expect(
        check.get("acquisition") == "existing_filesystem",
        "policy-skip probe must use existing_filesystem, not launch_time_probe",
    )
    results = check.get("results")
    _expect(isinstance(results, list) and len(results) == 1, "expected one environment result")
    result = results[0]
    _expect(isinstance(result, dict), "environment result must be an object")
    _expect(result.get("satisfied") is False, "environment result must be unsatisfied")
    _expect(
        result.get("reason_code") == probe.reason_code,
        f"expected {probe.name} reason_code {probe.reason_code}",
    )
    observed = result.get("observed")
    _expect(isinstance(observed, dict), "environment result missing observed payload")
    _expect(observed.get("platform") == probe.platform, f"{probe.name} platform drifted")
    ui = observed.get("ui")
    _expect(isinstance(ui, dict), "environment result missing observed ui payload")
    for key, expected in probe.ui.items():
        _expect(ui.get(key) == expected, f"{probe.name} ui.{key} drifted")


def _validate_no_script_files(run_dir: Path) -> None:
    for subdir_name in ("script-results", "suite-results"):
        subdir = run_dir / subdir_name
        if not subdir.exists():
            continue
        files = [path for path in subdir.rglob("*") if path.is_file()]
        _expect(not files, f"admission skip should not execute item files under {subdir}")


def _validate_report(report: dict[str, Any], probe: ProbeCase) -> Path:
    counters = report.get("counters")
    _expect(isinstance(counters, dict), "missing campaign counters")
    _expect(counters.get("campaigns_total") == 1, "expected one selected campaign")
    _expect(counters.get("campaigns_failed") == 1, "policy skip stays non-passing")
    _expect(counters.get("campaigns_passed") == 0, "policy skip must not pass")
    _expect(counters.get("campaigns_skipped_policy") == 1, "expected one policy skip")
    _expect(counters.get("items_failed") == 0, "policy skip must not fail campaign items")
    _expect(counters.get("items_total") == 1, "expected one campaign item")
    _expect(counters.get("scripts_total") == 1, "expected one script item in the campaign")
    _expect(counters.get("suites_total") == 0, "Wayland campaign should not contain suites")

    runs = report.get("runs")
    _expect(isinstance(runs, list) and len(runs) == 1, "expected one campaign run")
    run = runs[0]
    _expect(isinstance(run, dict), "campaign run must be an object")
    _expect(run.get("campaign_id") == CAMPAIGN_ID, "unexpected campaign id")
    _expect(run.get("status") == "skipped_policy", "campaign must be skipped_policy")
    _expect(run.get("ok") is False, "skipped_policy run must not be ok")
    _expect(run.get("skipped_policy") is True, "skipped_policy bool missing")
    _expect(
        run.get("reason_code") == "environment.requirement_unsatisfied",
        "expected environment requirement reason_code",
    )
    _expect(run.get("items_failed") == 0, "policy skip must not fail items")
    _expect(run.get("items_total") == 1, "expected one item in run")
    _expect(run.get("scripts_total") == 1, "expected one script in run")
    _expect(run.get("capabilities_check_path") is None, "capability preflight should pass")

    environment_check_path = _path_from_json(
        run.get("environment_check_path"), field="environment_check_path"
    )
    _expect(environment_check_path.is_file(), f"missing {environment_check_path}")
    _validate_check_environment(environment_check_path, probe)

    run_dir = _path_from_json(run.get("out_dir"), field="out_dir")
    _expect(run_dir.is_dir(), f"missing campaign out_dir {run_dir}")
    _validate_no_script_files(run_dir)

    campaign_result = _read_json(run_dir / "campaign.result.json")
    aggregate = campaign_result.get("aggregate")
    _expect(isinstance(aggregate, dict), "campaign.result.json missing aggregate")
    _expect(
        aggregate.get("environment_check_path") == str(environment_check_path),
        "campaign.result.json must preserve environment_check_path",
    )
    _expect(
        aggregate.get("capabilities_check_path") is None,
        "campaign.result.json must not report capability skip",
    )

    return environment_check_path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--reuse-built",
        action="store_true",
        help="Run target/debug/fretboard-dev instead of cargo run.",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    for probe in PROBE_CASES:
        out_dir = _prepare_probe_dir(repo_root, probe)
        proc = subprocess.run(
            _campaign_argv(repo_root, out_dir, reuse_built=args.reuse_built),
            cwd=str(repo_root),
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )

        if proc.returncode == 0:
            sys.stdout.write(proc.stdout)
            sys.stderr.write(proc.stderr)
            raise SystemExit(
                f"[{GATE_NAME}] expected non-zero campaign command exit code for policy skip, got 0"
            )

        report = _extract_json_object(proc.stdout)
        check_path = _validate_report(report, probe)
        print(f"[{GATE_NAME}] {probe.name} ok")
        print(f"[{GATE_NAME}] {probe.name} policy_skip_check={check_path}")
    print(f"[{GATE_NAME}] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
