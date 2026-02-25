use super::*;

fn json_u64(v: &serde_json::Value, key: &str) -> u64 {
    v.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
}

pub(super) fn print_perf_no_last_bundle_dir(src: &Path, sort: BundleStatsSort, repeat: Option<usize>) {
    match repeat {
        Some(repeat) => {
            println!(
                "PERF {} sort={} repeat={} (no last_bundle_dir recorded)",
                src.display(),
                sort.as_str(),
                repeat
            );
        }
        None => {
            println!(
                "PERF {} sort={} (no last_bundle_dir recorded)",
                src.display(),
                sort.as_str()
            );
        }
    }
}

pub(super) fn push_perf_json_no_last_bundle_dir(
    perf_json_rows: &mut Vec<serde_json::Value>,
    script: String,
    sort: BundleStatsSort,
    repeat: Option<usize>,
) {
    let mut obj = serde_json::Map::new();
    obj.insert("script".to_string(), serde_json::Value::String(script));
    obj.insert(
        "sort".to_string(),
        serde_json::Value::String(sort.as_str().to_string()),
    );
    if let Some(repeat) = repeat {
        obj.insert(
            "repeat".to_string(),
            serde_json::Value::Number(serde_json::Number::from(repeat as u64)),
        );
    }
    obj.insert(
        "error".to_string(),
        serde_json::Value::String("no_last_bundle_dir".to_string()),
    );
    perf_json_rows.push(serde_json::Value::Object(obj));
}

pub(super) fn print_perf_repeat_summary(
    src: &Path,
    sort: BundleStatsSort,
    repeat: usize,
    total: &serde_json::Value,
    layout: &serde_json::Value,
    solve: &serde_json::Value,
    prepaint: &serde_json::Value,
    paint: &serde_json::Value,
    dispatch: &serde_json::Value,
    hit_test: &serde_json::Value,
) {
    println!(
        "PERF {} sort={} repeat={} p50.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{} p95.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{} max.us(total/layout/solve/prepaint/paint/dispatch/hit_test)={}/{}/{}/{}/{}/{}/{}",
        src.display(),
        sort.as_str(),
        repeat,
        json_u64(total, "p50"),
        json_u64(layout, "p50"),
        json_u64(solve, "p50"),
        json_u64(prepaint, "p50"),
        json_u64(paint, "p50"),
        json_u64(dispatch, "p50"),
        json_u64(hit_test, "p50"),
        json_u64(total, "p95"),
        json_u64(layout, "p95"),
        json_u64(solve, "p95"),
        json_u64(prepaint, "p95"),
        json_u64(paint, "p95"),
        json_u64(dispatch, "p95"),
        json_u64(hit_test, "p95"),
        json_u64(total, "max"),
        json_u64(layout, "max"),
        json_u64(solve, "max"),
        json_u64(prepaint, "max"),
        json_u64(paint, "max"),
        json_u64(dispatch, "max"),
        json_u64(hit_test, "max"),
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn push_perf_json_repeat_summary_row(
    perf_json_rows: &mut Vec<serde_json::Value>,
    src: &Path,
    sort: BundleStatsSort,
    repeat: usize,
    runs_json: &[serde_json::Value],
    runs_total: &[u64],
    runs_layout: &[u64],
    runs_solve: &[u64],
    runs_prepaint: &[u64],
    runs_paint: &[u64],
    runs_dispatch: &[u64],
    runs_hit_test: &[u64],
    runs_pointer_move_dispatch: &[u64],
    runs_pointer_move_hit_test: &[u64],
    runs_pointer_move_global_changes: &[u64],
    script_worst: Option<&(u64, PathBuf, u64)>,
) {
    let mut top_frame_arena_capacity_estimate_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_frame_arena_grow_events: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_element_children_vec_pool_reuses: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_element_children_vec_pool_misses: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_view_cache_contained_relayouts: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_view_cache_roots_total: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_view_cache_roots_reused: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_view_cache_roots_cache_key_mismatch: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_view_cache_roots_needs_rerender: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_view_cache_roots_layout_invalidated: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_cache_roots_contained_relayout: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_set_children_barrier_writes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_barrier_relayouts_scheduled: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_barrier_relayouts_performed: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_virtual_list_visible_range_checks: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_virtual_list_visible_range_refreshes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_encode_scene_us: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_prepare_text_us: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_draw_calls: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_pipeline_switches: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_bind_group_switches: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_misses: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_material_quad_ops: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_material_sampled_quad_ops: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_material_distinct: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_material_unknown_ids: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_material_degraded_due_to_budget: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_text_atlas_upload_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_text_atlas_evicted_pages: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_svg_upload_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_image_upload_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_svg_raster_cache_misses: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_svg_raster_budget_evictions: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_svg_rasters_live: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_svg_mask_atlas_pages_live: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_svg_mask_atlas_used_px: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_intermediate_budget_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_intermediate_in_use_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_intermediate_peak_in_use_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_intermediate_release_targets: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_intermediate_pool_allocations: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_intermediate_pool_reuses: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_intermediate_pool_releases: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_intermediate_pool_evictions: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_intermediate_pool_free_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_intermediate_pool_free_textures: Vec<u64> = Vec::with_capacity(repeat);

    for run in runs_json {
        top_frame_arena_capacity_estimate_bytes
            .push(json_u64(run, "top_frame_arena_capacity_estimate_bytes"));
        top_frame_arena_grow_events.push(json_u64(run, "top_frame_arena_grow_events"));
        top_element_children_vec_pool_reuses.push(json_u64(run, "top_element_children_vec_pool_reuses"));
        top_element_children_vec_pool_misses.push(json_u64(run, "top_element_children_vec_pool_misses"));
        top_view_cache_contained_relayouts.push(json_u64(run, "top_view_cache_contained_relayouts"));
        top_view_cache_roots_total.push(json_u64(run, "top_view_cache_roots_total"));
        top_view_cache_roots_reused.push(json_u64(run, "top_view_cache_roots_reused"));
        top_view_cache_roots_cache_key_mismatch
            .push(json_u64(run, "top_view_cache_roots_cache_key_mismatch"));
        top_view_cache_roots_needs_rerender.push(json_u64(run, "top_view_cache_roots_needs_rerender"));
        top_view_cache_roots_layout_invalidated
            .push(json_u64(run, "top_view_cache_roots_layout_invalidated"));
        top_cache_roots_contained_relayout.push(json_u64(run, "top_cache_roots_contained_relayout"));
        top_set_children_barrier_writes.push(json_u64(run, "top_set_children_barrier_writes"));
        top_barrier_relayouts_scheduled.push(json_u64(run, "top_barrier_relayouts_scheduled"));
        top_barrier_relayouts_performed.push(json_u64(run, "top_barrier_relayouts_performed"));
        top_virtual_list_visible_range_checks
            .push(json_u64(run, "top_virtual_list_visible_range_checks"));
        top_virtual_list_visible_range_refreshes
            .push(json_u64(run, "top_virtual_list_visible_range_refreshes"));
        top_renderer_encode_scene_us.push(json_u64(run, "top_renderer_encode_scene_us"));
        top_renderer_prepare_text_us.push(json_u64(run, "top_renderer_prepare_text_us"));
        top_renderer_draw_calls.push(json_u64(run, "top_renderer_draw_calls"));
        top_renderer_pipeline_switches.push(json_u64(run, "top_renderer_pipeline_switches"));
        top_renderer_bind_group_switches.push(json_u64(run, "top_renderer_bind_group_switches"));
        top_renderer_scene_encoding_cache_misses
            .push(json_u64(run, "top_renderer_scene_encoding_cache_misses"));
        top_renderer_material_quad_ops.push(json_u64(run, "top_renderer_material_quad_ops"));
        top_renderer_material_sampled_quad_ops
            .push(json_u64(run, "top_renderer_material_sampled_quad_ops"));
        top_renderer_material_distinct.push(json_u64(run, "top_renderer_material_distinct"));
        top_renderer_material_unknown_ids.push(json_u64(run, "top_renderer_material_unknown_ids"));
        top_renderer_material_degraded_due_to_budget
            .push(json_u64(run, "top_renderer_material_degraded_due_to_budget"));
        top_renderer_text_atlas_upload_bytes.push(json_u64(run, "top_renderer_text_atlas_upload_bytes"));
        top_renderer_text_atlas_evicted_pages
            .push(json_u64(run, "top_renderer_text_atlas_evicted_pages"));
        top_renderer_svg_upload_bytes.push(json_u64(run, "top_renderer_svg_upload_bytes"));
        top_renderer_image_upload_bytes.push(json_u64(run, "top_renderer_image_upload_bytes"));
        top_renderer_svg_raster_cache_misses
            .push(json_u64(run, "top_renderer_svg_raster_cache_misses"));
        top_renderer_svg_raster_budget_evictions
            .push(json_u64(run, "top_renderer_svg_raster_budget_evictions"));
        top_renderer_svg_rasters_live.push(json_u64(run, "top_renderer_svg_rasters_live"));
        top_renderer_svg_mask_atlas_pages_live
            .push(json_u64(run, "top_renderer_svg_mask_atlas_pages_live"));
        top_renderer_svg_mask_atlas_used_px.push(json_u64(run, "top_renderer_svg_mask_atlas_used_px"));
        top_renderer_intermediate_budget_bytes
            .push(json_u64(run, "top_renderer_intermediate_budget_bytes"));
        top_renderer_intermediate_in_use_bytes
            .push(json_u64(run, "top_renderer_intermediate_in_use_bytes"));
        top_renderer_intermediate_peak_in_use_bytes
            .push(json_u64(run, "top_renderer_intermediate_peak_in_use_bytes"));
        top_renderer_intermediate_release_targets
            .push(json_u64(run, "top_renderer_intermediate_release_targets"));
        top_renderer_intermediate_pool_allocations
            .push(json_u64(run, "top_renderer_intermediate_pool_allocations"));
        top_renderer_intermediate_pool_reuses
            .push(json_u64(run, "top_renderer_intermediate_pool_reuses"));
        top_renderer_intermediate_pool_releases
            .push(json_u64(run, "top_renderer_intermediate_pool_releases"));
        top_renderer_intermediate_pool_evictions
            .push(json_u64(run, "top_renderer_intermediate_pool_evictions"));
        top_renderer_intermediate_pool_free_bytes
            .push(json_u64(run, "top_renderer_intermediate_pool_free_bytes"));
        top_renderer_intermediate_pool_free_textures
            .push(json_u64(run, "top_renderer_intermediate_pool_free_textures"));
    }

    perf_json_rows.push(serde_json::json!({
        "script": src.display().to_string(),
        "sort": sort.as_str(),
        "repeat": repeat,
        "runs": runs_json,
        "stats": {
            "total_time_us": summarize_times_us(runs_total),
            "layout_time_us": summarize_times_us(runs_layout),
            "layout_engine_solve_time_us": summarize_times_us(runs_solve),
            "prepaint_time_us": summarize_times_us(runs_prepaint),
            "paint_time_us": summarize_times_us(runs_paint),
            "dispatch_time_us": summarize_times_us(runs_dispatch),
            "hit_test_time_us": summarize_times_us(runs_hit_test),
            "pointer_move_max_dispatch_time_us": summarize_times_us(runs_pointer_move_dispatch),
            "pointer_move_max_hit_test_time_us": summarize_times_us(runs_pointer_move_hit_test),
            "pointer_move_snapshots_with_global_changes": summarize_times_us(runs_pointer_move_global_changes),
            "top_frame_arena_capacity_estimate_bytes": summarize_times_us(&top_frame_arena_capacity_estimate_bytes),
            "top_frame_arena_grow_events": summarize_times_us(&top_frame_arena_grow_events),
            "top_element_children_vec_pool_reuses": summarize_times_us(&top_element_children_vec_pool_reuses),
            "top_element_children_vec_pool_misses": summarize_times_us(&top_element_children_vec_pool_misses),
            "top_view_cache_contained_relayouts": summarize_times_us(&top_view_cache_contained_relayouts),
            "top_view_cache_roots_total": summarize_times_us(&top_view_cache_roots_total),
            "top_view_cache_roots_reused": summarize_times_us(&top_view_cache_roots_reused),
            "top_view_cache_roots_cache_key_mismatch": summarize_times_us(&top_view_cache_roots_cache_key_mismatch),
            "top_view_cache_roots_needs_rerender": summarize_times_us(&top_view_cache_roots_needs_rerender),
            "top_view_cache_roots_layout_invalidated": summarize_times_us(&top_view_cache_roots_layout_invalidated),
            "top_cache_roots_contained_relayout": summarize_times_us(&top_cache_roots_contained_relayout),
            "top_set_children_barrier_writes": summarize_times_us(&top_set_children_barrier_writes),
            "top_barrier_relayouts_scheduled": summarize_times_us(&top_barrier_relayouts_scheduled),
            "top_barrier_relayouts_performed": summarize_times_us(&top_barrier_relayouts_performed),
            "top_virtual_list_visible_range_checks": summarize_times_us(&top_virtual_list_visible_range_checks),
            "top_virtual_list_visible_range_refreshes": summarize_times_us(&top_virtual_list_visible_range_refreshes),
            "top_renderer_encode_scene_us": summarize_times_us(&top_renderer_encode_scene_us),
            "top_renderer_prepare_text_us": summarize_times_us(&top_renderer_prepare_text_us),
            "top_renderer_draw_calls": summarize_times_us(&top_renderer_draw_calls),
            "top_renderer_pipeline_switches": summarize_times_us(&top_renderer_pipeline_switches),
            "top_renderer_bind_group_switches": summarize_times_us(&top_renderer_bind_group_switches),
            "top_renderer_scene_encoding_cache_misses": summarize_times_us(&top_renderer_scene_encoding_cache_misses),
            "top_renderer_material_quad_ops": summarize_times_us(&top_renderer_material_quad_ops),
            "top_renderer_material_sampled_quad_ops": summarize_times_us(&top_renderer_material_sampled_quad_ops),
            "top_renderer_material_distinct": summarize_times_us(&top_renderer_material_distinct),
            "top_renderer_material_unknown_ids": summarize_times_us(&top_renderer_material_unknown_ids),
            "top_renderer_material_degraded_due_to_budget": summarize_times_us(&top_renderer_material_degraded_due_to_budget),
            "top_renderer_text_atlas_upload_bytes": summarize_times_us(&top_renderer_text_atlas_upload_bytes),
            "top_renderer_text_atlas_evicted_pages": summarize_times_us(&top_renderer_text_atlas_evicted_pages),
            "top_renderer_svg_upload_bytes": summarize_times_us(&top_renderer_svg_upload_bytes),
            "top_renderer_image_upload_bytes": summarize_times_us(&top_renderer_image_upload_bytes),
            "top_renderer_svg_raster_cache_misses": summarize_times_us(&top_renderer_svg_raster_cache_misses),
            "top_renderer_svg_raster_budget_evictions": summarize_times_us(&top_renderer_svg_raster_budget_evictions),
            "top_renderer_svg_rasters_live": summarize_times_us(&top_renderer_svg_rasters_live),
            "top_renderer_svg_mask_atlas_pages_live": summarize_times_us(&top_renderer_svg_mask_atlas_pages_live),
            "top_renderer_svg_mask_atlas_used_px": summarize_times_us(&top_renderer_svg_mask_atlas_used_px),
            "top_renderer_intermediate_budget_bytes": summarize_times_us(&top_renderer_intermediate_budget_bytes),
            "top_renderer_intermediate_in_use_bytes": summarize_times_us(&top_renderer_intermediate_in_use_bytes),
            "top_renderer_intermediate_peak_in_use_bytes": summarize_times_us(&top_renderer_intermediate_peak_in_use_bytes),
            "top_renderer_intermediate_release_targets": summarize_times_us(&top_renderer_intermediate_release_targets),
            "top_renderer_intermediate_pool_allocations": summarize_times_us(&top_renderer_intermediate_pool_allocations),
            "top_renderer_intermediate_pool_reuses": summarize_times_us(&top_renderer_intermediate_pool_reuses),
            "top_renderer_intermediate_pool_releases": summarize_times_us(&top_renderer_intermediate_pool_releases),
            "top_renderer_intermediate_pool_evictions": summarize_times_us(&top_renderer_intermediate_pool_evictions),
            "top_renderer_intermediate_pool_free_bytes": summarize_times_us(&top_renderer_intermediate_pool_free_bytes),
            "top_renderer_intermediate_pool_free_textures": summarize_times_us(&top_renderer_intermediate_pool_free_textures),
        },
        "worst_run": script_worst
            .map(|(us, bundle, run_index)| serde_json::json!({
                "top_total_time_us": us,
                "bundle": bundle.display().to_string(),
                "run_index": run_index,
            })),
    }));
}
