#!/usr/bin/env python3
"""Summarize same-backend external+internal memory samples for Fret compare vs pure wgpu control."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


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


def delta_or_none(lhs: int | None, rhs: int | None) -> int | None:
    if lhs is None or rhs is None:
        return None
    return int(lhs) - int(rhs)


def format_mib(value: int | None) -> str:
    value_mib = mib(value)
    if value_mib is None:
        return "n/a"
    return f"{value_mib:.1f}"


def summarize_case(
    label: str,
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
        allocator = (internal_sample or {}).get("allocator") or {}
        runtime = (internal_sample or {}).get("runtime") or {}
        runner_present = runtime.get("runner_present") or {}
        present_count = runtime.get("present_count")
        if present_count is None:
            present_count = runner_present.get("total_present_count")
        owned_unmapped_bytes = key_metrics.get("owned_unmapped_memory_dirty_bytes")
        io_surface_bytes = key_metrics.get("io_surface_dirty_bytes")
        io_accelerator_bytes = key_metrics.get("io_accelerator_dirty_bytes")
        graphics_total_bytes = sum(
            int(value or 0)
            for value in [owned_unmapped_bytes, io_surface_bytes, io_accelerator_bytes]
        )
        metal_current_allocated_size_bytes = allocator.get("metal_current_allocated_size_bytes")
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
                "residual_bytes": residual_bytes,
                "redraw_count": runtime.get("redraw_count") or runtime.get("render_count"),
                "present_count": present_count,
            }
        )

    steady = find_sample(paired_samples, steady_offset_secs)
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
        "steady_offset_secs": steady_offset_secs,
        "steady": steady,
        "paired_samples": paired_samples,
        "external_summary": external_summary,
        "internal_summary": internal_summary,
        "physical_footprint_peak_bytes_max": physical_footprint_peak_bytes_max,
        "physical_footprint_sample_max_bytes": physical_footprint_sample_max_bytes,
        "graphics_total_sample_max_bytes": graphics_total_sample_max_bytes,
        "metal_current_allocated_size_sample_max_bytes": metal_current_allocated_size_sample_max_bytes,
        "startup_collapse_bytes": startup_collapse_bytes,
    }


def markdown_report(rows: list[dict[str, Any]]) -> str:
    lines = [
        "| Case | Steady | Peak MiB | Drop MiB | Physical MiB | Graphics MiB | Metal MiB | Residual MiB | Owned MiB | IOSurface MiB | IOAccel MiB | Redraws | Presents |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for row in rows:
        steady = row.get("steady") or {}
        lines.append(
            "| {label} | {offset:.1f}s | {peak} | {drop} | {physical} | {graphics} | {metal} | {residual} | {owned} | {iosurface} | {ioaccel} | {redraws} | {presents} |".format(
                label=row["label"],
                offset=float(steady.get("offset_secs", row.get("steady_offset_secs", 0.0))),
                peak=format_mib(row.get("physical_footprint_peak_bytes_max")),
                drop=format_mib(row.get("startup_collapse_bytes")),
                physical=format_mib(steady.get("physical_footprint_bytes")),
                graphics=format_mib(steady.get("graphics_total_bytes")),
                metal=format_mib(steady.get("metal_current_allocated_size_bytes")),
                residual=format_mib(steady.get("residual_bytes")),
                owned=format_mib(steady.get("owned_unmapped_memory_dirty_bytes")),
                iosurface=format_mib(steady.get("io_surface_dirty_bytes")),
                ioaccel=format_mib(steady.get("io_accelerator_dirty_bytes")),
                redraws=steady.get("redraw_count", "n/a"),
                presents=steady.get("present_count", "n/a"),
            )
        )
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--control-external", type=Path, required=True)
    parser.add_argument("--control-internal", type=Path, required=True)
    parser.add_argument("--fret-full-external", type=Path, required=True)
    parser.add_argument("--fret-full-internal", type=Path, required=True)
    parser.add_argument("--fret-empty-external", type=Path, required=True)
    parser.add_argument("--fret-empty-internal", type=Path, required=True)
    parser.add_argument("--steady-offset-secs", type=float, default=6.0)
    parser.add_argument("--out-dir", type=Path, required=True)
    args = parser.parse_args()

    rows = [
        summarize_case(
            "wgpu control",
            load_json(args.control_external),
            load_json(args.control_internal),
            args.steady_offset_secs,
        ),
        summarize_case(
            "fret compare full",
            load_json(args.fret_full_external),
            load_json(args.fret_full_internal),
            args.steady_offset_secs,
        ),
        summarize_case(
            "fret compare empty",
            load_json(args.fret_empty_external),
            load_json(args.fret_empty_internal),
            args.steady_offset_secs,
        ),
    ]

    comparisons = {}
    control_steady = rows[0]["steady"] or {}
    control_peak = rows[0].get("physical_footprint_peak_bytes_max")
    control_drop = rows[0].get("startup_collapse_bytes")
    for row in rows[1:]:
        steady = row["steady"] or {}
        comparisons[row["label"]] = {
            "physical_footprint_delta_bytes_vs_control": delta_or_none(
                steady.get("physical_footprint_bytes"),
                control_steady.get("physical_footprint_bytes"),
            ),
            "physical_footprint_peak_delta_bytes_vs_control": delta_or_none(
                row.get("physical_footprint_peak_bytes_max"),
                control_peak,
            ),
            "startup_collapse_delta_bytes_vs_control": delta_or_none(
                row.get("startup_collapse_bytes"),
                control_drop,
            ),
            "graphics_total_delta_bytes_vs_control": delta_or_none(
                steady.get("graphics_total_bytes"),
                control_steady.get("graphics_total_bytes"),
            ),
            "metal_current_allocated_size_delta_bytes_vs_control": delta_or_none(
                steady.get("metal_current_allocated_size_bytes"),
                control_steady.get("metal_current_allocated_size_bytes"),
            ),
            "residual_delta_bytes_vs_control": delta_or_none(
                steady.get("residual_bytes"),
                control_steady.get("residual_bytes"),
            ),
        }

    payload = {
        "schema_version": 1,
        "kind": "wgpu_hello_world_control_vs_fret_summary",
        "steady_offset_secs": args.steady_offset_secs,
        "rows": rows,
        "comparisons_vs_control": comparisons,
    }

    args.out_dir.mkdir(parents=True, exist_ok=True)
    (args.out_dir / "summary.json").write_text(json.dumps(payload, indent=2))
    (args.out_dir / "summary.md").write_text(markdown_report(rows) + "\n")
    print(json.dumps(payload, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
