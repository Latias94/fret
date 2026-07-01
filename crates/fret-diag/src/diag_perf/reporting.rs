use super::*;

fn json_u64(v: &serde_json::Value, key: &str) -> u64 {
    v.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
}

pub(super) fn print_perf_no_last_bundle_dir(
    src: &Path,
    sort: BundleStatsSort,
    repeat: Option<usize>,
) {
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
    let mut top_element_children_vec_pool_grow_events: Vec<u64> = Vec::with_capacity(repeat);
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
    let mut top_code_editor_row_text_get_calls: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_text_hits: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_text_misses: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_text_resets: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_text_hit_rate_pct: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_text_us: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_rows_painted: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_rows_scene_replayed: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_rows_scene_stored: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_scene_ops_stored: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_scene_replay_hit_rate_pct: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_scene_cache_get_calls: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_scene_cache_hits: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_scene_cache_misses: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_scene_cache_resets: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_scene_cache_hit_rate_pct: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_scene_prepaint_plan_us: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_scene_prepaint_probe_us: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_code_editor_row_scene_prepaint_key_compare_us: Vec<u64> =
        Vec::with_capacity(repeat);
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
    let mut top_renderer_uniform_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_instance_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_vertex_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_quad_instance_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_quad_instance_write_count: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_path_paint_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_path_paint_write_count: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_text_paint_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_text_paint_write_count: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_viewport_vertex_bytes: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_viewport_vertex_write_count: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_text_glyph_instance_bytes: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_text_glyph_instance_write_count: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_text_vertex_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_text_vertex_write_count: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_path_vertex_bytes: Vec<u64> = Vec::with_capacity(repeat);
    let mut top_renderer_geometry_upload_path_vertex_write_count: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_cold_start: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_format_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_viewport_size_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_scale_factor_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_scene_ops_len_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_render_targets_generation_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_images_generation_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_text_atlas_revision_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_text_quality_key_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_materials_generation_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_material_paint_budget_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_material_distinct_budget_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_scene_encoding_cache_miss_custom_effects_generation_changed: Vec<u64> =
        Vec::with_capacity(repeat);
    let mut top_renderer_encode_scene_text_ops: Vec<u64> = Vec::with_capacity(repeat);
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
        top_element_children_vec_pool_reuses
            .push(json_u64(run, "top_element_children_vec_pool_reuses"));
        top_element_children_vec_pool_misses
            .push(json_u64(run, "top_element_children_vec_pool_misses"));
        top_element_children_vec_pool_grow_events
            .push(json_u64(run, "top_element_children_vec_pool_grow_events"));
        top_view_cache_contained_relayouts
            .push(json_u64(run, "top_view_cache_contained_relayouts"));
        top_view_cache_roots_total.push(json_u64(run, "top_view_cache_roots_total"));
        top_view_cache_roots_reused.push(json_u64(run, "top_view_cache_roots_reused"));
        top_view_cache_roots_cache_key_mismatch
            .push(json_u64(run, "top_view_cache_roots_cache_key_mismatch"));
        top_view_cache_roots_needs_rerender
            .push(json_u64(run, "top_view_cache_roots_needs_rerender"));
        top_view_cache_roots_layout_invalidated
            .push(json_u64(run, "top_view_cache_roots_layout_invalidated"));
        top_cache_roots_contained_relayout
            .push(json_u64(run, "top_cache_roots_contained_relayout"));
        top_set_children_barrier_writes.push(json_u64(run, "top_set_children_barrier_writes"));
        top_barrier_relayouts_scheduled.push(json_u64(run, "top_barrier_relayouts_scheduled"));
        top_barrier_relayouts_performed.push(json_u64(run, "top_barrier_relayouts_performed"));
        top_virtual_list_visible_range_checks
            .push(json_u64(run, "top_virtual_list_visible_range_checks"));
        top_virtual_list_visible_range_refreshes
            .push(json_u64(run, "top_virtual_list_visible_range_refreshes"));
        top_code_editor_row_text_get_calls
            .push(json_u64(run, "top_code_editor_row_text_get_calls"));
        top_code_editor_row_text_hits.push(json_u64(run, "top_code_editor_row_text_hits"));
        top_code_editor_row_text_misses.push(json_u64(run, "top_code_editor_row_text_misses"));
        top_code_editor_row_text_resets.push(json_u64(run, "top_code_editor_row_text_resets"));
        top_code_editor_row_text_hit_rate_pct
            .push(json_u64(run, "top_code_editor_row_text_hit_rate_pct"));
        top_code_editor_row_text_us.push(json_u64(run, "top_code_editor_row_text_us"));
        top_code_editor_rows_painted.push(json_u64(run, "top_code_editor_rows_painted"));
        top_code_editor_rows_scene_replayed
            .push(json_u64(run, "top_code_editor_rows_scene_replayed"));
        top_code_editor_rows_scene_stored.push(json_u64(run, "top_code_editor_rows_scene_stored"));
        top_code_editor_row_scene_ops_stored
            .push(json_u64(run, "top_code_editor_row_scene_ops_stored"));
        top_code_editor_row_scene_replay_hit_rate_pct.push(json_u64(
            run,
            "top_code_editor_row_scene_replay_hit_rate_pct",
        ));
        top_code_editor_row_scene_cache_get_calls
            .push(json_u64(run, "top_code_editor_row_scene_cache_get_calls"));
        top_code_editor_row_scene_cache_hits
            .push(json_u64(run, "top_code_editor_row_scene_cache_hits"));
        top_code_editor_row_scene_cache_misses
            .push(json_u64(run, "top_code_editor_row_scene_cache_misses"));
        top_code_editor_row_scene_cache_resets
            .push(json_u64(run, "top_code_editor_row_scene_cache_resets"));
        top_code_editor_row_scene_cache_hit_rate_pct.push(json_u64(
            run,
            "top_code_editor_row_scene_cache_hit_rate_pct",
        ));
        top_code_editor_row_scene_prepaint_plan_us
            .push(json_u64(run, "top_code_editor_row_scene_prepaint_plan_us"));
        top_code_editor_row_scene_prepaint_probe_us
            .push(json_u64(run, "top_code_editor_row_scene_prepaint_probe_us"));
        top_code_editor_row_scene_prepaint_key_compare_us.push(json_u64(
            run,
            "top_code_editor_row_scene_prepaint_key_compare_us",
        ));
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
        top_renderer_material_degraded_due_to_budget.push(json_u64(
            run,
            "top_renderer_material_degraded_due_to_budget",
        ));
        top_renderer_text_atlas_upload_bytes
            .push(json_u64(run, "top_renderer_text_atlas_upload_bytes"));
        top_renderer_text_atlas_evicted_pages
            .push(json_u64(run, "top_renderer_text_atlas_evicted_pages"));
        top_renderer_svg_upload_bytes.push(json_u64(run, "top_renderer_svg_upload_bytes"));
        top_renderer_image_upload_bytes.push(json_u64(run, "top_renderer_image_upload_bytes"));
        top_renderer_uniform_bytes.push(json_u64(run, "top_renderer_uniform_bytes"));
        top_renderer_instance_bytes.push(json_u64(run, "top_renderer_instance_bytes"));
        top_renderer_vertex_bytes.push(json_u64(run, "top_renderer_vertex_bytes"));
        top_renderer_geometry_upload_quad_instance_bytes.push(json_u64(
            run,
            "top_renderer_geometry_upload_quad_instance_bytes",
        ));
        top_renderer_geometry_upload_quad_instance_write_count.push(json_u64(
            run,
            "top_renderer_geometry_upload_quad_instance_write_count",
        ));
        top_renderer_geometry_upload_path_paint_bytes.push(json_u64(
            run,
            "top_renderer_geometry_upload_path_paint_bytes",
        ));
        top_renderer_geometry_upload_path_paint_write_count.push(json_u64(
            run,
            "top_renderer_geometry_upload_path_paint_write_count",
        ));
        top_renderer_geometry_upload_text_paint_bytes.push(json_u64(
            run,
            "top_renderer_geometry_upload_text_paint_bytes",
        ));
        top_renderer_geometry_upload_text_paint_write_count.push(json_u64(
            run,
            "top_renderer_geometry_upload_text_paint_write_count",
        ));
        top_renderer_geometry_upload_viewport_vertex_bytes.push(json_u64(
            run,
            "top_renderer_geometry_upload_viewport_vertex_bytes",
        ));
        top_renderer_geometry_upload_viewport_vertex_write_count.push(json_u64(
            run,
            "top_renderer_geometry_upload_viewport_vertex_write_count",
        ));
        top_renderer_geometry_upload_text_glyph_instance_bytes.push(json_u64(
            run,
            "top_renderer_geometry_upload_text_glyph_instance_bytes",
        ));
        top_renderer_geometry_upload_text_glyph_instance_write_count.push(json_u64(
            run,
            "top_renderer_geometry_upload_text_glyph_instance_write_count",
        ));
        top_renderer_geometry_upload_text_vertex_bytes.push(json_u64(
            run,
            "top_renderer_geometry_upload_text_vertex_bytes",
        ));
        top_renderer_geometry_upload_text_vertex_write_count.push(json_u64(
            run,
            "top_renderer_geometry_upload_text_vertex_write_count",
        ));
        top_renderer_geometry_upload_path_vertex_bytes.push(json_u64(
            run,
            "top_renderer_geometry_upload_path_vertex_bytes",
        ));
        top_renderer_geometry_upload_path_vertex_write_count.push(json_u64(
            run,
            "top_renderer_geometry_upload_path_vertex_write_count",
        ));
        top_renderer_scene_encoding_cache_miss_cold_start.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_cold_start",
        ));
        top_renderer_scene_encoding_cache_miss_format_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_format_changed",
        ));
        top_renderer_scene_encoding_cache_miss_viewport_size_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_viewport_size_changed",
        ));
        top_renderer_scene_encoding_cache_miss_scale_factor_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_scale_factor_changed",
        ));
        top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed",
        ));
        top_renderer_scene_encoding_cache_miss_scene_ops_len_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_scene_ops_len_changed",
        ));
        top_renderer_scene_encoding_cache_miss_render_targets_generation_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_render_targets_generation_changed",
        ));
        top_renderer_scene_encoding_cache_miss_images_generation_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_images_generation_changed",
        ));
        top_renderer_scene_encoding_cache_miss_text_atlas_revision_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_text_atlas_revision_changed",
        ));
        top_renderer_scene_encoding_cache_miss_text_quality_key_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_text_quality_key_changed",
        ));
        top_renderer_scene_encoding_cache_miss_materials_generation_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_materials_generation_changed",
        ));
        top_renderer_scene_encoding_cache_miss_material_paint_budget_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_material_paint_budget_changed",
        ));
        top_renderer_scene_encoding_cache_miss_material_distinct_budget_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_material_distinct_budget_changed",
        ));
        top_renderer_scene_encoding_cache_miss_custom_effects_generation_changed.push(json_u64(
            run,
            "top_renderer_scene_encoding_cache_miss_custom_effects_generation_changed",
        ));
        top_renderer_encode_scene_text_ops
            .push(json_u64(run, "top_renderer_encode_scene_text_ops"));
        top_renderer_svg_raster_cache_misses
            .push(json_u64(run, "top_renderer_svg_raster_cache_misses"));
        top_renderer_svg_raster_budget_evictions
            .push(json_u64(run, "top_renderer_svg_raster_budget_evictions"));
        top_renderer_svg_rasters_live.push(json_u64(run, "top_renderer_svg_rasters_live"));
        top_renderer_svg_mask_atlas_pages_live
            .push(json_u64(run, "top_renderer_svg_mask_atlas_pages_live"));
        top_renderer_svg_mask_atlas_used_px
            .push(json_u64(run, "top_renderer_svg_mask_atlas_used_px"));
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
        top_renderer_intermediate_pool_free_textures.push(json_u64(
            run,
            "top_renderer_intermediate_pool_free_textures",
        ));
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
            "top_element_children_vec_pool_grow_events": summarize_times_us(&top_element_children_vec_pool_grow_events),
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
            "top_code_editor_row_text_get_calls": summarize_times_us(&top_code_editor_row_text_get_calls),
            "top_code_editor_row_text_hits": summarize_times_us(&top_code_editor_row_text_hits),
            "top_code_editor_row_text_misses": summarize_times_us(&top_code_editor_row_text_misses),
            "top_code_editor_row_text_resets": summarize_times_us(&top_code_editor_row_text_resets),
            "top_code_editor_row_text_hit_rate_pct": summarize_times_us(&top_code_editor_row_text_hit_rate_pct),
            "top_code_editor_row_text_us": summarize_times_us(&top_code_editor_row_text_us),
            "top_code_editor_rows_painted": summarize_times_us(&top_code_editor_rows_painted),
            "top_code_editor_rows_scene_replayed": summarize_times_us(&top_code_editor_rows_scene_replayed),
            "top_code_editor_rows_scene_stored": summarize_times_us(&top_code_editor_rows_scene_stored),
            "top_code_editor_row_scene_ops_stored": summarize_times_us(&top_code_editor_row_scene_ops_stored),
            "top_code_editor_row_scene_replay_hit_rate_pct": summarize_times_us(&top_code_editor_row_scene_replay_hit_rate_pct),
            "top_code_editor_row_scene_cache_get_calls": summarize_times_us(&top_code_editor_row_scene_cache_get_calls),
            "top_code_editor_row_scene_cache_hits": summarize_times_us(&top_code_editor_row_scene_cache_hits),
            "top_code_editor_row_scene_cache_misses": summarize_times_us(&top_code_editor_row_scene_cache_misses),
            "top_code_editor_row_scene_cache_resets": summarize_times_us(&top_code_editor_row_scene_cache_resets),
            "top_code_editor_row_scene_cache_hit_rate_pct": summarize_times_us(&top_code_editor_row_scene_cache_hit_rate_pct),
            "top_code_editor_row_scene_prepaint_plan_us": summarize_times_us(&top_code_editor_row_scene_prepaint_plan_us),
            "top_code_editor_row_scene_prepaint_probe_us": summarize_times_us(&top_code_editor_row_scene_prepaint_probe_us),
            "top_code_editor_row_scene_prepaint_key_compare_us": summarize_times_us(&top_code_editor_row_scene_prepaint_key_compare_us),
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
            "top_renderer_uniform_bytes": summarize_times_us(&top_renderer_uniform_bytes),
            "top_renderer_instance_bytes": summarize_times_us(&top_renderer_instance_bytes),
            "top_renderer_vertex_bytes": summarize_times_us(&top_renderer_vertex_bytes),
            "top_renderer_geometry_upload_quad_instance_bytes": summarize_times_us(&top_renderer_geometry_upload_quad_instance_bytes),
            "top_renderer_geometry_upload_quad_instance_write_count": summarize_times_us(&top_renderer_geometry_upload_quad_instance_write_count),
            "top_renderer_geometry_upload_path_paint_bytes": summarize_times_us(&top_renderer_geometry_upload_path_paint_bytes),
            "top_renderer_geometry_upload_path_paint_write_count": summarize_times_us(&top_renderer_geometry_upload_path_paint_write_count),
            "top_renderer_geometry_upload_text_paint_bytes": summarize_times_us(&top_renderer_geometry_upload_text_paint_bytes),
            "top_renderer_geometry_upload_text_paint_write_count": summarize_times_us(&top_renderer_geometry_upload_text_paint_write_count),
            "top_renderer_geometry_upload_viewport_vertex_bytes": summarize_times_us(&top_renderer_geometry_upload_viewport_vertex_bytes),
            "top_renderer_geometry_upload_viewport_vertex_write_count": summarize_times_us(&top_renderer_geometry_upload_viewport_vertex_write_count),
            "top_renderer_geometry_upload_text_glyph_instance_bytes": summarize_times_us(&top_renderer_geometry_upload_text_glyph_instance_bytes),
            "top_renderer_geometry_upload_text_glyph_instance_write_count": summarize_times_us(&top_renderer_geometry_upload_text_glyph_instance_write_count),
            "top_renderer_geometry_upload_text_vertex_bytes": summarize_times_us(&top_renderer_geometry_upload_text_vertex_bytes),
            "top_renderer_geometry_upload_text_vertex_write_count": summarize_times_us(&top_renderer_geometry_upload_text_vertex_write_count),
            "top_renderer_geometry_upload_path_vertex_bytes": summarize_times_us(&top_renderer_geometry_upload_path_vertex_bytes),
            "top_renderer_geometry_upload_path_vertex_write_count": summarize_times_us(&top_renderer_geometry_upload_path_vertex_write_count),
            "top_renderer_scene_encoding_cache_miss_cold_start": summarize_times_us(&top_renderer_scene_encoding_cache_miss_cold_start),
            "top_renderer_scene_encoding_cache_miss_format_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_format_changed),
            "top_renderer_scene_encoding_cache_miss_viewport_size_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_viewport_size_changed),
            "top_renderer_scene_encoding_cache_miss_scale_factor_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_scale_factor_changed),
            "top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed),
            "top_renderer_scene_encoding_cache_miss_scene_ops_len_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_scene_ops_len_changed),
            "top_renderer_scene_encoding_cache_miss_render_targets_generation_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_render_targets_generation_changed),
            "top_renderer_scene_encoding_cache_miss_images_generation_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_images_generation_changed),
            "top_renderer_scene_encoding_cache_miss_text_atlas_revision_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_text_atlas_revision_changed),
            "top_renderer_scene_encoding_cache_miss_text_quality_key_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_text_quality_key_changed),
            "top_renderer_scene_encoding_cache_miss_materials_generation_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_materials_generation_changed),
            "top_renderer_scene_encoding_cache_miss_material_paint_budget_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_material_paint_budget_changed),
            "top_renderer_scene_encoding_cache_miss_material_distinct_budget_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_material_distinct_budget_changed),
            "top_renderer_scene_encoding_cache_miss_custom_effects_generation_changed": summarize_times_us(&top_renderer_scene_encoding_cache_miss_custom_effects_generation_changed),
            "top_renderer_encode_scene_text_ops": summarize_times_us(&top_renderer_encode_scene_text_ops),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perf_repeat_summary_json_row_summarizes_code_editor_row_scene_fields() {
        let mut rows = Vec::new();
        let runs = vec![
            serde_json::json!({
                "top_code_editor_row_text_get_calls": 20,
                "top_code_editor_row_text_hits": 16,
                "top_code_editor_row_text_misses": 4,
                "top_code_editor_row_text_resets": 1,
                "top_code_editor_row_text_hit_rate_pct": 80,
                "top_code_editor_row_text_us": 12,
                "top_code_editor_rows_painted": 10,
                "top_code_editor_rows_scene_replayed": 6,
                "top_code_editor_rows_scene_stored": 4,
                "top_code_editor_row_scene_ops_stored": 40,
                "top_code_editor_row_scene_replay_hit_rate_pct": 60,
                "top_code_editor_row_scene_cache_get_calls": 8,
                "top_code_editor_row_scene_cache_hits": 6,
                "top_code_editor_row_scene_cache_misses": 2,
                "top_code_editor_row_scene_cache_resets": 0,
                "top_code_editor_row_scene_cache_hit_rate_pct": 75,
                "top_code_editor_row_scene_prepaint_plan_us": 30,
                "top_code_editor_row_scene_prepaint_probe_us": 20,
                "top_code_editor_row_scene_prepaint_key_compare_us": 5,
                "top_renderer_geometry_upload_quad_instance_bytes": 128,
                "top_renderer_geometry_upload_quad_instance_write_count": 1,
                "top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed": 2
            }),
            serde_json::json!({
                "top_code_editor_row_text_get_calls": 30,
                "top_code_editor_row_text_hits": 27,
                "top_code_editor_row_text_misses": 3,
                "top_code_editor_row_text_resets": 0,
                "top_code_editor_row_text_hit_rate_pct": 90,
                "top_code_editor_row_text_us": 8,
                "top_code_editor_rows_painted": 20,
                "top_code_editor_rows_scene_replayed": 18,
                "top_code_editor_rows_scene_stored": 2,
                "top_code_editor_row_scene_ops_stored": 12,
                "top_code_editor_row_scene_replay_hit_rate_pct": 90,
                "top_code_editor_row_scene_cache_get_calls": 10,
                "top_code_editor_row_scene_cache_hits": 9,
                "top_code_editor_row_scene_cache_misses": 1,
                "top_code_editor_row_scene_cache_resets": 1,
                "top_code_editor_row_scene_cache_hit_rate_pct": 90,
                "top_code_editor_row_scene_prepaint_plan_us": 70,
                "top_code_editor_row_scene_prepaint_probe_us": 50,
                "top_code_editor_row_scene_prepaint_key_compare_us": 11,
                "top_renderer_geometry_upload_quad_instance_bytes": 64,
                "top_renderer_geometry_upload_quad_instance_write_count": 2,
                "top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed": 4
            }),
        ];

        push_perf_json_repeat_summary_row(
            &mut rows,
            Path::new("tools/diag-scripts/editor.json"),
            BundleStatsSort::Time,
            runs.len(),
            &runs,
            &[100, 200],
            &[10, 20],
            &[0, 0],
            &[5, 6],
            &[80, 160],
            &[1, 2],
            &[0, 1],
            &[0, 0],
            &[0, 0],
            &[0, 0],
            None,
        );

        let stats = &rows[0]["stats"];
        assert_eq!(stats["top_code_editor_row_text_get_calls"]["max"], 30);
        assert_eq!(stats["top_code_editor_row_text_hits"]["p50"], 16);
        assert_eq!(stats["top_code_editor_row_text_misses"]["max"], 4);
        assert_eq!(stats["top_code_editor_row_text_resets"]["max"], 1);
        assert_eq!(stats["top_code_editor_row_text_hit_rate_pct"]["p95"], 90);
        assert_eq!(stats["top_code_editor_row_text_us"]["p50"], 8);
        assert_eq!(stats["top_code_editor_rows_painted"]["max"], 20);
        assert_eq!(stats["top_code_editor_rows_scene_replayed"]["p50"], 6);
        assert_eq!(stats["top_code_editor_rows_scene_stored"]["p95"], 4);
        assert_eq!(stats["top_code_editor_row_scene_ops_stored"]["max"], 40);
        assert_eq!(
            stats["top_code_editor_row_scene_replay_hit_rate_pct"]["p95"],
            90
        );
        assert_eq!(
            stats["top_code_editor_row_scene_cache_get_calls"]["max"],
            10
        );
        assert_eq!(stats["top_code_editor_row_scene_cache_hits"]["p50"], 6);
        assert_eq!(stats["top_code_editor_row_scene_cache_misses"]["max"], 2);
        assert_eq!(stats["top_code_editor_row_scene_cache_resets"]["p95"], 1);
        assert_eq!(
            stats["top_code_editor_row_scene_cache_hit_rate_pct"]["p95"],
            90
        );
        assert_eq!(
            stats["top_code_editor_row_scene_prepaint_plan_us"]["p95"],
            70
        );
        assert_eq!(
            stats["top_code_editor_row_scene_prepaint_probe_us"]["p50"],
            20
        );
        assert_eq!(
            stats["top_code_editor_row_scene_prepaint_key_compare_us"]["max"],
            11
        );
        assert_eq!(
            stats["top_renderer_geometry_upload_quad_instance_bytes"]["max"],
            128
        );
        assert_eq!(
            stats["top_renderer_geometry_upload_quad_instance_write_count"]["p95"],
            2
        );
        assert_eq!(
            stats["top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed"]["max"],
            4
        );
    }
}
