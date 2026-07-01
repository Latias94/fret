use super::*;

pub(crate) fn push_perf_json_row(
    rows: &mut Vec<serde_json::Value>,
    script_key: &str,
    sort: BundleStatsSort,
    report: &BundleStatsReport,
    bundle_path: &Path,
) {
    let top = report.top.first();
    let top_total = top.map(|r| r.total_time_us).unwrap_or(0);
    let top_layout = top.map(|r| r.layout_time_us).unwrap_or(0);
    let top_solve = top.map(|r| r.layout_engine_solve_time_us).unwrap_or(0);
    let top_solves = top.map(|r| r.layout_engine_solves).unwrap_or(0);
    let top_prepaint = top.map(|r| r.prepaint_time_us).unwrap_or(0);
    let top_paint = top.map(|r| r.paint_time_us).unwrap_or(0);
    let top_dispatch = top.map(|r| r.dispatch_time_us).unwrap_or(0);
    let top_hit_test = top.map(|r| r.hit_test_time_us).unwrap_or(0);
    let top_dispatch_events = top.map(|r| r.dispatch_events).unwrap_or(0);
    let top_hit_test_queries = top.map(|r| r.hit_test_queries).unwrap_or(0);
    let top_code_editor = code_editor_rows::TopCodeEditorRowSceneFields::from_top(top);

    let pointer_move_frames_present = report.pointer_move_frames_present;
    let pointer_move_frames_considered = report.pointer_move_frames_considered as u64;
    let pointer_move_max_dispatch_time_us = report.pointer_move_max_dispatch_time_us;
    let pointer_move_max_hit_test_time_us = report.pointer_move_max_hit_test_time_us;
    let pointer_move_snapshots_with_global_changes =
        report.pointer_move_snapshots_with_global_changes as u64;

    let top_hit_test_bounds_tree_queries = top.map(|r| r.hit_test_bounds_tree_queries).unwrap_or(0);
    let top_hit_test_bounds_tree_disabled =
        top.map(|r| r.hit_test_bounds_tree_disabled).unwrap_or(0);
    let top_hit_test_bounds_tree_misses = top.map(|r| r.hit_test_bounds_tree_misses).unwrap_or(0);
    let top_hit_test_bounds_tree_hits = top.map(|r| r.hit_test_bounds_tree_hits).unwrap_or(0);
    let top_hit_test_bounds_tree_candidate_rejected = top
        .map(|r| r.hit_test_bounds_tree_candidate_rejected)
        .unwrap_or(0);

    let top_frame_arena_capacity_estimate_bytes = top
        .map(|r| r.frame_arena_capacity_estimate_bytes)
        .unwrap_or(0);
    let top_frame_arena_grow_events = top.map(|r| r.frame_arena_grow_events).unwrap_or(0);
    let top_element_children_vec_pool_reuses =
        top.map(|r| r.element_children_vec_pool_reuses).unwrap_or(0);
    let top_element_children_vec_pool_misses =
        top.map(|r| r.element_children_vec_pool_misses).unwrap_or(0);
    let top_element_children_vec_pool_grow_events = top
        .map(|r| r.element_children_vec_pool_grow_events)
        .unwrap_or(0);
    let top_tick = top.map(|r| r.tick_id).unwrap_or(0);
    let top_frame = top.map(|r| r.frame_id).unwrap_or(0);

    let top_view_cache_contained_relayouts =
        top.map(|r| r.view_cache_contained_relayouts).unwrap_or(0);
    let top_view_cache_roots_total = top.map(|r| r.view_cache_roots_total).unwrap_or(0);
    let top_view_cache_roots_reused = top.map(|r| r.view_cache_roots_reused).unwrap_or(0);
    let top_view_cache_roots_first_mount = top.map(|r| r.view_cache_roots_first_mount).unwrap_or(0);
    let top_view_cache_roots_node_recreated =
        top.map(|r| r.view_cache_roots_node_recreated).unwrap_or(0);
    let top_view_cache_roots_cache_key_mismatch = top
        .map(|r| r.view_cache_roots_cache_key_mismatch)
        .unwrap_or(0);
    let top_view_cache_roots_not_marked_reuse_root = top
        .map(|r| r.view_cache_roots_not_marked_reuse_root)
        .unwrap_or(0);
    let top_view_cache_roots_needs_rerender =
        top.map(|r| r.view_cache_roots_needs_rerender).unwrap_or(0);
    let top_view_cache_roots_layout_invalidated = top
        .map(|r| r.view_cache_roots_layout_invalidated)
        .unwrap_or(0);
    let top_view_cache_roots_manual = top.map(|r| r.view_cache_roots_manual).unwrap_or(0);

    let top_cache_roots_contained_relayout =
        top.map(|r| r.cache_roots_contained_relayout).unwrap_or(0);
    let top_set_children_barrier_writes = top.map(|r| r.set_children_barrier_writes).unwrap_or(0);
    let top_barrier_relayouts_scheduled = top.map(|r| r.barrier_relayouts_scheduled).unwrap_or(0);
    let top_barrier_relayouts_performed = top.map(|r| r.barrier_relayouts_performed).unwrap_or(0);
    let top_virtual_list_visible_range_checks = top
        .map(|r| r.virtual_list_visible_range_checks)
        .unwrap_or(0);
    let top_virtual_list_visible_range_refreshes = top
        .map(|r| r.virtual_list_visible_range_refreshes)
        .unwrap_or(0);

    let top_renderer_tick_id = top.map(|r| r.renderer_tick_id).unwrap_or(0);
    let top_renderer_frame_id = top.map(|r| r.renderer_frame_id).unwrap_or(0);
    let top_renderer_encode_scene_us = top.map(|r| r.renderer_encode_scene_us).unwrap_or(0);
    let top_renderer_prepare_text_us = top.map(|r| r.renderer_prepare_text_us).unwrap_or(0);
    let top_renderer_prepare_svg_us = top.map(|r| r.renderer_prepare_svg_us).unwrap_or(0);
    let top_renderer_draw_calls = top.map(|r| r.renderer_draw_calls).unwrap_or(0);
    let top_renderer_pipeline_switches = top.map(|r| r.renderer_pipeline_switches).unwrap_or(0);
    let top_renderer_bind_group_switches = top.map(|r| r.renderer_bind_group_switches).unwrap_or(0);
    let top_renderer_scissor_sets = top.map(|r| r.renderer_scissor_sets).unwrap_or(0);
    let top_renderer_scene_encoding_cache_misses = top
        .map(|r| r.renderer_scene_encoding_cache_misses)
        .unwrap_or(0);
    let top_renderer_material_quad_ops = top.map(|r| r.renderer_material_quad_ops).unwrap_or(0);
    let top_renderer_material_sampled_quad_ops = top
        .map(|r| r.renderer_material_sampled_quad_ops)
        .unwrap_or(0);
    let top_renderer_material_distinct = top.map(|r| r.renderer_material_distinct).unwrap_or(0);
    let top_renderer_material_unknown_ids =
        top.map(|r| r.renderer_material_unknown_ids).unwrap_or(0);
    let top_renderer_material_degraded_due_to_budget = top
        .map(|r| r.renderer_material_degraded_due_to_budget)
        .unwrap_or(0);
    let top_renderer_text_atlas_upload_bytes =
        top.map(|r| r.renderer_text_atlas_upload_bytes).unwrap_or(0);
    let top_renderer_text_atlas_evicted_pages = top
        .map(|r| r.renderer_text_atlas_evicted_pages)
        .unwrap_or(0);
    let top_renderer_svg_upload_bytes = top.map(|r| r.renderer_svg_upload_bytes).unwrap_or(0);
    let top_renderer_image_upload_bytes = top.map(|r| r.renderer_image_upload_bytes).unwrap_or(0);
    let top_renderer_uniform_bytes = top.map(|r| r.renderer_uniform_bytes).unwrap_or(0);
    let top_renderer_instance_bytes = top.map(|r| r.renderer_instance_bytes).unwrap_or(0);
    let top_renderer_vertex_bytes = top.map(|r| r.renderer_vertex_bytes).unwrap_or(0);
    let top_renderer_geometry_upload_quad_instance_bytes = top
        .map(|r| r.renderer_geometry_upload_quad_instance_bytes)
        .unwrap_or(0);
    let top_renderer_geometry_upload_quad_instance_write_count = top
        .map(|r| r.renderer_geometry_upload_quad_instance_write_count)
        .unwrap_or(0);
    let top_renderer_geometry_upload_path_paint_bytes = top
        .map(|r| r.renderer_geometry_upload_path_paint_bytes)
        .unwrap_or(0);
    let top_renderer_geometry_upload_path_paint_write_count = top
        .map(|r| r.renderer_geometry_upload_path_paint_write_count)
        .unwrap_or(0);
    let top_renderer_geometry_upload_text_paint_bytes = top
        .map(|r| r.renderer_geometry_upload_text_paint_bytes)
        .unwrap_or(0);
    let top_renderer_geometry_upload_text_paint_write_count = top
        .map(|r| r.renderer_geometry_upload_text_paint_write_count)
        .unwrap_or(0);
    let top_renderer_geometry_upload_viewport_vertex_bytes = top
        .map(|r| r.renderer_geometry_upload_viewport_vertex_bytes)
        .unwrap_or(0);
    let top_renderer_geometry_upload_viewport_vertex_write_count = top
        .map(|r| r.renderer_geometry_upload_viewport_vertex_write_count)
        .unwrap_or(0);
    let top_renderer_geometry_upload_text_glyph_instance_bytes = top
        .map(|r| r.renderer_geometry_upload_text_glyph_instance_bytes)
        .unwrap_or(0);
    let top_renderer_geometry_upload_text_glyph_instance_write_count = top
        .map(|r| r.renderer_geometry_upload_text_glyph_instance_write_count)
        .unwrap_or(0);
    let top_renderer_geometry_upload_text_vertex_bytes = top
        .map(|r| r.renderer_geometry_upload_text_vertex_bytes)
        .unwrap_or(0);
    let top_renderer_geometry_upload_text_vertex_write_count = top
        .map(|r| r.renderer_geometry_upload_text_vertex_write_count)
        .unwrap_or(0);
    let top_renderer_geometry_upload_path_vertex_bytes = top
        .map(|r| r.renderer_geometry_upload_path_vertex_bytes)
        .unwrap_or(0);
    let top_renderer_geometry_upload_path_vertex_write_count = top
        .map(|r| r.renderer_geometry_upload_path_vertex_write_count)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_cold_start = top
        .map(|r| r.renderer_scene_encoding_cache_miss_cold_start)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_format_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_format_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_viewport_size_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_viewport_size_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_scale_factor_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_scale_factor_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_scene_fingerprint_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_scene_ops_len_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_scene_ops_len_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_render_targets_generation_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_render_targets_generation_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_images_generation_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_images_generation_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_text_atlas_revision_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_text_atlas_revision_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_text_quality_key_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_text_quality_key_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_materials_generation_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_materials_generation_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_material_paint_budget_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_material_paint_budget_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_material_distinct_budget_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_material_distinct_budget_changed)
        .unwrap_or(0);
    let top_renderer_scene_encoding_cache_miss_custom_effects_generation_changed = top
        .map(|r| r.renderer_scene_encoding_cache_miss_custom_effects_generation_changed)
        .unwrap_or(0);
    let top_renderer_encode_scene_text_ops =
        top.map(|r| r.renderer_encode_scene_text_ops).unwrap_or(0);
    let top_renderer_svg_raster_cache_misses =
        top.map(|r| r.renderer_svg_raster_cache_misses).unwrap_or(0);
    let top_renderer_svg_raster_budget_evictions = top
        .map(|r| r.renderer_svg_raster_budget_evictions)
        .unwrap_or(0);
    let top_renderer_svg_raster_budget_bytes =
        top.map(|r| r.renderer_svg_raster_budget_bytes).unwrap_or(0);
    let top_renderer_svg_rasters_live = top.map(|r| r.renderer_svg_rasters_live).unwrap_or(0);
    let top_renderer_svg_standalone_bytes_live = top
        .map(|r| r.renderer_svg_standalone_bytes_live)
        .unwrap_or(0);
    let top_renderer_svg_mask_atlas_pages_live = top
        .map(|r| r.renderer_svg_mask_atlas_pages_live)
        .unwrap_or(0);
    let top_renderer_svg_mask_atlas_bytes_live = top
        .map(|r| r.renderer_svg_mask_atlas_bytes_live)
        .unwrap_or(0);
    let top_renderer_svg_mask_atlas_used_px =
        top.map(|r| r.renderer_svg_mask_atlas_used_px).unwrap_or(0);
    let top_renderer_svg_mask_atlas_capacity_px = top
        .map(|r| r.renderer_svg_mask_atlas_capacity_px)
        .unwrap_or(0);
    let top_renderer_svg_raster_cache_hits =
        top.map(|r| r.renderer_svg_raster_cache_hits).unwrap_or(0);
    let top_renderer_svg_mask_atlas_page_evictions = top
        .map(|r| r.renderer_svg_mask_atlas_page_evictions)
        .unwrap_or(0);
    let top_renderer_svg_mask_atlas_entries_evicted = top
        .map(|r| r.renderer_svg_mask_atlas_entries_evicted)
        .unwrap_or(0);
    let top_renderer_intermediate_budget_bytes = top
        .map(|r| r.renderer_intermediate_budget_bytes)
        .unwrap_or(0);
    let top_renderer_intermediate_in_use_bytes = top
        .map(|r| r.renderer_intermediate_in_use_bytes)
        .unwrap_or(0);
    let top_renderer_intermediate_peak_in_use_bytes = top
        .map(|r| r.renderer_intermediate_peak_in_use_bytes)
        .unwrap_or(0);
    let top_renderer_intermediate_release_targets = top
        .map(|r| r.renderer_intermediate_release_targets)
        .unwrap_or(0);
    let top_renderer_intermediate_pool_allocations = top
        .map(|r| r.renderer_intermediate_pool_allocations)
        .unwrap_or(0);
    let top_renderer_intermediate_pool_reuses = top
        .map(|r| r.renderer_intermediate_pool_reuses)
        .unwrap_or(0);
    let top_renderer_intermediate_pool_releases = top
        .map(|r| r.renderer_intermediate_pool_releases)
        .unwrap_or(0);
    let top_renderer_intermediate_pool_evictions = top
        .map(|r| r.renderer_intermediate_pool_evictions)
        .unwrap_or(0);
    let top_renderer_intermediate_pool_free_bytes = top
        .map(|r| r.renderer_intermediate_pool_free_bytes)
        .unwrap_or(0);
    let top_renderer_intermediate_pool_free_textures = top
        .map(|r| r.renderer_intermediate_pool_free_textures)
        .unwrap_or(0);
    rows.push(serde_json::json!({
        "script": script_key.to_string(),
        "sort": sort.as_str(),
        "top_total_time_us": top_total,
        "top_layout_time_us": top_layout,
        "top_layout_engine_solve_time_us": top_solve,
        "top_layout_engine_solves": top_solves,
        "top_prepaint_time_us": top_prepaint,
        "top_paint_time_us": top_paint,
        "top_dispatch_time_us": top_dispatch,
        "top_hit_test_time_us": top_hit_test,
        "top_dispatch_events": top_dispatch_events,
        "top_hit_test_queries": top_hit_test_queries,
        "top_code_editor_row_text_get_calls": top_code_editor.row_text_get_calls,
        "top_code_editor_row_text_hits": top_code_editor.row_text_hits,
        "top_code_editor_row_text_misses": top_code_editor.row_text_misses,
        "top_code_editor_row_text_resets": top_code_editor.row_text_resets,
        "top_code_editor_row_text_hit_rate_pct": top_code_editor.row_text_hit_rate_pct,
        "top_code_editor_row_text_us": top_code_editor.row_text_us,
        "top_code_editor_rows_painted": top_code_editor.rows_painted,
        "top_code_editor_rows_scene_replayed": top_code_editor.rows_scene_replayed,
        "top_code_editor_rows_scene_stored": top_code_editor.rows_scene_stored,
        "top_code_editor_row_scene_ops_stored": top_code_editor.row_scene_ops_stored,
        "top_code_editor_row_scene_replay_hit_rate_pct": top_code_editor.row_scene_replay_hit_rate_pct,
        "top_code_editor_row_scene_cache_get_calls": top_code_editor.row_scene_cache_get_calls,
        "top_code_editor_row_scene_cache_hits": top_code_editor.row_scene_cache_hits,
        "top_code_editor_row_scene_cache_misses": top_code_editor.row_scene_cache_misses,
        "top_code_editor_row_scene_cache_resets": top_code_editor.row_scene_cache_resets,
        "top_code_editor_row_scene_cache_hit_rate_pct": top_code_editor.row_scene_cache_hit_rate_pct,
        "top_code_editor_row_scene_prepaint_plan_us": top_code_editor.row_scene_prepaint_plan_us,
        "top_code_editor_row_scene_prepaint_probe_us": top_code_editor.row_scene_prepaint_probe_us,
        "top_code_editor_row_scene_prepaint_key_compare_us": top_code_editor.row_scene_prepaint_key_compare_us,
        "top_code_editor_windowed_surface_paint_callback_us": top_code_editor.windowed_surface_paint_callback_us,
        "top_code_editor_windowed_surface_non_row_us": top_code_editor.windowed_surface_non_row_us,
        "top_code_editor_windowed_surface_row_callback_gap_us": top_code_editor.windowed_surface_row_callback_gap_us,
        "top_code_editor_torture_autoscroll_us": top_code_editor.torture_autoscroll_us,
        "top_code_editor_torture_overlay_us": top_code_editor.torture_overlay_us,
        "pointer_move_frames_present": pointer_move_frames_present,
        "pointer_move_frames_considered": pointer_move_frames_considered,
        "pointer_move_max_dispatch_time_us": pointer_move_max_dispatch_time_us,
        "pointer_move_max_hit_test_time_us": pointer_move_max_hit_test_time_us,
        "pointer_move_snapshots_with_global_changes": pointer_move_snapshots_with_global_changes,
        "top_hit_test_bounds_tree_queries": top_hit_test_bounds_tree_queries,
        "top_hit_test_bounds_tree_disabled": top_hit_test_bounds_tree_disabled,
        "top_hit_test_bounds_tree_misses": top_hit_test_bounds_tree_misses,
        "top_hit_test_bounds_tree_hits": top_hit_test_bounds_tree_hits,
        "top_hit_test_bounds_tree_candidate_rejected": top_hit_test_bounds_tree_candidate_rejected,
        "top_frame_arena_capacity_estimate_bytes": top_frame_arena_capacity_estimate_bytes,
        "top_frame_arena_grow_events": top_frame_arena_grow_events,
        "top_element_children_vec_pool_reuses": top_element_children_vec_pool_reuses,
        "top_element_children_vec_pool_misses": top_element_children_vec_pool_misses,
        "top_element_children_vec_pool_grow_events": top_element_children_vec_pool_grow_events,
        "top_tick_id": top_tick,
        "top_frame_id": top_frame,
        "top_view_cache_contained_relayouts": top_view_cache_contained_relayouts,
        "top_view_cache_roots_total": top_view_cache_roots_total,
        "top_view_cache_roots_reused": top_view_cache_roots_reused,
        "top_view_cache_roots_first_mount": top_view_cache_roots_first_mount,
        "top_view_cache_roots_node_recreated": top_view_cache_roots_node_recreated,
        "top_view_cache_roots_cache_key_mismatch": top_view_cache_roots_cache_key_mismatch,
        "top_view_cache_roots_not_marked_reuse_root": top_view_cache_roots_not_marked_reuse_root,
        "top_view_cache_roots_needs_rerender": top_view_cache_roots_needs_rerender,
        "top_view_cache_roots_layout_invalidated": top_view_cache_roots_layout_invalidated,
        "top_view_cache_roots_manual": top_view_cache_roots_manual,
        "top_cache_roots_contained_relayout": top_cache_roots_contained_relayout,
        "top_set_children_barrier_writes": top_set_children_barrier_writes,
        "top_barrier_relayouts_scheduled": top_barrier_relayouts_scheduled,
        "top_barrier_relayouts_performed": top_barrier_relayouts_performed,
        "top_virtual_list_visible_range_checks": top_virtual_list_visible_range_checks,
        "top_virtual_list_visible_range_refreshes": top_virtual_list_visible_range_refreshes,
        "top_renderer_tick_id": top_renderer_tick_id,
        "top_renderer_frame_id": top_renderer_frame_id,
        "top_renderer_encode_scene_us": top_renderer_encode_scene_us,
        "top_renderer_prepare_text_us": top_renderer_prepare_text_us,
        "top_renderer_prepare_svg_us": top_renderer_prepare_svg_us,
        "top_renderer_draw_calls": top_renderer_draw_calls,
        "top_renderer_pipeline_switches": top_renderer_pipeline_switches,
        "top_renderer_bind_group_switches": top_renderer_bind_group_switches,
        "top_renderer_scissor_sets": top_renderer_scissor_sets,
        "top_renderer_scene_encoding_cache_misses": top_renderer_scene_encoding_cache_misses,
        "top_renderer_material_quad_ops": top_renderer_material_quad_ops,
        "top_renderer_material_sampled_quad_ops": top_renderer_material_sampled_quad_ops,
        "top_renderer_material_distinct": top_renderer_material_distinct,
        "top_renderer_material_unknown_ids": top_renderer_material_unknown_ids,
        "top_renderer_material_degraded_due_to_budget": top_renderer_material_degraded_due_to_budget,
        "top_renderer_text_atlas_upload_bytes": top_renderer_text_atlas_upload_bytes,
        "top_renderer_text_atlas_evicted_pages": top_renderer_text_atlas_evicted_pages,
        "top_renderer_svg_upload_bytes": top_renderer_svg_upload_bytes,
        "top_renderer_image_upload_bytes": top_renderer_image_upload_bytes,
        "top_renderer_uniform_bytes": top_renderer_uniform_bytes,
        "top_renderer_instance_bytes": top_renderer_instance_bytes,
        "top_renderer_vertex_bytes": top_renderer_vertex_bytes,
        "top_renderer_geometry_upload_quad_instance_bytes": top_renderer_geometry_upload_quad_instance_bytes,
        "top_renderer_geometry_upload_quad_instance_write_count": top_renderer_geometry_upload_quad_instance_write_count,
        "top_renderer_geometry_upload_path_paint_bytes": top_renderer_geometry_upload_path_paint_bytes,
        "top_renderer_geometry_upload_path_paint_write_count": top_renderer_geometry_upload_path_paint_write_count,
        "top_renderer_geometry_upload_text_paint_bytes": top_renderer_geometry_upload_text_paint_bytes,
        "top_renderer_geometry_upload_text_paint_write_count": top_renderer_geometry_upload_text_paint_write_count,
        "top_renderer_geometry_upload_viewport_vertex_bytes": top_renderer_geometry_upload_viewport_vertex_bytes,
        "top_renderer_geometry_upload_viewport_vertex_write_count": top_renderer_geometry_upload_viewport_vertex_write_count,
        "top_renderer_geometry_upload_text_glyph_instance_bytes": top_renderer_geometry_upload_text_glyph_instance_bytes,
        "top_renderer_geometry_upload_text_glyph_instance_write_count": top_renderer_geometry_upload_text_glyph_instance_write_count,
        "top_renderer_geometry_upload_text_vertex_bytes": top_renderer_geometry_upload_text_vertex_bytes,
        "top_renderer_geometry_upload_text_vertex_write_count": top_renderer_geometry_upload_text_vertex_write_count,
        "top_renderer_geometry_upload_path_vertex_bytes": top_renderer_geometry_upload_path_vertex_bytes,
        "top_renderer_geometry_upload_path_vertex_write_count": top_renderer_geometry_upload_path_vertex_write_count,
        "top_renderer_scene_encoding_cache_miss_cold_start": top_renderer_scene_encoding_cache_miss_cold_start,
        "top_renderer_scene_encoding_cache_miss_format_changed": top_renderer_scene_encoding_cache_miss_format_changed,
        "top_renderer_scene_encoding_cache_miss_viewport_size_changed": top_renderer_scene_encoding_cache_miss_viewport_size_changed,
        "top_renderer_scene_encoding_cache_miss_scale_factor_changed": top_renderer_scene_encoding_cache_miss_scale_factor_changed,
        "top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed": top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed,
        "top_renderer_scene_encoding_cache_miss_scene_ops_len_changed": top_renderer_scene_encoding_cache_miss_scene_ops_len_changed,
        "top_renderer_scene_encoding_cache_miss_render_targets_generation_changed": top_renderer_scene_encoding_cache_miss_render_targets_generation_changed,
        "top_renderer_scene_encoding_cache_miss_images_generation_changed": top_renderer_scene_encoding_cache_miss_images_generation_changed,
        "top_renderer_scene_encoding_cache_miss_text_atlas_revision_changed": top_renderer_scene_encoding_cache_miss_text_atlas_revision_changed,
        "top_renderer_scene_encoding_cache_miss_text_quality_key_changed": top_renderer_scene_encoding_cache_miss_text_quality_key_changed,
        "top_renderer_scene_encoding_cache_miss_materials_generation_changed": top_renderer_scene_encoding_cache_miss_materials_generation_changed,
        "top_renderer_scene_encoding_cache_miss_material_paint_budget_changed": top_renderer_scene_encoding_cache_miss_material_paint_budget_changed,
        "top_renderer_scene_encoding_cache_miss_material_distinct_budget_changed": top_renderer_scene_encoding_cache_miss_material_distinct_budget_changed,
        "top_renderer_scene_encoding_cache_miss_custom_effects_generation_changed": top_renderer_scene_encoding_cache_miss_custom_effects_generation_changed,
        "top_renderer_encode_scene_text_ops": top_renderer_encode_scene_text_ops,
        "top_renderer_svg_raster_cache_misses": top_renderer_svg_raster_cache_misses,
        "top_renderer_svg_raster_budget_evictions": top_renderer_svg_raster_budget_evictions,
        "top_renderer_svg_raster_budget_bytes": top_renderer_svg_raster_budget_bytes,
        "top_renderer_svg_rasters_live": top_renderer_svg_rasters_live,
        "top_renderer_svg_standalone_bytes_live": top_renderer_svg_standalone_bytes_live,
        "top_renderer_svg_mask_atlas_pages_live": top_renderer_svg_mask_atlas_pages_live,
        "top_renderer_svg_mask_atlas_bytes_live": top_renderer_svg_mask_atlas_bytes_live,
        "top_renderer_svg_mask_atlas_used_px": top_renderer_svg_mask_atlas_used_px,
        "top_renderer_svg_mask_atlas_capacity_px": top_renderer_svg_mask_atlas_capacity_px,
        "top_renderer_svg_raster_cache_hits": top_renderer_svg_raster_cache_hits,
        "top_renderer_svg_mask_atlas_page_evictions": top_renderer_svg_mask_atlas_page_evictions,
        "top_renderer_svg_mask_atlas_entries_evicted": top_renderer_svg_mask_atlas_entries_evicted,
        "top_renderer_intermediate_budget_bytes": top_renderer_intermediate_budget_bytes,
        "top_renderer_intermediate_in_use_bytes": top_renderer_intermediate_in_use_bytes,
        "top_renderer_intermediate_peak_in_use_bytes": top_renderer_intermediate_peak_in_use_bytes,
        "top_renderer_intermediate_release_targets": top_renderer_intermediate_release_targets,
        "top_renderer_intermediate_pool_allocations": top_renderer_intermediate_pool_allocations,
        "top_renderer_intermediate_pool_reuses": top_renderer_intermediate_pool_reuses,
        "top_renderer_intermediate_pool_releases": top_renderer_intermediate_pool_releases,
        "top_renderer_intermediate_pool_evictions": top_renderer_intermediate_pool_evictions,
        "top_renderer_intermediate_pool_free_bytes": top_renderer_intermediate_pool_free_bytes,
        "top_renderer_intermediate_pool_free_textures": top_renderer_intermediate_pool_free_textures,
        "bundle": bundle_path.display().to_string(),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::{
        BundleStatsCodeEditorCacheStats, BundleStatsCodeEditorPaintPerf, BundleStatsSnapshotRow,
    };

    #[test]
    fn perf_json_row_exports_top_code_editor_row_scene_fields() {
        let mut rows = Vec::new();
        let mut report = BundleStatsReport::default();
        report.top.push(BundleStatsSnapshotRow {
            total_time_us: 100,
            renderer_geometry_upload_quad_instance_bytes: 128,
            renderer_geometry_upload_quad_instance_write_count: 1,
            renderer_geometry_upload_text_vertex_bytes: 256,
            renderer_geometry_upload_text_vertex_write_count: 2,
            renderer_scene_encoding_cache_miss_scene_fingerprint_changed: 3,
            renderer_scene_encoding_cache_miss_text_quality_key_changed: 1,
            code_editor_paint_perf: Some(BundleStatsCodeEditorPaintPerf {
                us_row_text: 15,
                rows_painted: 20,
                rows_scene_replayed: 15,
                rows_scene_stored: 5,
                row_scene_ops_stored: 64,
                us_row_scene_prepaint_plan: 33,
                us_row_scene_prepaint_probe: 21,
                us_row_scene_prepaint_key_compare: 13,
                us_windowed_surface_paint_callback: 200,
                us_windowed_surface_non_row: 40,
                us_windowed_surface_row_callback_gap: 9,
                us_torture_autoscroll: 5,
                us_torture_overlay: 23,
                ..Default::default()
            }),
            code_editor_cache_stats: Some(BundleStatsCodeEditorCacheStats {
                row_text_get_calls: 25,
                row_text_hits: 20,
                row_text_misses: 5,
                row_text_resets: 1,
                row_scene_get_calls: 16,
                row_scene_hits: 12,
                row_scene_misses: 4,
                row_scene_resets: 2,
                ..Default::default()
            }),
            ..Default::default()
        });

        push_perf_json_row(
            &mut rows,
            "tools/diag-scripts/editor.json",
            BundleStatsSort::Time,
            &report,
            Path::new("target/fret-diag/editor/bundle.schema2.json"),
        );

        assert_eq!(rows[0]["top_code_editor_row_text_get_calls"], 25);
        assert_eq!(rows[0]["top_code_editor_row_text_hits"], 20);
        assert_eq!(rows[0]["top_code_editor_row_text_misses"], 5);
        assert_eq!(rows[0]["top_code_editor_row_text_resets"], 1);
        assert_eq!(rows[0]["top_code_editor_row_text_hit_rate_pct"], 80);
        assert_eq!(rows[0]["top_code_editor_row_text_us"], 15);
        assert_eq!(rows[0]["top_code_editor_rows_painted"], 20);
        assert_eq!(rows[0]["top_code_editor_rows_scene_replayed"], 15);
        assert_eq!(rows[0]["top_code_editor_rows_scene_stored"], 5);
        assert_eq!(rows[0]["top_code_editor_row_scene_ops_stored"], 64);
        assert_eq!(rows[0]["top_code_editor_row_scene_replay_hit_rate_pct"], 75);
        assert_eq!(rows[0]["top_code_editor_row_scene_cache_get_calls"], 16);
        assert_eq!(rows[0]["top_code_editor_row_scene_cache_hits"], 12);
        assert_eq!(rows[0]["top_code_editor_row_scene_cache_misses"], 4);
        assert_eq!(rows[0]["top_code_editor_row_scene_cache_resets"], 2);
        assert_eq!(rows[0]["top_code_editor_row_scene_cache_hit_rate_pct"], 75);
        assert_eq!(rows[0]["top_code_editor_row_scene_prepaint_plan_us"], 33);
        assert_eq!(rows[0]["top_code_editor_row_scene_prepaint_probe_us"], 21);
        assert_eq!(
            rows[0]["top_code_editor_row_scene_prepaint_key_compare_us"],
            13
        );
        assert_eq!(
            rows[0]["top_code_editor_windowed_surface_paint_callback_us"],
            200
        );
        assert_eq!(rows[0]["top_code_editor_windowed_surface_non_row_us"], 40);
        assert_eq!(
            rows[0]["top_code_editor_windowed_surface_row_callback_gap_us"],
            9
        );
        assert_eq!(rows[0]["top_code_editor_torture_autoscroll_us"], 5);
        assert_eq!(rows[0]["top_code_editor_torture_overlay_us"], 23);
        assert_eq!(
            rows[0]["top_renderer_geometry_upload_quad_instance_bytes"],
            128
        );
        assert_eq!(
            rows[0]["top_renderer_geometry_upload_quad_instance_write_count"],
            1
        );
        assert_eq!(
            rows[0]["top_renderer_geometry_upload_text_vertex_bytes"],
            256
        );
        assert_eq!(
            rows[0]["top_renderer_geometry_upload_text_vertex_write_count"],
            2
        );
        assert_eq!(
            rows[0]["top_renderer_scene_encoding_cache_miss_scene_fingerprint_changed"],
            3
        );
        assert_eq!(
            rows[0]["top_renderer_scene_encoding_cache_miss_text_quality_key_changed"],
            1
        );
    }
}
