#!/usr/bin/env python3
"""Summarize same-backend external+internal memory samples for Fret compare vs pure wgpu control."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

FOOTPRINT_VERBOSE_FAMILY_ALIASES = {
    "Owned physical footprint (unmapped) (graphics)": "owned_unmapped_graphics",
    "Owned physical footprint (unmapped)": "owned_unmapped",
    "IOSurface CAMetalLayer Display Drawable": "iosurface_cametallayer_display_drawable",
    "IOSurface": "iosurface",
    "IOAccelerator (graphics)": "ioaccelerator_graphics",
}


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


def format_surface_note(requested_surface: dict[str, Any], steady_surface: dict[str, Any]) -> str:
    actual_mode = steady_surface.get("present_mode") or requested_surface.get("present_mode")
    requested_mode = requested_surface.get("present_mode_raw")
    latency = steady_surface.get("desired_maximum_frame_latency")
    if latency is None:
        latency = requested_surface.get("desired_maximum_frame_latency")
    parts: list[str] = []
    if actual_mode and requested_mode and actual_mode != requested_mode:
        parts.append(f"mode={actual_mode}(req={requested_mode})")
    elif actual_mode:
        parts.append(f"mode={actual_mode}")
    elif requested_mode:
        parts.append(f"req={requested_mode}")
    if latency is not None:
        parts.append(f"lat={latency}")
    return ",".join(parts) if parts else "n/a"


def select_footprint_verbose_focus(external_sample: dict[str, Any]) -> dict[str, Any]:
    focus_families = (((external_sample.get("footprint_verbose") or {}).get("focus_families")) or {})
    return {
        alias: focus_families[family]
        for family, alias in FOOTPRINT_VERBOSE_FAMILY_ALIASES.items()
        if family in focus_families
    }


def focus_metric(focus: dict[str, Any], alias: str, key: str) -> int | None:
    return ((focus.get(alias) or {}).get(key))


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
        footprint_verbose_focus = select_footprint_verbose_focus(external_sample)
        internal_sample = find_sample(internal_samples, offset_secs)
        allocator = (internal_sample or {}).get("allocator") or {}
        runtime = (internal_sample or {}).get("runtime") or {}
        surface = (internal_sample or {}).get("surface") or {}
        renderer_perf = (internal_sample or {}).get("renderer_perf") or {}
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
                "surface": surface,
                "renderer_gpu_images_bytes_estimate": renderer_perf.get("gpu_images_bytes_estimate"),
                "renderer_gpu_render_targets_bytes_estimate": renderer_perf.get("gpu_render_targets_bytes_estimate"),
                "renderer_intermediate_peak_in_use_bytes": renderer_perf.get("intermediate_peak_in_use_bytes"),
                "renderer_intermediate_pool_free_bytes": renderer_perf.get("intermediate_pool_free_bytes"),
                "renderer_intermediate_pool_free_textures": renderer_perf.get("intermediate_pool_free_textures"),
                "renderer_path_intermediate_bytes_estimate": renderer_perf.get("path_intermediate_bytes_estimate"),
                "renderer_path_intermediate_msaa_bytes_estimate": renderer_perf.get("path_intermediate_msaa_bytes_estimate"),
                "renderer_path_intermediate_resolved_bytes_estimate": renderer_perf.get("path_intermediate_resolved_bytes_estimate"),
                "renderer_custom_effect_v3_pyramid_scratch_bytes_estimate": renderer_perf.get("custom_effect_v3_pyramid_scratch_bytes_estimate"),
                "renderer_clip_path_mask_cache_bytes_live": renderer_perf.get("clip_path_mask_cache_bytes_live"),
                "renderer_path_msaa_samples_effective": renderer_perf.get("path_msaa_samples_effective"),
                "renderer_path_draw_calls": renderer_perf.get("path_draw_calls"),
                "renderer_render_plan_custom_effect_chain_base_required_full_targets_max": renderer_perf.get("render_plan_custom_effect_chain_base_required_full_targets_max"),
                "renderer_render_plan_effect_chain_other_live_max_bytes": renderer_perf.get("render_plan_effect_chain_other_live_max_bytes"),
                "renderer_render_plan_estimated_peak_intermediate_bytes": renderer_perf.get("render_plan_estimated_peak_intermediate_bytes"),
                "vmmap_regions_sorted_top_dirty_region_type": key_metrics.get("vmmap_regions_sorted_top_dirty_region_type"),
                "vmmap_regions_sorted_top_dirty_detail": key_metrics.get("vmmap_regions_sorted_top_dirty_detail"),
                "vmmap_regions_sorted_top_dirty_bytes": key_metrics.get("vmmap_regions_sorted_top_dirty_bytes"),
                "footprint_verbose_focus": footprint_verbose_focus,
                "footprint_verbose_owned_unmapped_graphics_dirty_bytes": focus_metric(
                    footprint_verbose_focus, "owned_unmapped_graphics", "dirty_bytes_total"
                ),
                "footprint_verbose_owned_unmapped_graphics_virtual_bytes": focus_metric(
                    footprint_verbose_focus, "owned_unmapped_graphics", "virtual_bytes_total"
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
                    footprint_verbose_focus, "iosurface_cametallayer_display_drawable", "dirty_bytes_total"
                ),
                "footprint_verbose_ioaccelerator_graphics_dirty_bytes": focus_metric(
                    footprint_verbose_focus, "ioaccelerator_graphics", "dirty_bytes_total"
                ),
            }
        )

    steady = find_sample(paired_samples, steady_offset_secs)
    requested_runtime = internal_summary.get("requested_runtime") or {}
    scene = requested_runtime.get("scene") or {}
    requested_surface = requested_runtime.get("surface") or {}
    steady_surface = (steady or {}).get("surface") or {}
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
        "requested_runtime": requested_runtime,
        "active_mode": scene.get("active_mode")
        or ("continuous-redraw" if scene.get("continuous_redraw") else "n/a"),
        "surface_note": format_surface_note(requested_surface, steady_surface),
        "physical_footprint_peak_bytes_max": physical_footprint_peak_bytes_max,
        "physical_footprint_sample_max_bytes": physical_footprint_sample_max_bytes,
        "graphics_total_sample_max_bytes": graphics_total_sample_max_bytes,
        "metal_current_allocated_size_sample_max_bytes": metal_current_allocated_size_sample_max_bytes,
        "startup_collapse_bytes": startup_collapse_bytes,
    }


def markdown_report(rows: list[dict[str, Any]]) -> str:
    lines = [
        "| Case | Mode | Surface | Steady | Peak MiB | Drop MiB | Physical MiB | Graphics MiB | Metal MiB | Residual MiB | RImg MiB | RRT MiB | RInterm MiB | Owned MiB | IOSurface MiB | IOAccel MiB | Redraws | Presents |",
        "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for row in rows:
        steady = row.get("steady") or {}
        lines.append(
            "| {label} | {active_mode} | {surface_note} | {offset:.1f}s | {peak} | {drop} | {physical} | {graphics} | {metal} | {residual} | {renderer_images} | {renderer_render_targets} | {renderer_intermediate} | {owned} | {iosurface} | {ioaccel} | {redraws} | {presents} |".format(
                label=row["label"],
                active_mode=row.get("active_mode", "n/a"),
                surface_note=row.get("surface_note", "n/a"),
                offset=float(steady.get("offset_secs", row.get("steady_offset_secs", 0.0))),
                peak=format_mib(row.get("physical_footprint_peak_bytes_max")),
                drop=format_mib(row.get("startup_collapse_bytes")),
                physical=format_mib(steady.get("physical_footprint_bytes")),
                graphics=format_mib(steady.get("graphics_total_bytes")),
                metal=format_mib(steady.get("metal_current_allocated_size_bytes")),
                residual=format_mib(steady.get("residual_bytes")),
                renderer_images=format_mib(steady.get("renderer_gpu_images_bytes_estimate")),
                renderer_render_targets=format_mib(steady.get("renderer_gpu_render_targets_bytes_estimate")),
                renderer_intermediate=format_mib(steady.get("renderer_intermediate_peak_in_use_bytes")),
                owned=format_mib(steady.get("owned_unmapped_memory_dirty_bytes")),
                iosurface=format_mib(steady.get("io_surface_dirty_bytes")),
                ioaccel=format_mib(steady.get("io_accelerator_dirty_bytes")),
                redraws=steady.get("redraw_count", "n/a"),
                presents=steady.get("present_count", "n/a"),
            )
        )

    lines.extend([
        "",
        "| Case | Metal MiB | Residual MiB | Renderer images | Renderer RTs | Renderer interm peak | Path scratch | Pool free | Clip-mask cache | Owned gfx bucket |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |",
    ])
    for row in rows:
        steady = row["steady"]
        lines.append(
            "| {label} | {metal} | {residual} | {renderer_images} | {renderer_rts} | {renderer_intermediate} | {path_scratch} | {pool_free} | {clip_mask_cache} | {bucket} |".format(
                label=row["label"],
                metal=format_mib(steady.get("metal_current_allocated_size_bytes")),
                residual=format_mib(steady.get("residual_bytes")),
                renderer_images=format_mib(steady.get("renderer_gpu_images_bytes_estimate")),
                renderer_rts=format_mib(steady.get("renderer_gpu_render_targets_bytes_estimate")),
                renderer_intermediate=format_mib(steady.get("renderer_intermediate_peak_in_use_bytes")),
                path_scratch=format_mib(steady.get("renderer_path_intermediate_bytes_estimate")),
                pool_free=format_mib(steady.get("renderer_intermediate_pool_free_bytes")),
                clip_mask_cache=format_mib(steady.get("renderer_clip_path_mask_cache_bytes_live")),
                bucket=format_bucket_signature((steady.get("footprint_verbose_focus") or {}).get("owned_unmapped_graphics")),
            )
        )

    lines.extend([
        "",
        "| Case | Path MSAA | Path draws | Path scratch MSAA | Path scratch resolved | Pyramid scratch | CustomEffect full-target max | Effect other-live max | Pool free textures |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ])
    for row in rows:
        steady = row["steady"]
        lines.append(
            "| {label} | {path_msaa} | {path_draws} | {path_msaa_bytes} | {path_resolved_bytes} | {pyramid} | {custom_full_targets} | {other_live} | {pool_free_textures} |".format(
                label=row["label"],
                path_msaa=steady.get("renderer_path_msaa_samples_effective", "n/a"),
                path_draws=steady.get("renderer_path_draw_calls", "n/a"),
                path_msaa_bytes=format_mib(steady.get("renderer_path_intermediate_msaa_bytes_estimate")),
                path_resolved_bytes=format_mib(steady.get("renderer_path_intermediate_resolved_bytes_estimate")),
                pyramid=format_mib(steady.get("renderer_custom_effect_v3_pyramid_scratch_bytes_estimate")),
                custom_full_targets=steady.get("renderer_render_plan_custom_effect_chain_base_required_full_targets_max", "n/a"),
                other_live=format_mib(steady.get("renderer_render_plan_effect_chain_other_live_max_bytes")),
                pool_free_textures=steady.get("renderer_intermediate_pool_free_textures", "n/a"),
            )
        )

    if any((row.get("steady") or {}).get("footprint_verbose_focus") for row in rows):
        lines.extend([
            "",
            "| Case | Owned gfx MiB | Owned gfx rows | Owned gfx bucket | Owned plain MiB | Owned plain rows | Drawable MiB | IOAccel MiB |",
            "| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: |",
        ])
        for row in rows:
            steady = row.get("steady") or {}
            focus = steady.get("footprint_verbose_focus") or {}
            lines.append(
                "| {label} | {owned_gfx} | {owned_gfx_rows} | {owned_gfx_bucket} | {owned_plain} | {owned_plain_rows} | {drawables} | {ioaccel} |".format(
                    label=row["label"],
                    owned_gfx=format_mib(steady.get("footprint_verbose_owned_unmapped_graphics_dirty_bytes")),
                    owned_gfx_rows=steady.get("footprint_verbose_owned_unmapped_graphics_rows_total", "n/a"),
                    owned_gfx_bucket=format_bucket_signature(focus.get("owned_unmapped_graphics")),
                    owned_plain=format_mib(steady.get("footprint_verbose_owned_unmapped_dirty_bytes")),
                    owned_plain_rows=steady.get("footprint_verbose_owned_unmapped_rows_total", "n/a"),
                    drawables=format_mib(steady.get("footprint_verbose_drawable_iosurface_dirty_bytes")),
                    ioaccel=format_mib(steady.get("footprint_verbose_ioaccelerator_graphics_dirty_bytes")),
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
            "footprint_verbose_owned_unmapped_graphics_dirty_delta_bytes_vs_control": delta_or_none(
                steady.get("footprint_verbose_owned_unmapped_graphics_dirty_bytes"),
                control_steady.get("footprint_verbose_owned_unmapped_graphics_dirty_bytes"),
            ),
            "footprint_verbose_owned_unmapped_graphics_rows_delta_vs_control": delta_or_none(
                steady.get("footprint_verbose_owned_unmapped_graphics_rows_total"),
                control_steady.get("footprint_verbose_owned_unmapped_graphics_rows_total"),
            ),
            "footprint_verbose_owned_unmapped_dirty_delta_bytes_vs_control": delta_or_none(
                steady.get("footprint_verbose_owned_unmapped_dirty_bytes"),
                control_steady.get("footprint_verbose_owned_unmapped_dirty_bytes"),
            ),
            "footprint_verbose_drawable_iosurface_dirty_delta_bytes_vs_control": delta_or_none(
                steady.get("footprint_verbose_drawable_iosurface_dirty_bytes"),
                control_steady.get("footprint_verbose_drawable_iosurface_dirty_bytes"),
            ),
            "footprint_verbose_ioaccelerator_graphics_dirty_delta_bytes_vs_control": delta_or_none(
                steady.get("footprint_verbose_ioaccelerator_graphics_dirty_bytes"),
                control_steady.get("footprint_verbose_ioaccelerator_graphics_dirty_bytes"),
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
