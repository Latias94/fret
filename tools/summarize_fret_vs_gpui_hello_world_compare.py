#!/usr/bin/env python3
"""Summarize same-scene hello-world memory comparisons between Fret and GPUI."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--gpui-full-external", required=True)
    parser.add_argument("--gpui-full-internal", required=True)
    parser.add_argument("--gpui-empty-external", required=True)
    parser.add_argument("--gpui-empty-internal", required=True)
    parser.add_argument("--fret-full-external", required=True)
    parser.add_argument("--fret-full-internal", required=True)
    parser.add_argument("--fret-empty-external", required=True)
    parser.add_argument("--fret-empty-internal", required=True)
    parser.add_argument("--steady-offset-secs", type=float, default=6.0)
    parser.add_argument("--out-dir", required=True)
    return parser.parse_args()


def load_json(path: str) -> dict[str, Any]:
    return json.loads(Path(path).read_text())


def find_sample(samples: list[dict[str, Any]], offset_secs: float) -> dict[str, Any] | None:
    if not samples:
        return None
    return min(
        samples,
        key=lambda sample: abs(float(sample.get("offset_secs", 0.0)) - offset_secs),
    )


def format_mib(value: int | float | None) -> str:
    if value is None:
        return "n/a"
    return f"{float(value) / (1024.0 * 1024.0):.1f}"


def graphics_total_bytes(sample: dict[str, Any]) -> int | None:
    key_metrics = sample.get("key_metrics") or {}
    owned = key_metrics.get("owned_unmapped_memory_dirty_bytes")
    iosurface = key_metrics.get("io_surface_dirty_bytes") or 0
    ioaccel = key_metrics.get("io_accelerator_dirty_bytes") or 0
    if owned is None:
        return None
    return int(owned) + int(iosurface) + int(ioaccel)


def peak_physical_bytes(summary: dict[str, Any]) -> int | None:
    values = []
    for sample in summary.get("samples") or []:
        key_metrics = sample.get("key_metrics") or {}
        peak = key_metrics.get("physical_footprint_peak_bytes")
        if peak is not None:
            values.append(int(peak))
    if not values:
        return None
    return max(values)


def extract_case(
    *,
    label: str,
    framework: str,
    scene_kind: str,
    external_summary: dict[str, Any],
    internal_summary: dict[str, Any],
    steady_offset_secs: float,
) -> dict[str, Any]:
    external_sample = find_sample(external_summary.get("samples") or [], steady_offset_secs) or {}
    key_metrics = external_sample.get("key_metrics") or {}
    internal_sample = find_sample(internal_summary.get("samples") or [], steady_offset_secs) or {}
    runtime = internal_sample.get("runtime") or {}
    allocator = internal_sample.get("allocator") or {}
    scene = (internal_summary.get("requested_runtime") or {}).get("scene") or {}
    return {
        "label": label,
        "framework": framework,
        "scene_kind": scene_kind,
        "active_mode": scene.get("active_mode") or "n/a",
        "steady_offset_secs": float(external_sample.get("offset_secs", steady_offset_secs)),
        "physical_footprint_bytes": key_metrics.get("physical_footprint_bytes"),
        "physical_footprint_peak_bytes": peak_physical_bytes(external_summary),
        "owned_unmapped_memory_dirty_bytes": key_metrics.get("owned_unmapped_memory_dirty_bytes"),
        "io_surface_dirty_bytes": key_metrics.get("io_surface_dirty_bytes"),
        "io_accelerator_dirty_bytes": key_metrics.get("io_accelerator_dirty_bytes"),
        "graphics_total_bytes": graphics_total_bytes(external_sample),
        "render_count": runtime.get("render_count"),
        "frame_tick": runtime.get("frame_tick"),
        "metal_current_allocated_size_bytes": allocator.get("metal_current_allocated_size_bytes"),
        "external_summary": external_summary,
        "internal_summary": internal_summary,
    }


def delta_row(fret_row: dict[str, Any], gpui_row: dict[str, Any]) -> dict[str, Any]:
    def delta(key: str) -> int | None:
        left = fret_row.get(key)
        right = gpui_row.get(key)
        if left is None or right is None:
            return None
        return int(left) - int(right)

    return {
        "scene_kind": fret_row["scene_kind"],
        "active_mode": fret_row["active_mode"],
        "physical_delta_bytes": delta("physical_footprint_bytes"),
        "graphics_delta_bytes": delta("graphics_total_bytes"),
        "owned_delta_bytes": delta("owned_unmapped_memory_dirty_bytes"),
        "io_surface_delta_bytes": delta("io_surface_dirty_bytes"),
        "io_accelerator_delta_bytes": delta("io_accelerator_dirty_bytes"),
        "render_delta": delta("render_count"),
    }


def markdown_report(rows: list[dict[str, Any]], deltas: list[dict[str, Any]]) -> str:
    lines = [
        "| Framework | Case | Mode | Steady | Physical MiB | Graphics MiB | Owned MiB | IOSurface MiB | IOAccel MiB | Metal MiB | Renders |",
        "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for row in rows:
        lines.append(
            "| {framework} | {scene_kind} | {active_mode} | {steady:.1f}s | {physical} | {graphics} | {owned} | {iosurface} | {ioaccel} | {metal} | {renders} |".format(
                framework=row["framework"],
                scene_kind=row["scene_kind"],
                active_mode=row["active_mode"],
                steady=row["steady_offset_secs"],
                physical=format_mib(row.get("physical_footprint_bytes")),
                graphics=format_mib(row.get("graphics_total_bytes")),
                owned=format_mib(row.get("owned_unmapped_memory_dirty_bytes")),
                iosurface=format_mib(row.get("io_surface_dirty_bytes")),
                ioaccel=format_mib(row.get("io_accelerator_dirty_bytes")),
                metal=format_mib(row.get("metal_current_allocated_size_bytes")),
                renders=row.get("render_count", "n/a"),
            )
        )
    lines.append("")
    lines.append("## Fret minus GPUI")
    lines.append("")
    lines.append("| Case | Mode | Physical Δ MiB | Graphics Δ MiB | Owned Δ MiB | IOSurface Δ MiB | IOAccel Δ MiB | Render Δ |")
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in deltas:
        lines.append(
            "| {scene_kind} | {active_mode} | {physical} | {graphics} | {owned} | {iosurface} | {ioaccel} | {render_delta} |".format(
                scene_kind=row["scene_kind"],
                active_mode=row["active_mode"],
                physical=format_mib(row.get("physical_delta_bytes")),
                graphics=format_mib(row.get("graphics_delta_bytes")),
                owned=format_mib(row.get("owned_delta_bytes")),
                iosurface=format_mib(row.get("io_surface_delta_bytes")),
                ioaccel=format_mib(row.get("io_accelerator_delta_bytes")),
                render_delta=row.get("render_delta", "n/a"),
            )
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    gpui_full = extract_case(
        label="gpui full",
        framework="gpui",
        scene_kind="full",
        external_summary=load_json(args.gpui_full_external),
        internal_summary=load_json(args.gpui_full_internal),
        steady_offset_secs=args.steady_offset_secs,
    )
    gpui_empty = extract_case(
        label="gpui empty",
        framework="gpui",
        scene_kind="empty",
        external_summary=load_json(args.gpui_empty_external),
        internal_summary=load_json(args.gpui_empty_internal),
        steady_offset_secs=args.steady_offset_secs,
    )
    fret_full = extract_case(
        label="fret full",
        framework="fret",
        scene_kind="full",
        external_summary=load_json(args.fret_full_external),
        internal_summary=load_json(args.fret_full_internal),
        steady_offset_secs=args.steady_offset_secs,
    )
    fret_empty = extract_case(
        label="fret empty",
        framework="fret",
        scene_kind="empty",
        external_summary=load_json(args.fret_empty_external),
        internal_summary=load_json(args.fret_empty_internal),
        steady_offset_secs=args.steady_offset_secs,
    )

    rows = [gpui_empty, gpui_full, fret_empty, fret_full]
    deltas = [delta_row(fret_empty, gpui_empty), delta_row(fret_full, gpui_full)]
    summary = {
        "schema_version": 1,
        "kind": "fret_vs_gpui_hello_world_compare_summary",
        "steady_offset_secs": args.steady_offset_secs,
        "rows": rows,
        "deltas": deltas,
    }
    (out_dir / "summary.json").write_text(json.dumps(summary, indent=2))
    (out_dir / "summary.md").write_text(markdown_report(rows, deltas))
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
