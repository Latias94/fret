#!/usr/bin/env python3
"""Summarize same-scene hello-world memory comparisons between Fret and GPUI."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

FOOTPRINT_VERBOSE_FAMILY_ALIASES = {
    "Owned physical footprint (unmapped) (graphics)": "owned_unmapped_graphics",
    "Owned physical footprint (unmapped)": "owned_unmapped",
    "IOSurface CAMetalLayer Display Drawable": "iosurface_cametallayer_display_drawable",
    "IOAccelerator (graphics)": "ioaccelerator_graphics",
}


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


def select_footprint_verbose_focus(external_sample: dict[str, Any]) -> dict[str, Any]:
    focus_families = (((external_sample.get("footprint_verbose") or {}).get("focus_families")) or {})
    return {
        alias: focus_families[family]
        for family, alias in FOOTPRINT_VERBOSE_FAMILY_ALIASES.items()
        if family in focus_families
    }


def focus_metric(focus: dict[str, Any], alias: str, key: str) -> int | None:
    return (focus.get(alias) or {}).get(key)


def format_bucket_signature(family_summary: dict[str, Any] | None) -> str:
    if not family_summary:
        return "n/a"
    buckets = family_summary.get("dirty_page_buckets") or []
    if not buckets:
        buckets = family_summary.get("virtual_page_buckets") or []
    if not buckets:
        return "n/a"
    top_bucket = buckets[0]
    return f"{format_mib(top_bucket.get('bytes_per_row'))}×{top_bucket.get('rows_total', 0)}"


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
    footprint_verbose_focus = select_footprint_verbose_focus(external_sample)
    internal_sample = find_sample(internal_summary.get("samples") or [], steady_offset_secs) or {}
    runtime = internal_sample.get("runtime") or {}
    allocator = internal_sample.get("allocator") or {}
    renderer_perf = internal_sample.get("renderer_perf") or {}
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
        "renderer_path_intermediate_bytes_estimate": renderer_perf.get("path_intermediate_bytes_estimate"),
        "renderer_path_intermediate_msaa_bytes_estimate": renderer_perf.get("path_intermediate_msaa_bytes_estimate"),
        "renderer_path_intermediate_resolved_bytes_estimate": renderer_perf.get("path_intermediate_resolved_bytes_estimate"),
        "renderer_custom_effect_v3_pyramid_scratch_bytes_estimate": renderer_perf.get("custom_effect_v3_pyramid_scratch_bytes_estimate"),
        "renderer_intermediate_pool_free_bytes": renderer_perf.get("intermediate_pool_free_bytes"),
        "renderer_intermediate_pool_free_textures": renderer_perf.get("intermediate_pool_free_textures"),
        "renderer_path_msaa_samples_effective": renderer_perf.get("path_msaa_samples_effective"),
        "renderer_path_draw_calls": renderer_perf.get("path_draw_calls"),
        "renderer_clip_path_mask_cache_bytes_live": renderer_perf.get("clip_path_mask_cache_bytes_live"),
        "footprint_verbose_focus": footprint_verbose_focus,
        "footprint_verbose_owned_unmapped_graphics_dirty_bytes": focus_metric(
            footprint_verbose_focus, "owned_unmapped_graphics", "dirty_bytes_total"
        ),
        "footprint_verbose_owned_unmapped_graphics_rows_total": focus_metric(
            footprint_verbose_focus, "owned_unmapped_graphics", "rows_total"
        ),
        "footprint_verbose_owned_unmapped_dirty_bytes": focus_metric(
            footprint_verbose_focus, "owned_unmapped", "dirty_bytes_total"
        ),
        "footprint_verbose_owned_unmapped_rows_total": focus_metric(
            footprint_verbose_focus, "owned_unmapped", "rows_total"
        ),
        "footprint_verbose_drawable_iosurface_dirty_bytes": focus_metric(
            footprint_verbose_focus,
            "iosurface_cametallayer_display_drawable",
            "dirty_bytes_total",
        ),
        "footprint_verbose_ioaccelerator_graphics_dirty_bytes": focus_metric(
            footprint_verbose_focus, "ioaccelerator_graphics", "dirty_bytes_total"
        ),
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
        "footprint_verbose_owned_unmapped_graphics_dirty_delta_bytes": delta(
            "footprint_verbose_owned_unmapped_graphics_dirty_bytes"
        ),
        "footprint_verbose_owned_unmapped_graphics_rows_delta": delta(
            "footprint_verbose_owned_unmapped_graphics_rows_total"
        ),
        "footprint_verbose_owned_unmapped_dirty_delta_bytes": delta(
            "footprint_verbose_owned_unmapped_dirty_bytes"
        ),
        "footprint_verbose_drawable_iosurface_dirty_delta_bytes": delta(
            "footprint_verbose_drawable_iosurface_dirty_bytes"
        ),
        "footprint_verbose_ioaccelerator_graphics_dirty_delta_bytes": delta(
            "footprint_verbose_ioaccelerator_graphics_dirty_bytes"
        ),
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

    if any(row.get("footprint_verbose_focus") for row in rows):
        lines.extend([
            "",
            "| Framework | Case | Owned gfx MiB | Owned gfx rows | Owned gfx bucket | Owned plain MiB | Drawable MiB | IOAccel MiB |",
            "| --- | --- | ---: | ---: | --- | ---: | ---: | ---: |",
        ])
        for row in rows:
            lines.append(
                "| {framework} | {scene_kind} | {owned_gfx} | {owned_gfx_rows} | {owned_gfx_bucket} | {owned_plain} | {drawables} | {ioaccel} |".format(
                    framework=row["framework"],
                    scene_kind=row["scene_kind"],
                    owned_gfx=format_mib(row.get("footprint_verbose_owned_unmapped_graphics_dirty_bytes")),
                    owned_gfx_rows=row.get("footprint_verbose_owned_unmapped_graphics_rows_total", "n/a"),
                    owned_gfx_bucket=format_bucket_signature(
                        (row.get("footprint_verbose_focus") or {}).get("owned_unmapped_graphics")
                    ),
                    owned_plain=format_mib(row.get("footprint_verbose_owned_unmapped_dirty_bytes")),
                    drawables=format_mib(row.get("footprint_verbose_drawable_iosurface_dirty_bytes")),
                    ioaccel=format_mib(row.get("footprint_verbose_ioaccelerator_graphics_dirty_bytes")),
                )
            )

    lines.append("")
    lines.append("## Fret minus GPUI")
    lines.append("")
    lines.append("| Case | Mode | Physical Δ MiB | Graphics Δ MiB | Owned Δ MiB | IOSurface Δ MiB | IOAccel Δ MiB | Render Δ | Owned gfx Δ MiB | Owned gfx rows Δ | Drawable Δ MiB |")
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in deltas:
        lines.append(
            "| {scene_kind} | {active_mode} | {physical} | {graphics} | {owned} | {iosurface} | {ioaccel} | {render_delta} | {owned_gfx} | {owned_gfx_rows} | {drawable} |".format(
                scene_kind=row["scene_kind"],
                active_mode=row["active_mode"],
                physical=format_mib(row.get("physical_delta_bytes")),
                graphics=format_mib(row.get("graphics_delta_bytes")),
                owned=format_mib(row.get("owned_delta_bytes")),
                iosurface=format_mib(row.get("io_surface_delta_bytes")),
                ioaccel=format_mib(row.get("io_accelerator_delta_bytes")),
                render_delta=row.get("render_delta", "n/a"),
                owned_gfx=format_mib(row.get("footprint_verbose_owned_unmapped_graphics_dirty_delta_bytes")),
                owned_gfx_rows=row.get("footprint_verbose_owned_unmapped_graphics_rows_delta", "n/a"),
                drawable=format_mib(row.get("footprint_verbose_drawable_iosurface_dirty_delta_bytes")),
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
