#!/usr/bin/env python3
"""Run paired external+internal memory sampling for hello_world_compare_demo."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class Case:
    label: str
    env: dict[str, str]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out-dir", required=True)
    parser.add_argument("--binary", default="target/release/hello_world_compare_demo")
    parser.add_argument(
        "--external-sampler",
        default="tools/sample_external_process_memory.py",
    )
    parser.add_argument(
        "--sample-at-secs",
        default="2,6,12",
        help="Comma-separated offsets passed to external and internal sampling.",
    )
    parser.add_argument("--post-sample-wait-secs", type=float, default=0.0)
    parser.add_argument(
        "--steady-offset-secs",
        type=float,
        default=6.0,
        help="Offset used for the summary table.",
    )
    parser.add_argument(
        "--scene",
        choices=["full", "no_text", "empty"],
        default="full",
    )
    parser.add_argument(
        "--preset",
        choices=["surface", "latency", "msaa", "size"],
        default="surface",
    )
    parser.add_argument(
        "--case",
        action="append",
        default=[],
        help="Custom case: label[:KEY=VALUE,KEY=VALUE...]",
    )
    parser.add_argument(
        "--keep-going",
        action="store_true",
        help="Keep running remaining cases after a failure.",
    )
    return parser.parse_args()


def parse_case(raw: str) -> Case:
    label, _, env_raw = raw.partition(":")
    label = label.strip()
    if not label:
        raise SystemExit(f"invalid case `{raw}`: missing label")
    env: dict[str, str] = {}
    if env_raw.strip():
        for piece in env_raw.split(","):
            piece = piece.strip()
            if not piece:
                continue
            key, sep, value = piece.partition("=")
            if not sep:
                raise SystemExit(f"invalid case env `{piece}` in `{raw}`")
            env[key.strip()] = value.strip()
    return Case(label=label, env=env)


def preset_cases(name: str) -> list[Case]:
    if name == "latency":
        return [
            Case("baseline", {}),
            Case("latency1", {"FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY": "1"}),
            Case("latency2", {"FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY": "2"}),
            Case("latency3", {"FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY": "3"}),
        ]
    if name == "msaa":
        return [
            Case("baseline", {}),
            Case("msaa1", {"FRET_RENDER_WGPU_PATH_MSAA_SAMPLES": "1"}),
            Case("msaa2", {"FRET_RENDER_WGPU_PATH_MSAA_SAMPLES": "2"}),
            Case("msaa4", {"FRET_RENDER_WGPU_PATH_MSAA_SAMPLES": "4"}),
        ]
    if name == "size":
        return [
            Case("baseline", {}),
            Case(
                "size256",
                {
                    "FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH": "256",
                    "FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT": "256",
                },
            ),
            Case(
                "size1000",
                {
                    "FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH": "1000",
                    "FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT": "1000",
                },
            ),
        ]
    return [
        Case("baseline", {}),
        Case("latency1", {"FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY": "1"}),
        Case("latency2", {"FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY": "2"}),
        Case("latency3", {"FRET_RENDER_WGPU_SURFACE_DESIRED_MAX_FRAME_LATENCY": "3"}),
        Case("msaa1", {"FRET_RENDER_WGPU_PATH_MSAA_SAMPLES": "1"}),
        Case(
            "size256",
            {
                "FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH": "256",
                "FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT": "256",
            },
        ),
        Case(
            "size1000",
            {
                "FRET_HELLO_WORLD_COMPARE_WINDOW_WIDTH": "1000",
                "FRET_HELLO_WORLD_COMPARE_WINDOW_HEIGHT": "1000",
            },
        ),
    ]


def scene_env(scene: str) -> dict[str, str]:
    if scene == "no_text":
        return {"FRET_HELLO_WORLD_COMPARE_NO_TEXT": "1"}
    if scene == "empty":
        return {
            "FRET_HELLO_WORLD_COMPARE_NO_TEXT": "1",
            "FRET_HELLO_WORLD_COMPARE_NO_SWATCHES": "1",
        }
    return {}


def sanitize_label(label: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]+", "-", label).strip("-") or "case"


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def find_sample(samples: list[dict[str, Any]], offset_secs: float) -> dict[str, Any] | None:
    if not samples:
        return None
    return min(samples, key=lambda sample: abs(float(sample.get("offset_secs", 0.0)) - offset_secs))


def mib(value: int | None) -> float | None:
    if value is None:
        return None
    return value / (1024.0 * 1024.0)


def max_sample_value(samples: list[dict[str, Any]], key: str) -> int | None:
    values = [int(value) for sample in samples if (value := sample.get(key)) is not None]
    if not values:
        return None
    return max(values)


def summarize_pair(
    label: str,
    env_overrides: dict[str, str],
    external_summary: dict[str, Any],
    internal_summary: dict[str, Any],
    steady_offset_secs: float,
) -> dict[str, Any]:
    external_samples = external_summary.get("samples") or []
    internal_samples = internal_summary.get("samples") or []

    paired_samples: list[dict[str, Any]] = []
    for external_sample in external_samples:
        offset_secs = float(external_sample.get("offset_secs", 0.0))
        key_metrics = external_sample.get("key_metrics") or {}
        internal_sample = find_sample(internal_samples, offset_secs)
        metal_current_allocated_size_bytes = None
        allocator_total_allocated_bytes = None
        if internal_sample is not None:
            allocator = internal_sample.get("allocator") or {}
            metal_current_allocated_size_bytes = allocator.get("metal_current_allocated_size_bytes")
            allocator_total_allocated_bytes = allocator.get("total_allocated_bytes")
        owned_unmapped_bytes = key_metrics.get("owned_unmapped_memory_dirty_bytes")
        io_surface_bytes = key_metrics.get("io_surface_dirty_bytes")
        io_accelerator_bytes = key_metrics.get("io_accelerator_dirty_bytes")
        graphics_total_bytes = sum(
            int(value or 0)
            for value in [owned_unmapped_bytes, io_surface_bytes, io_accelerator_bytes]
        )
        residual_bytes = None
        if metal_current_allocated_size_bytes is not None:
            residual_bytes = graphics_total_bytes - int(metal_current_allocated_size_bytes)
        paired_samples.append(
            {
                "offset_secs": offset_secs,
                "physical_footprint_bytes": key_metrics.get("physical_footprint_bytes"),
                "physical_footprint_peak_bytes": key_metrics.get("physical_footprint_peak_bytes"),
                "owned_unmapped_memory_dirty_bytes": owned_unmapped_bytes,
                "io_surface_dirty_bytes": io_surface_bytes,
                "io_accelerator_dirty_bytes": io_accelerator_bytes,
                "graphics_total_bytes": graphics_total_bytes,
                "metal_current_allocated_size_bytes": metal_current_allocated_size_bytes,
                "allocator_total_allocated_bytes": allocator_total_allocated_bytes,
                "residual_bytes": residual_bytes,
            }
        )

    steady = find_sample(paired_samples, steady_offset_secs)
    requested_runtime = internal_summary.get("requested_runtime") or {}
    physical_footprint_peak_bytes_max = max_sample_value(
        paired_samples, "physical_footprint_peak_bytes"
    )
    physical_footprint_sample_max_bytes = max_sample_value(
        paired_samples, "physical_footprint_bytes"
    )
    graphics_total_sample_max_bytes = max_sample_value(
        paired_samples, "graphics_total_bytes"
    )
    metal_current_allocated_size_sample_max_bytes = max_sample_value(
        paired_samples, "metal_current_allocated_size_bytes"
    )
    startup_collapse_bytes = None
    if physical_footprint_peak_bytes_max is not None and steady is not None:
        steady_physical = steady.get("physical_footprint_bytes")
        if steady_physical is not None:
            startup_collapse_bytes = (
                int(physical_footprint_peak_bytes_max) - int(steady_physical)
            )
    return {
        "label": label,
        "env": env_overrides,
        "requested_runtime": requested_runtime,
        "artifacts": {
            "external_summary": external_summary,
            "internal_summary": internal_summary,
        },
        "paired_samples": paired_samples,
        "steady_offset_secs": steady_offset_secs,
        "steady": steady,
        "physical_footprint_peak_bytes_max": physical_footprint_peak_bytes_max,
        "physical_footprint_sample_max_bytes": physical_footprint_sample_max_bytes,
        "graphics_total_sample_max_bytes": graphics_total_sample_max_bytes,
        "metal_current_allocated_size_sample_max_bytes": metal_current_allocated_size_sample_max_bytes,
        "startup_collapse_bytes": startup_collapse_bytes,
    }


def markdown_table(rows: list[dict[str, Any]]) -> str:
    header = (
        "| Case | Scene | Steady | Peak MiB | Drop MiB | Physical MiB | Graphics MiB | Metal MiB | Residual MiB | "
        "Owned MiB | IOSurface MiB | IOAccel MiB | Notes |"
    )
    sep = "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |"
    lines = [header, sep]
    for row in rows:
        steady = row.get("steady") or {}
        requested = row.get("requested_runtime") or {}
        scene = requested.get("scene") or {}
        renderer = requested.get("renderer") or {}
        window = requested.get("window") or {}
        surface = requested.get("surface") or {}
        scene_name = "full"
        if scene.get("no_text") and scene.get("no_swatches"):
            scene_name = "empty"
        elif scene.get("no_text"):
            scene_name = "no_text"
        notes = []
        if surface.get("desired_maximum_frame_latency") is not None:
            notes.append(f"lat={surface['desired_maximum_frame_latency']}")
        if renderer.get("path_msaa_samples_effective") is not None:
            notes.append(f"msaa={renderer['path_msaa_samples_effective']}")
        if window.get("width_px") or window.get("height_px"):
            notes.append(f"{window.get('width_px', '?')}x{window.get('height_px', '?')}")
        lines.append(
            "| {label} | {scene_name} | {offset:.1f}s | {peak} | {drop} | {physical} | {graphics} | {metal} | {residual} | {owned} | {iosurface} | {ioaccel} | {notes} |".format(
                label=row["label"],
                scene_name=scene_name,
                offset=float((steady or {}).get("offset_secs", row.get("steady_offset_secs", 0.0))),
                peak=format_mib(row.get("physical_footprint_peak_bytes_max")),
                drop=format_mib(row.get("startup_collapse_bytes")),
                physical=format_mib((steady or {}).get("physical_footprint_bytes")),
                graphics=format_mib((steady or {}).get("graphics_total_bytes")),
                metal=format_mib((steady or {}).get("metal_current_allocated_size_bytes")),
                residual=format_mib((steady or {}).get("residual_bytes")),
                owned=format_mib((steady or {}).get("owned_unmapped_memory_dirty_bytes")),
                iosurface=format_mib((steady or {}).get("io_surface_dirty_bytes")),
                ioaccel=format_mib((steady or {}).get("io_accelerator_dirty_bytes")),
                notes=", ".join(notes),
            )
        )
    return "\n".join(lines)


def format_mib(value: int | None) -> str:
    value_mib = mib(value)
    if value_mib is None:
        return "n/a"
    return f"{value_mib:.1f}"


def run_case(
    out_dir: Path,
    case: Case,
    args: argparse.Namespace,
    base_env: dict[str, str],
) -> dict[str, Any]:
    case_dir = out_dir / "cases" / sanitize_label(case.label)
    case_dir.mkdir(parents=True, exist_ok=True)
    internal_report_path = case_dir / "internal.gpu.json"
    env = os.environ.copy()
    env.update(base_env)
    env.update(case.env)
    env["FRET_HELLO_WORLD_COMPARE_INTERNAL_REPORT_PATH"] = str(internal_report_path)
    env["FRET_HELLO_WORLD_COMPARE_INTERNAL_SAMPLE_AT_SECS"] = args.sample_at_secs

    command = [
        sys.executable,
        args.external_sampler,
        "--out-dir",
        str(case_dir),
        "--label",
        case.label,
        "--sample-at-secs",
        args.sample_at_secs,
        "--post-sample-wait-secs",
        str(args.post_sample_wait_secs),
        "--",
        args.binary,
    ]
    subprocess.run(command, check=True, env=env)

    external_summary_path = case_dir / "summary.json"
    if not external_summary_path.is_file():
        raise FileNotFoundError(f"missing external summary: {external_summary_path}")
    if not internal_report_path.is_file():
        raise FileNotFoundError(f"missing internal report: {internal_report_path}")

    external_summary = load_json(external_summary_path)
    internal_summary = load_json(internal_report_path)
    paired_summary = summarize_pair(
        label=case.label,
        env_overrides={**base_env, **case.env},
        external_summary=external_summary,
        internal_summary=internal_summary,
        steady_offset_secs=args.steady_offset_secs,
    )
    paired_summary["artifacts"] = {
        "external_summary": str(external_summary_path),
        "internal_summary": str(internal_report_path),
        "stdout": str(case_dir / "stdout.log"),
        "stderr": str(case_dir / "stderr.log"),
    }
    return paired_summary


def main() -> int:
    args = parse_args()
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    if args.case:
        cases = [parse_case(raw) for raw in args.case]
    else:
        cases = preset_cases(args.preset)

    base_env = scene_env(args.scene)
    results: list[dict[str, Any]] = []
    failures: list[dict[str, str]] = []

    for case in cases:
        print(f"==> Running case `{case.label}`", flush=True)
        try:
            results.append(run_case(out_dir, case, args, base_env))
        except Exception as exc:  # noqa: BLE001
            failures.append({"label": case.label, "error": repr(exc)})
            print(f"Case `{case.label}` failed: {exc}", file=sys.stderr, flush=True)
            if not args.keep_going:
                break

    summary = {
        "schema_version": 1,
        "binary": args.binary,
        "external_sampler": args.external_sampler,
        "sample_at_secs": args.sample_at_secs,
        "steady_offset_secs": args.steady_offset_secs,
        "scene": args.scene,
        "preset": args.preset,
        "results": results,
        "failures": failures,
    }
    summary_json_path = out_dir / "summary.json"
    summary_md_path = out_dir / "summary.md"
    summary_json_path.write_text(json.dumps(summary, indent=2))
    summary_md_path.write_text(markdown_table(results) + "\n")

    print(markdown_table(results))
    if failures:
        print(json.dumps({"failures": failures}, indent=2), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
