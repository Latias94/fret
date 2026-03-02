
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiFrameStatsV1 {
    #[serde(default)]
    pub frame_arena_capacity_estimate_bytes: u64,
    #[serde(default)]
    pub frame_arena_grow_events: u32,
    #[serde(default)]
    pub element_children_vec_pool_reuses: u32,
    #[serde(default)]
    pub element_children_vec_pool_misses: u32,
    /// UI thread CPU time spent since the previous snapshot (approx frame CPU time).
    ///
    /// This is intended to distinguish "real work" from schedule noise: if wall time spikes but
    /// CPU time does not, the thread likely wasn't running (preempted / ReadyThread / DPC/ISR).
    #[serde(default)]
    pub ui_thread_cpu_time_us: u64,
    /// Cumulative UI thread CPU time since process start (kernel + user).
    #[serde(default)]
    pub ui_thread_cpu_total_time_us: u64,
    /// UI thread CPU cycle time since the previous snapshot (high-resolution signal on Windows).
    ///
    /// Prefer this over `ui_thread_cpu_time_us` when doing per-frame triage: `GetThreadTimes`
    /// resolution can be too coarse on some systems.
    #[serde(default)]
    pub ui_thread_cpu_cycle_time_delta_cycles: u64,
    /// Cumulative UI thread CPU cycle time since process start.
    #[serde(default)]
    pub ui_thread_cpu_cycle_time_total_cycles: u64,
    pub layout_time_us: u64,
    #[serde(default)]
    pub layout_collect_roots_time_us: u64,
    #[serde(default)]
    pub layout_invalidate_scroll_handle_bindings_time_us: u64,
    #[serde(default)]
    pub layout_expand_view_cache_invalidations_time_us: u64,
    #[serde(default)]
    pub layout_request_build_roots_time_us: u64,
    #[serde(default)]
    pub layout_pending_barrier_relayouts_time_us: u64,
    #[serde(default)]
    pub layout_repair_view_cache_bounds_time_us: u64,
    #[serde(default)]
    pub layout_contained_view_cache_roots_time_us: u64,
    #[serde(default)]
    pub layout_collapse_layout_observations_time_us: u64,
    #[serde(default)]
    pub layout_observation_record_time_us: u64,
    #[serde(default)]
    pub layout_observation_record_models_items: u32,
    #[serde(default)]
    pub layout_observation_record_globals_items: u32,
    #[serde(default)]
    pub layout_prepaint_after_layout_time_us: u64,
    #[serde(default)]
    pub layout_skipped_engine_frame: bool,
    #[serde(default)]
    pub layout_roots_time_us: u64,
    #[serde(default)]
    pub layout_barrier_relayouts_time_us: u64,
    #[serde(default)]
    pub layout_view_cache_time_us: u64,
    #[serde(default)]
    pub layout_semantics_refresh_time_us: u64,
    #[serde(default)]
    pub layout_focus_repair_time_us: u64,
    #[serde(default)]
    pub layout_deferred_cleanup_time_us: u64,
    #[serde(default)]
    pub prepaint_time_us: u64,
    pub paint_time_us: u64,
    #[serde(default)]
    pub paint_record_visual_bounds_time_us: u64,
    #[serde(default)]
    pub paint_record_visual_bounds_calls: u32,
    #[serde(default)]
    pub paint_cache_key_time_us: u64,
    #[serde(default)]
    pub paint_cache_hit_check_time_us: u64,
    #[serde(default)]
    pub paint_widget_time_us: u64,
    #[serde(default)]
    pub paint_observation_record_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_observed_models_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_observed_models_items: u32,
    #[serde(default)]
    pub paint_host_widget_observed_globals_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_observed_globals_items: u32,
    #[serde(default)]
    pub paint_host_widget_instance_lookup_time_us: u64,
    #[serde(default)]
    pub paint_host_widget_instance_lookup_calls: u32,
    #[serde(default)]
    pub paint_text_prepare_time_us: u64,
    #[serde(default)]
    pub paint_text_prepare_calls: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_blob_missing: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_scale_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_text_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_rich_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_style_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_wrap_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_overflow_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_width_changed: u32,
    #[serde(default)]
    pub paint_text_prepare_reason_font_stack_changed: u32,
    #[serde(default)]
    pub paint_input_context_time_us: u64,
    #[serde(default)]
    pub paint_scroll_handle_invalidation_time_us: u64,
    #[serde(default)]
    pub paint_collect_roots_time_us: u64,
    #[serde(default)]
    pub paint_publish_text_input_snapshot_time_us: u64,
    #[serde(default)]
    pub paint_collapse_observations_time_us: u64,
    #[serde(default)]
    pub dispatch_time_us: u64,
    #[serde(default)]
    pub dispatch_pointer_events: u32,
    #[serde(default)]
    pub dispatch_pointer_event_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_events: u32,
    #[serde(default)]
    pub dispatch_timer_event_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_targeted_events: u32,
    #[serde(default)]
    pub dispatch_timer_targeted_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_broadcast_events: u32,
    #[serde(default)]
    pub dispatch_timer_broadcast_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_broadcast_layers_visited: u32,
    #[serde(default)]
    pub dispatch_timer_broadcast_rebuild_visible_layers_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_broadcast_loop_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_slowest_event_time_us: u64,
    #[serde(default)]
    pub dispatch_timer_slowest_token: Option<u64>,
    #[serde(default)]
    pub dispatch_timer_slowest_was_broadcast: bool,
    #[serde(default)]
    pub dispatch_other_events: u32,
    #[serde(default)]
    pub dispatch_other_event_time_us: u64,
    #[serde(default)]
    pub hit_test_time_us: u64,
    #[serde(default)]
    pub dispatch_events: u32,
    #[serde(default)]
    pub hit_test_queries: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_queries: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_disabled: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_misses: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_hits: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_candidate_rejected: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_nodes_visited: u32,
    #[serde(default)]
    pub hit_test_bounds_tree_nodes_pushed: u32,
    #[serde(default)]
    pub hit_test_path_cache_hits: u32,
    #[serde(default)]
    pub hit_test_path_cache_misses: u32,
    #[serde(default)]
    pub hit_test_cached_path_time_us: u64,
    #[serde(default)]
    pub hit_test_bounds_tree_query_time_us: u64,
    #[serde(default)]
    pub hit_test_candidate_self_only_time_us: u64,
    #[serde(default)]
    pub hit_test_fallback_traversal_time_us: u64,
    #[serde(default)]
    pub dispatch_hover_update_time_us: u64,
    #[serde(default)]
    pub dispatch_scroll_handle_invalidation_time_us: u64,
    #[serde(default)]
    pub dispatch_active_layers_time_us: u64,
    #[serde(default)]
    pub dispatch_input_context_time_us: u64,
    #[serde(default)]
    pub dispatch_event_chain_build_time_us: u64,
    #[serde(default)]
    pub dispatch_widget_capture_time_us: u64,
    #[serde(default)]
    pub dispatch_widget_bubble_time_us: u64,
    #[serde(default)]
    pub dispatch_cursor_query_time_us: u64,
    #[serde(default)]
    pub dispatch_pointer_move_layer_observers_time_us: u64,
    #[serde(default)]
    pub dispatch_synth_hover_observer_time_us: u64,
    #[serde(default)]
    pub dispatch_cursor_effect_time_us: u64,
    #[serde(default)]
    pub dispatch_post_dispatch_snapshot_time_us: u64,
    pub layout_nodes_visited: u32,
    pub layout_nodes_performed: u32,
    #[serde(default)]
    pub prepaint_nodes_visited: u32,
    pub paint_nodes: u32,
    pub paint_nodes_performed: u32,
    pub paint_cache_hits: u32,
    pub paint_cache_misses: u32,
    pub paint_cache_replayed_ops: u32,
    #[serde(default)]
    pub paint_cache_replay_time_us: u64,
    #[serde(default)]
    pub paint_cache_bounds_translate_time_us: u64,
    #[serde(default)]
    pub paint_cache_bounds_translated_nodes: u32,
    #[serde(default)]
    pub interaction_cache_hits: u32,
    #[serde(default)]
    pub interaction_cache_misses: u32,
    #[serde(default)]
    pub interaction_cache_replayed_records: u32,
    #[serde(default)]
    pub interaction_records: u32,
    pub layout_engine_solves: u64,
    pub layout_engine_solve_time_us: u64,
    pub layout_engine_widget_fallback_solves: u64,
    #[serde(default)]
    pub layout_fast_path_taken: bool,
    #[serde(default)]
    pub layout_invalidations_count: u32,
    #[serde(default)]
    pub model_change_invalidation_roots: u32,
    #[serde(default)]
    pub model_change_models: u32,
    #[serde(default)]
    pub model_change_observation_edges: u32,
    #[serde(default)]
    pub model_change_unobserved_models: u32,
    #[serde(default)]
    pub global_change_invalidation_roots: u32,
    #[serde(default)]
    pub global_change_globals: u32,
    #[serde(default)]
    pub global_change_observation_edges: u32,
    #[serde(default)]
    pub global_change_unobserved_globals: u32,
    #[serde(default)]
    pub invalidation_walk_nodes: u32,
    #[serde(default)]
    pub invalidation_walk_calls: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_model_change: u32,
    #[serde(default)]
    pub invalidation_walk_calls_model_change: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_global_change: u32,
    #[serde(default)]
    pub invalidation_walk_calls_global_change: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_hover: u32,
    #[serde(default)]
    pub invalidation_walk_calls_hover: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_focus: u32,
    #[serde(default)]
    pub invalidation_walk_calls_focus: u32,
    #[serde(default)]
    pub invalidation_walk_nodes_other: u32,
    #[serde(default)]
    pub invalidation_walk_calls_other: u32,
    #[serde(default)]
    pub hover_pressable_target_changes: u32,
    #[serde(default)]
    pub hover_hover_region_target_changes: u32,
    #[serde(default)]
    pub hover_declarative_instance_changes: u32,
    #[serde(default)]
    pub hover_declarative_hit_test_invalidations: u32,
    #[serde(default)]
    pub hover_declarative_layout_invalidations: u32,
    #[serde(default)]
    pub hover_declarative_paint_invalidations: u32,
    #[serde(default)]
    pub view_cache_active: bool,
    #[serde(default)]
    pub view_cache_invalidation_truncations: u32,
    #[serde(default)]
    pub view_cache_contained_relayouts: u32,
    #[serde(default)]
    pub view_cache_roots_total: u32,
    #[serde(default)]
    pub view_cache_roots_reused: u32,
    #[serde(default)]
    pub view_cache_roots_first_mount: u32,
    #[serde(default)]
    pub view_cache_roots_node_recreated: u32,
    #[serde(default)]
    pub view_cache_roots_cache_key_mismatch: u32,
    #[serde(default)]
    pub view_cache_roots_not_marked_reuse_root: u32,
    #[serde(default)]
    pub view_cache_roots_needs_rerender: u32,
    #[serde(default)]
    pub view_cache_roots_layout_invalidated: u32,
    #[serde(default)]
    pub view_cache_roots_manual: u32,
    #[serde(default)]
    pub set_children_barrier_writes: u32,
    #[serde(default)]
    pub barrier_relayouts_scheduled: u32,
    #[serde(default)]
    pub barrier_relayouts_performed: u32,
    #[serde(default)]
    pub virtual_list_visible_range_checks: u32,
    #[serde(default)]
    pub virtual_list_visible_range_refreshes: u32,
    #[serde(default)]
    pub virtual_list_window_shifts_total: u32,
    #[serde(default)]
    pub virtual_list_window_shifts_non_retained: u32,
    #[serde(default)]
    pub retained_virtual_list_reconciles: u32,
    #[serde(default)]
    pub retained_virtual_list_attached_items: u32,
    #[serde(default)]
    pub retained_virtual_list_detached_items: u32,
    pub focused_node: Option<u64>,
    pub captured_node: Option<u64>,

    // Renderer (wgpu) perf sample (best-effort; may be absent or lag a frame).
    #[serde(default)]
    pub renderer_tick_id: u64,
    #[serde(default)]
    pub renderer_frame_id: u64,
    #[serde(default)]
    pub renderer_frames: u64,
    #[serde(default)]
    pub renderer_encode_scene_us: u64,
    #[serde(default)]
    pub renderer_ensure_pipelines_us: u64,
    #[serde(default)]
    pub renderer_plan_compile_us: u64,
    #[serde(default)]
    pub renderer_upload_us: u64,
    #[serde(default)]
    pub renderer_record_passes_us: u64,
    #[serde(default)]
    pub renderer_encoder_finish_us: u64,
    #[serde(default)]
    pub renderer_prepare_svg_us: u64,
    #[serde(default)]
    pub renderer_prepare_text_us: u64,
    #[serde(default)]
    pub renderer_svg_uploads: u64,
    #[serde(default)]
    pub renderer_svg_upload_bytes: u64,
    #[serde(default)]
    pub renderer_image_uploads: u64,
    #[serde(default)]
    pub renderer_image_upload_bytes: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_unknown: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_owned: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_external_zero_copy: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_gpu_copy: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_cpu_upload: u64,
    #[serde(default)]
    pub renderer_render_target_updates_requested_ingest_unknown: u64,
    #[serde(default)]
    pub renderer_render_target_updates_requested_ingest_owned: u64,
    #[serde(default)]
    pub renderer_render_target_updates_requested_ingest_external_zero_copy: u64,
    #[serde(default)]
    pub renderer_render_target_updates_requested_ingest_gpu_copy: u64,
    #[serde(default)]
    pub renderer_render_target_updates_requested_ingest_cpu_upload: u64,
    #[serde(default)]
    pub renderer_render_target_updates_ingest_fallbacks: u64,
    #[serde(default)]
    pub renderer_render_target_metadata_degradations_color_encoding_dropped: u64,
    #[serde(default)]
    pub renderer_svg_raster_budget_bytes: u64,
    #[serde(default)]
    pub renderer_svg_rasters_live: u64,
    #[serde(default)]
    pub renderer_svg_standalone_bytes_live: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_pages_live: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_bytes_live: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_used_px: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_capacity_px: u64,
    #[serde(default)]
    pub renderer_svg_raster_cache_hits: u64,
    #[serde(default)]
    pub renderer_svg_raster_cache_misses: u64,
    #[serde(default)]
    pub renderer_svg_raster_budget_evictions: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_page_evictions: u64,
    #[serde(default)]
    pub renderer_svg_mask_atlas_entries_evicted: u64,
    #[serde(default)]
    pub renderer_text_atlas_revision: u64,
    #[serde(default)]
    pub renderer_text_atlas_uploads: u64,
    #[serde(default)]
    pub renderer_text_atlas_upload_bytes: u64,
    #[serde(default)]
    pub renderer_text_atlas_evicted_glyphs: u64,
    #[serde(default)]
    pub renderer_text_atlas_evicted_pages: u64,
    #[serde(default)]
    pub renderer_text_atlas_evicted_page_glyphs: u64,
    #[serde(default)]
    pub renderer_text_atlas_resets: u64,
    #[serde(default)]
    pub renderer_intermediate_budget_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_in_use_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_peak_in_use_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_release_targets: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_allocations: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_reuses: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_releases: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_evictions: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_free_bytes: u64,
    #[serde(default)]
    pub renderer_intermediate_pool_free_textures: u64,
    #[serde(default)]
    pub renderer_draw_calls: u64,
    #[serde(default)]
    pub renderer_text_draw_calls: u64,
    #[serde(default)]
    pub renderer_quad_draw_calls: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls_ingest_unknown: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls_ingest_owned: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls_ingest_external_zero_copy: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls_ingest_gpu_copy: u64,
    #[serde(default)]
    pub renderer_viewport_draw_calls_ingest_cpu_upload: u64,
    #[serde(default)]
    pub renderer_mask_draw_calls: u64,
    #[serde(default)]
    pub renderer_pipeline_switches: u64,
    #[serde(default)]
    pub renderer_bind_group_switches: u64,
    #[serde(default)]
    pub renderer_scissor_sets: u64,
    #[serde(default)]
    pub renderer_uniform_bytes: u64,
    #[serde(default)]
    pub renderer_instance_bytes: u64,
    #[serde(default)]
    pub renderer_vertex_bytes: u64,
    #[serde(default)]
    pub renderer_scene_encoding_cache_hits: u64,
    #[serde(default)]
    pub renderer_scene_encoding_cache_misses: u64,
    #[serde(default)]
    pub renderer_material_quad_ops: u64,
    #[serde(default)]
    pub renderer_material_sampled_quad_ops: u64,
    #[serde(default)]
    pub renderer_material_distinct: u64,
    #[serde(default)]
    pub renderer_material_unknown_ids: u64,
    #[serde(default)]
    pub renderer_material_degraded_due_to_budget: u64,

    // Renderer effect degradation counters (best-effort). These are only populated when
    // `FRET_DIAG_RENDERER_PERF=1` and the runner records perf samples.
    #[serde(default)]
    pub renderer_custom_effect_v1_steps_requested: u64,
    #[serde(default)]
    pub renderer_custom_effect_v1_passes_emitted: u64,
    #[serde(default)]
    pub renderer_custom_effect_v2_steps_requested: u64,
    #[serde(default)]
    pub renderer_custom_effect_v2_passes_emitted: u64,
    #[serde(default)]
    pub renderer_custom_effect_v3_steps_requested: u64,
    #[serde(default)]
    pub renderer_custom_effect_v3_passes_emitted: u64,
    #[serde(default)]
    pub renderer_custom_effect_v3_sources_raw_requested: u64,
    #[serde(default)]
    pub renderer_custom_effect_v3_sources_raw_distinct: u64,
    #[serde(default)]
    pub renderer_custom_effect_v3_sources_raw_aliased_to_src: u64,
    #[serde(default)]
    pub renderer_custom_effect_v3_sources_pyramid_requested: u64,
    #[serde(default)]
    pub renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2: u64,
    #[serde(default)]
    pub renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero: u64,
    #[serde(default)]
    pub renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_insufficient: u64,
    #[serde(default)]
    pub renderer_backdrop_source_groups_requested: u64,
    #[serde(default)]
    pub renderer_backdrop_source_groups_applied_raw: u64,
    #[serde(default)]
    pub renderer_backdrop_source_groups_raw_degraded_budget_zero: u64,
    #[serde(default)]
    pub renderer_backdrop_source_groups_raw_degraded_budget_insufficient: u64,
    #[serde(default)]
    pub renderer_backdrop_source_groups_raw_degraded_target_exhausted: u64,
    #[serde(default)]
    pub renderer_backdrop_source_groups_pyramid_requested: u64,
    #[serde(default)]
    pub renderer_backdrop_source_groups_pyramid_applied_levels_ge2: u64,
    #[serde(default)]
    pub renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_zero: u64,
    #[serde(default)]
    pub renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_insufficient: u64,
    #[serde(default)]
    pub renderer_backdrop_source_groups_pyramid_skipped_raw_unavailable: u64,
}

impl UiFrameStatsV1 {
    fn from_stats(
        stats: UiDebugFrameStats,
        renderer_perf: Option<fret_render::RendererPerfFrameSample>,
    ) -> Self {
        let cpu = ui_thread_cpu_time::sample_current_thread(stats.frame_id.0);

        let mut out = Self {
            frame_arena_capacity_estimate_bytes: stats.frame_arena_capacity_estimate_bytes,
            frame_arena_grow_events: stats.frame_arena_grow_events,
            element_children_vec_pool_reuses: stats.element_children_vec_pool_reuses,
            element_children_vec_pool_misses: stats.element_children_vec_pool_misses,
            ui_thread_cpu_time_us: cpu.delta_time_us,
            ui_thread_cpu_total_time_us: cpu.total_time_us,
            ui_thread_cpu_cycle_time_delta_cycles: cpu.delta_cycles,
            ui_thread_cpu_cycle_time_total_cycles: cpu.total_cycles,
            layout_time_us: stats.layout_time.as_micros() as u64,
            layout_collect_roots_time_us: stats.layout_collect_roots_time.as_micros() as u64,
            layout_invalidate_scroll_handle_bindings_time_us: stats
                .layout_invalidate_scroll_handle_bindings_time
                .as_micros() as u64,
            layout_expand_view_cache_invalidations_time_us: stats
                .layout_expand_view_cache_invalidations_time
                .as_micros() as u64,
            layout_request_build_roots_time_us: stats.layout_request_build_roots_time.as_micros()
                as u64,
            layout_pending_barrier_relayouts_time_us: stats
                .layout_pending_barrier_relayouts_time
                .as_micros() as u64,
            layout_repair_view_cache_bounds_time_us: stats
                .layout_repair_view_cache_bounds_time
                .as_micros() as u64,
            layout_contained_view_cache_roots_time_us: stats
                .layout_contained_view_cache_roots_time
                .as_micros() as u64,
            layout_collapse_layout_observations_time_us: stats
                .layout_collapse_layout_observations_time
                .as_micros() as u64,
            layout_observation_record_time_us: stats.layout_observation_record_time.as_micros()
                as u64,
            layout_observation_record_models_items: stats.layout_observation_record_models_items,
            layout_observation_record_globals_items: stats.layout_observation_record_globals_items,
            layout_prepaint_after_layout_time_us: stats
                .layout_prepaint_after_layout_time
                .as_micros() as u64,
            layout_skipped_engine_frame: stats.layout_skipped_engine_frame,
            layout_roots_time_us: stats.layout_roots_time.as_micros() as u64,
            layout_barrier_relayouts_time_us: stats.layout_barrier_relayouts_time.as_micros()
                as u64,
            layout_view_cache_time_us: stats.layout_view_cache_time.as_micros() as u64,
            layout_semantics_refresh_time_us: stats.layout_semantics_refresh_time.as_micros()
                as u64,
            layout_focus_repair_time_us: stats.layout_focus_repair_time.as_micros() as u64,
            layout_deferred_cleanup_time_us: stats.layout_deferred_cleanup_time.as_micros() as u64,
            prepaint_time_us: stats.prepaint_time.as_micros() as u64,
            paint_time_us: stats.paint_time.as_micros() as u64,
            paint_record_visual_bounds_time_us: stats.paint_record_visual_bounds_time.as_micros()
                as u64,
            paint_record_visual_bounds_calls: stats.paint_record_visual_bounds_calls,
            paint_cache_key_time_us: stats.paint_cache_key_time.as_micros() as u64,
            paint_cache_hit_check_time_us: stats.paint_cache_hit_check_time.as_micros() as u64,
            paint_widget_time_us: stats.paint_widget_time.as_micros() as u64,
            paint_observation_record_time_us: stats.paint_observation_record_time.as_micros()
                as u64,
            paint_host_widget_observed_models_time_us: stats
                .paint_host_widget_observed_models_time
                .as_micros() as u64,
            paint_host_widget_observed_models_items: stats.paint_host_widget_observed_models_items,
            paint_host_widget_observed_globals_time_us: stats
                .paint_host_widget_observed_globals_time
                .as_micros() as u64,
            paint_host_widget_observed_globals_items: stats
                .paint_host_widget_observed_globals_items,
            paint_host_widget_instance_lookup_time_us: stats
                .paint_host_widget_instance_lookup_time
                .as_micros() as u64,
            paint_host_widget_instance_lookup_calls: stats.paint_host_widget_instance_lookup_calls,
            paint_text_prepare_time_us: stats.paint_text_prepare_time.as_micros() as u64,
            paint_text_prepare_calls: stats.paint_text_prepare_calls,
            paint_text_prepare_reason_blob_missing: stats.paint_text_prepare_reason_blob_missing,
            paint_text_prepare_reason_scale_changed: stats.paint_text_prepare_reason_scale_changed,
            paint_text_prepare_reason_text_changed: stats.paint_text_prepare_reason_text_changed,
            paint_text_prepare_reason_rich_changed: stats.paint_text_prepare_reason_rich_changed,
            paint_text_prepare_reason_style_changed: stats.paint_text_prepare_reason_style_changed,
            paint_text_prepare_reason_wrap_changed: stats.paint_text_prepare_reason_wrap_changed,
            paint_text_prepare_reason_overflow_changed: stats
                .paint_text_prepare_reason_overflow_changed,
            paint_text_prepare_reason_width_changed: stats.paint_text_prepare_reason_width_changed,
            paint_text_prepare_reason_font_stack_changed: stats
                .paint_text_prepare_reason_font_stack_changed,
            paint_input_context_time_us: stats.paint_input_context_time.as_micros() as u64,
            paint_scroll_handle_invalidation_time_us: stats
                .paint_scroll_handle_invalidation_time
                .as_micros() as u64,
            paint_collect_roots_time_us: stats.paint_collect_roots_time.as_micros() as u64,
            paint_publish_text_input_snapshot_time_us: stats
                .paint_publish_text_input_snapshot_time
                .as_micros() as u64,
            paint_collapse_observations_time_us: stats.paint_collapse_observations_time.as_micros()
                as u64,
            dispatch_time_us: stats.dispatch_time.as_micros() as u64,
            dispatch_pointer_events: stats.dispatch_pointer_events,
            dispatch_pointer_event_time_us: stats.dispatch_pointer_event_time.as_micros() as u64,
            dispatch_timer_events: stats.dispatch_timer_events,
            dispatch_timer_event_time_us: stats.dispatch_timer_event_time.as_micros() as u64,
            dispatch_timer_targeted_events: stats.dispatch_timer_targeted_events,
            dispatch_timer_targeted_time_us: stats.dispatch_timer_targeted_time.as_micros() as u64,
            dispatch_timer_broadcast_events: stats.dispatch_timer_broadcast_events,
            dispatch_timer_broadcast_time_us: stats.dispatch_timer_broadcast_time.as_micros()
                as u64,
            dispatch_timer_broadcast_layers_visited: stats.dispatch_timer_broadcast_layers_visited,
            dispatch_timer_broadcast_rebuild_visible_layers_time_us: stats
                .dispatch_timer_broadcast_rebuild_visible_layers_time
                .as_micros()
                as u64,
            dispatch_timer_broadcast_loop_time_us: stats
                .dispatch_timer_broadcast_loop_time
                .as_micros() as u64,
            dispatch_timer_slowest_event_time_us: stats
                .dispatch_timer_slowest_event_time
                .as_micros() as u64,
            dispatch_timer_slowest_token: stats.dispatch_timer_slowest_token.map(|t| t.0),
            dispatch_timer_slowest_was_broadcast: stats.dispatch_timer_slowest_was_broadcast,
            dispatch_other_events: stats.dispatch_other_events,
            dispatch_other_event_time_us: stats.dispatch_other_event_time.as_micros() as u64,
            hit_test_time_us: stats.hit_test_time.as_micros() as u64,
            dispatch_events: stats.dispatch_events,
            hit_test_queries: stats.hit_test_queries,
            hit_test_bounds_tree_queries: stats.hit_test_bounds_tree_queries,
            hit_test_bounds_tree_disabled: stats.hit_test_bounds_tree_disabled,
            hit_test_bounds_tree_misses: stats.hit_test_bounds_tree_misses,
            hit_test_bounds_tree_hits: stats.hit_test_bounds_tree_hits,
            hit_test_bounds_tree_candidate_rejected: stats.hit_test_bounds_tree_candidate_rejected,
            hit_test_bounds_tree_nodes_visited: stats.hit_test_bounds_tree_nodes_visited,
            hit_test_bounds_tree_nodes_pushed: stats.hit_test_bounds_tree_nodes_pushed,
            hit_test_path_cache_hits: stats.hit_test_path_cache_hits,
            hit_test_path_cache_misses: stats.hit_test_path_cache_misses,
            hit_test_cached_path_time_us: stats.hit_test_cached_path_time.as_micros() as u64,
            hit_test_bounds_tree_query_time_us: stats.hit_test_bounds_tree_query_time.as_micros()
                as u64,
            hit_test_candidate_self_only_time_us: stats
                .hit_test_candidate_self_only_time
                .as_micros() as u64,
            hit_test_fallback_traversal_time_us: stats.hit_test_fallback_traversal_time.as_micros()
                as u64,
            dispatch_hover_update_time_us: stats.dispatch_hover_update_time.as_micros() as u64,
            dispatch_scroll_handle_invalidation_time_us: stats
                .dispatch_scroll_handle_invalidation_time
                .as_micros() as u64,
            dispatch_active_layers_time_us: stats.dispatch_active_layers_time.as_micros() as u64,
            dispatch_input_context_time_us: stats.dispatch_input_context_time.as_micros() as u64,
            dispatch_event_chain_build_time_us: stats.dispatch_event_chain_build_time.as_micros()
                as u64,
            dispatch_widget_capture_time_us: stats.dispatch_widget_capture_time.as_micros() as u64,
            dispatch_widget_bubble_time_us: stats.dispatch_widget_bubble_time.as_micros() as u64,
            dispatch_cursor_query_time_us: stats.dispatch_cursor_query_time.as_micros() as u64,
            dispatch_pointer_move_layer_observers_time_us: stats
                .dispatch_pointer_move_layer_observers_time
                .as_micros() as u64,
            dispatch_synth_hover_observer_time_us: stats
                .dispatch_synth_hover_observer_time
                .as_micros() as u64,
            dispatch_cursor_effect_time_us: stats.dispatch_cursor_effect_time.as_micros() as u64,
            dispatch_post_dispatch_snapshot_time_us: stats
                .dispatch_post_dispatch_snapshot_time
                .as_micros() as u64,
            layout_nodes_visited: stats.layout_nodes_visited,
            layout_nodes_performed: stats.layout_nodes_performed,
            prepaint_nodes_visited: stats.prepaint_nodes_visited,
            paint_nodes: stats.paint_nodes,
            paint_nodes_performed: stats.paint_nodes_performed,
            paint_cache_hits: stats.paint_cache_hits,
            paint_cache_misses: stats.paint_cache_misses,
            paint_cache_replayed_ops: stats.paint_cache_replayed_ops,
            paint_cache_replay_time_us: stats.paint_cache_replay_time.as_micros() as u64,
            paint_cache_bounds_translate_time_us: stats
                .paint_cache_bounds_translate_time
                .as_micros() as u64,
            paint_cache_bounds_translated_nodes: stats.paint_cache_bounds_translated_nodes,
            interaction_cache_hits: stats.interaction_cache_hits,
            interaction_cache_misses: stats.interaction_cache_misses,
            interaction_cache_replayed_records: stats.interaction_cache_replayed_records,
            interaction_records: stats.interaction_records,
            layout_engine_solves: stats.layout_engine_solves,
            layout_engine_solve_time_us: stats.layout_engine_solve_time.as_micros() as u64,
            layout_engine_widget_fallback_solves: stats.layout_engine_widget_fallback_solves,
            layout_fast_path_taken: stats.layout_fast_path_taken,
            layout_invalidations_count: stats.layout_invalidations_count,
            model_change_invalidation_roots: stats.model_change_invalidation_roots,
            model_change_models: stats.model_change_models,
            model_change_observation_edges: stats.model_change_observation_edges,
            model_change_unobserved_models: stats.model_change_unobserved_models,
            global_change_invalidation_roots: stats.global_change_invalidation_roots,
            global_change_globals: stats.global_change_globals,
            global_change_observation_edges: stats.global_change_observation_edges,
            global_change_unobserved_globals: stats.global_change_unobserved_globals,
            invalidation_walk_nodes: stats.invalidation_walk_nodes,
            invalidation_walk_calls: stats.invalidation_walk_calls,
            invalidation_walk_nodes_model_change: stats.invalidation_walk_nodes_model_change,
            invalidation_walk_calls_model_change: stats.invalidation_walk_calls_model_change,
            invalidation_walk_nodes_global_change: stats.invalidation_walk_nodes_global_change,
            invalidation_walk_calls_global_change: stats.invalidation_walk_calls_global_change,
            invalidation_walk_nodes_hover: stats.invalidation_walk_nodes_hover,
            invalidation_walk_calls_hover: stats.invalidation_walk_calls_hover,
            invalidation_walk_nodes_focus: stats.invalidation_walk_nodes_focus,
            invalidation_walk_calls_focus: stats.invalidation_walk_calls_focus,
            invalidation_walk_nodes_other: stats.invalidation_walk_nodes_other,
            invalidation_walk_calls_other: stats.invalidation_walk_calls_other,
            hover_pressable_target_changes: stats.hover_pressable_target_changes,
            hover_hover_region_target_changes: stats.hover_hover_region_target_changes,
            hover_declarative_instance_changes: stats.hover_declarative_instance_changes,
            hover_declarative_hit_test_invalidations: stats
                .hover_declarative_hit_test_invalidations,
            hover_declarative_layout_invalidations: stats.hover_declarative_layout_invalidations,
            hover_declarative_paint_invalidations: stats.hover_declarative_paint_invalidations,
            view_cache_active: stats.view_cache_active,
            view_cache_invalidation_truncations: stats.view_cache_invalidation_truncations,
            view_cache_contained_relayouts: stats.view_cache_contained_relayouts,
            view_cache_roots_total: stats.view_cache_roots_total,
            view_cache_roots_reused: stats.view_cache_roots_reused,
            view_cache_roots_first_mount: stats.view_cache_roots_first_mount,
            view_cache_roots_node_recreated: stats.view_cache_roots_node_recreated,
            view_cache_roots_cache_key_mismatch: stats.view_cache_roots_cache_key_mismatch,
            view_cache_roots_not_marked_reuse_root: stats.view_cache_roots_not_marked_reuse_root,
            view_cache_roots_needs_rerender: stats.view_cache_roots_needs_rerender,
            view_cache_roots_layout_invalidated: stats.view_cache_roots_layout_invalidated,
            view_cache_roots_manual: stats.view_cache_roots_manual,
            set_children_barrier_writes: stats.set_children_barrier_writes,
            barrier_relayouts_scheduled: stats.barrier_relayouts_scheduled,
            barrier_relayouts_performed: stats.barrier_relayouts_performed,
            virtual_list_visible_range_checks: stats.virtual_list_visible_range_checks,
            virtual_list_visible_range_refreshes: stats.virtual_list_visible_range_refreshes,
            virtual_list_window_shifts_total: stats.virtual_list_window_shifts_total,
            virtual_list_window_shifts_non_retained: stats.virtual_list_window_shifts_non_retained,
            retained_virtual_list_reconciles: stats.retained_virtual_list_reconciles,
            retained_virtual_list_attached_items: stats.retained_virtual_list_attached_items,
            retained_virtual_list_detached_items: stats.retained_virtual_list_detached_items,
            focused_node: stats.focus.map(key_to_u64),
            captured_node: stats.captured.map(key_to_u64),
            renderer_tick_id: 0,
            renderer_frame_id: 0,
            renderer_frames: 0,
            renderer_encode_scene_us: 0,
            renderer_ensure_pipelines_us: 0,
            renderer_plan_compile_us: 0,
            renderer_upload_us: 0,
            renderer_record_passes_us: 0,
            renderer_encoder_finish_us: 0,
            renderer_prepare_svg_us: 0,
            renderer_prepare_text_us: 0,
            renderer_svg_uploads: 0,
            renderer_svg_upload_bytes: 0,
            renderer_image_uploads: 0,
            renderer_image_upload_bytes: 0,
            renderer_render_target_updates_ingest_unknown: 0,
            renderer_render_target_updates_ingest_owned: 0,
            renderer_render_target_updates_ingest_external_zero_copy: 0,
            renderer_render_target_updates_ingest_gpu_copy: 0,
            renderer_render_target_updates_ingest_cpu_upload: 0,
            renderer_render_target_updates_requested_ingest_unknown: 0,
            renderer_render_target_updates_requested_ingest_owned: 0,
            renderer_render_target_updates_requested_ingest_external_zero_copy: 0,
            renderer_render_target_updates_requested_ingest_gpu_copy: 0,
            renderer_render_target_updates_requested_ingest_cpu_upload: 0,
            renderer_render_target_updates_ingest_fallbacks: 0,
            renderer_render_target_metadata_degradations_color_encoding_dropped: 0,
            renderer_svg_raster_budget_bytes: 0,
            renderer_svg_rasters_live: 0,
            renderer_svg_standalone_bytes_live: 0,
            renderer_svg_mask_atlas_pages_live: 0,
            renderer_svg_mask_atlas_bytes_live: 0,
            renderer_svg_mask_atlas_used_px: 0,
            renderer_svg_mask_atlas_capacity_px: 0,
            renderer_svg_raster_cache_hits: 0,
            renderer_svg_raster_cache_misses: 0,
            renderer_svg_raster_budget_evictions: 0,
            renderer_svg_mask_atlas_page_evictions: 0,
            renderer_svg_mask_atlas_entries_evicted: 0,
            renderer_text_atlas_revision: 0,
            renderer_text_atlas_uploads: 0,
            renderer_text_atlas_upload_bytes: 0,
            renderer_text_atlas_evicted_glyphs: 0,
            renderer_text_atlas_evicted_pages: 0,
            renderer_text_atlas_evicted_page_glyphs: 0,
            renderer_text_atlas_resets: 0,
            renderer_intermediate_budget_bytes: 0,
            renderer_intermediate_in_use_bytes: 0,
            renderer_intermediate_peak_in_use_bytes: 0,
            renderer_intermediate_release_targets: 0,
            renderer_intermediate_pool_allocations: 0,
            renderer_intermediate_pool_reuses: 0,
            renderer_intermediate_pool_releases: 0,
            renderer_intermediate_pool_evictions: 0,
            renderer_intermediate_pool_free_bytes: 0,
            renderer_intermediate_pool_free_textures: 0,
            renderer_draw_calls: 0,
            renderer_text_draw_calls: 0,
            renderer_quad_draw_calls: 0,
            renderer_viewport_draw_calls: 0,
            renderer_viewport_draw_calls_ingest_unknown: 0,
            renderer_viewport_draw_calls_ingest_owned: 0,
            renderer_viewport_draw_calls_ingest_external_zero_copy: 0,
            renderer_viewport_draw_calls_ingest_gpu_copy: 0,
            renderer_viewport_draw_calls_ingest_cpu_upload: 0,
            renderer_mask_draw_calls: 0,
            renderer_pipeline_switches: 0,
            renderer_bind_group_switches: 0,
            renderer_scissor_sets: 0,
            renderer_uniform_bytes: 0,
            renderer_instance_bytes: 0,
            renderer_vertex_bytes: 0,
            renderer_scene_encoding_cache_hits: 0,
            renderer_scene_encoding_cache_misses: 0,
            renderer_material_quad_ops: 0,
            renderer_material_sampled_quad_ops: 0,
            renderer_material_distinct: 0,
            renderer_material_unknown_ids: 0,
            renderer_material_degraded_due_to_budget: 0,
            renderer_custom_effect_v1_steps_requested: 0,
            renderer_custom_effect_v1_passes_emitted: 0,
            renderer_custom_effect_v2_steps_requested: 0,
            renderer_custom_effect_v2_passes_emitted: 0,
            renderer_custom_effect_v3_steps_requested: 0,
            renderer_custom_effect_v3_passes_emitted: 0,
            renderer_custom_effect_v3_sources_raw_requested: 0,
            renderer_custom_effect_v3_sources_raw_distinct: 0,
            renderer_custom_effect_v3_sources_raw_aliased_to_src: 0,
            renderer_custom_effect_v3_sources_pyramid_requested: 0,
            renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2: 0,
            renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero: 0,
            renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_insufficient: 0,
            renderer_backdrop_source_groups_requested: 0,
            renderer_backdrop_source_groups_applied_raw: 0,
            renderer_backdrop_source_groups_raw_degraded_budget_zero: 0,
            renderer_backdrop_source_groups_raw_degraded_budget_insufficient: 0,
            renderer_backdrop_source_groups_raw_degraded_target_exhausted: 0,
            renderer_backdrop_source_groups_pyramid_requested: 0,
            renderer_backdrop_source_groups_pyramid_applied_levels_ge2: 0,
            renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_zero: 0,
            renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_insufficient: 0,
            renderer_backdrop_source_groups_pyramid_skipped_raw_unavailable: 0,
        };

        if let Some(sample) = renderer_perf {
            out.renderer_tick_id = sample.tick_id;
            out.renderer_frame_id = sample.frame_id;
            out.renderer_frames = sample.perf.frames;
            out.renderer_encode_scene_us = sample.perf.encode_scene_us;
            out.renderer_ensure_pipelines_us = sample.perf.ensure_pipelines_us;
            out.renderer_plan_compile_us = sample.perf.plan_compile_us;
            out.renderer_upload_us = sample.perf.upload_us;
            out.renderer_record_passes_us = sample.perf.record_passes_us;
            out.renderer_encoder_finish_us = sample.perf.encoder_finish_us;
            out.renderer_prepare_svg_us = sample.perf.prepare_svg_us;
            out.renderer_prepare_text_us = sample.perf.prepare_text_us;
            out.renderer_svg_uploads = sample.perf.svg_uploads;
            out.renderer_svg_upload_bytes = sample.perf.svg_upload_bytes;
            out.renderer_image_uploads = sample.perf.image_uploads;
            out.renderer_image_upload_bytes = sample.perf.image_upload_bytes;
            out.renderer_render_target_updates_ingest_unknown =
                sample.perf.render_target_updates_ingest_unknown;
            out.renderer_render_target_updates_ingest_owned =
                sample.perf.render_target_updates_ingest_owned;
            out.renderer_render_target_updates_ingest_external_zero_copy =
                sample.perf.render_target_updates_ingest_external_zero_copy;
            out.renderer_render_target_updates_ingest_gpu_copy =
                sample.perf.render_target_updates_ingest_gpu_copy;
            out.renderer_render_target_updates_ingest_cpu_upload =
                sample.perf.render_target_updates_ingest_cpu_upload;
            out.renderer_render_target_updates_requested_ingest_unknown =
                sample.perf.render_target_updates_requested_ingest_unknown;
            out.renderer_render_target_updates_requested_ingest_owned =
                sample.perf.render_target_updates_requested_ingest_owned;
            out.renderer_render_target_updates_requested_ingest_external_zero_copy = sample
                .perf
                .render_target_updates_requested_ingest_external_zero_copy;
            out.renderer_render_target_updates_requested_ingest_gpu_copy =
                sample.perf.render_target_updates_requested_ingest_gpu_copy;
            out.renderer_render_target_updates_requested_ingest_cpu_upload = sample
                .perf
                .render_target_updates_requested_ingest_cpu_upload;
            out.renderer_render_target_updates_ingest_fallbacks =
                sample.perf.render_target_updates_ingest_fallbacks;
            out.renderer_render_target_metadata_degradations_color_encoding_dropped = sample
                .perf
                .render_target_metadata_degradations_color_encoding_dropped;
            out.renderer_svg_raster_budget_bytes = sample.perf.svg_raster_budget_bytes;
            out.renderer_svg_rasters_live = sample.perf.svg_rasters_live;
            out.renderer_svg_standalone_bytes_live = sample.perf.svg_standalone_bytes_live;
            out.renderer_svg_mask_atlas_pages_live = sample.perf.svg_mask_atlas_pages_live;
            out.renderer_svg_mask_atlas_bytes_live = sample.perf.svg_mask_atlas_bytes_live;
            out.renderer_svg_mask_atlas_used_px = sample.perf.svg_mask_atlas_used_px;
            out.renderer_svg_mask_atlas_capacity_px = sample.perf.svg_mask_atlas_capacity_px;
            out.renderer_svg_raster_cache_hits = sample.perf.svg_raster_cache_hits;
            out.renderer_svg_raster_cache_misses = sample.perf.svg_raster_cache_misses;
            out.renderer_svg_raster_budget_evictions = sample.perf.svg_raster_budget_evictions;
            out.renderer_svg_mask_atlas_page_evictions = sample.perf.svg_mask_atlas_page_evictions;
            out.renderer_svg_mask_atlas_entries_evicted =
                sample.perf.svg_mask_atlas_entries_evicted;
            out.renderer_text_atlas_revision = sample.perf.text_atlas_revision;
            out.renderer_text_atlas_uploads = sample.perf.text_atlas_uploads;
            out.renderer_text_atlas_upload_bytes = sample.perf.text_atlas_upload_bytes;
            out.renderer_text_atlas_evicted_glyphs = sample.perf.text_atlas_evicted_glyphs;
            out.renderer_text_atlas_evicted_pages = sample.perf.text_atlas_evicted_pages;
            out.renderer_text_atlas_evicted_page_glyphs =
                sample.perf.text_atlas_evicted_page_glyphs;
            out.renderer_text_atlas_resets = sample.perf.text_atlas_resets;
            out.renderer_intermediate_budget_bytes = sample.perf.intermediate_budget_bytes;
            out.renderer_intermediate_in_use_bytes = sample.perf.intermediate_in_use_bytes;
            out.renderer_intermediate_peak_in_use_bytes =
                sample.perf.intermediate_peak_in_use_bytes;
            out.renderer_intermediate_release_targets = sample.perf.intermediate_release_targets;
            out.renderer_intermediate_pool_allocations = sample.perf.intermediate_pool_allocations;
            out.renderer_intermediate_pool_reuses = sample.perf.intermediate_pool_reuses;
            out.renderer_intermediate_pool_releases = sample.perf.intermediate_pool_releases;
            out.renderer_intermediate_pool_evictions = sample.perf.intermediate_pool_evictions;
            out.renderer_intermediate_pool_free_bytes = sample.perf.intermediate_pool_free_bytes;
            out.renderer_intermediate_pool_free_textures =
                sample.perf.intermediate_pool_free_textures;
            out.renderer_draw_calls = sample.perf.draw_calls;
            out.renderer_text_draw_calls = sample.perf.text_draw_calls;
            out.renderer_quad_draw_calls = sample.perf.quad_draw_calls;
            out.renderer_viewport_draw_calls = sample.perf.viewport_draw_calls;
            out.renderer_viewport_draw_calls_ingest_unknown =
                sample.perf.viewport_draw_calls_ingest_unknown;
            out.renderer_viewport_draw_calls_ingest_owned =
                sample.perf.viewport_draw_calls_ingest_owned;
            out.renderer_viewport_draw_calls_ingest_external_zero_copy =
                sample.perf.viewport_draw_calls_ingest_external_zero_copy;
            out.renderer_viewport_draw_calls_ingest_gpu_copy =
                sample.perf.viewport_draw_calls_ingest_gpu_copy;
            out.renderer_viewport_draw_calls_ingest_cpu_upload =
                sample.perf.viewport_draw_calls_ingest_cpu_upload;
            out.renderer_mask_draw_calls = sample.perf.mask_draw_calls;
            out.renderer_pipeline_switches = sample.perf.pipeline_switches;
            out.renderer_bind_group_switches = sample.perf.bind_group_switches;
            out.renderer_scissor_sets = sample.perf.scissor_sets;
            out.renderer_uniform_bytes = sample.perf.uniform_bytes;
            out.renderer_instance_bytes = sample.perf.instance_bytes;
            out.renderer_vertex_bytes = sample.perf.vertex_bytes;
            out.renderer_scene_encoding_cache_hits = sample.perf.scene_encoding_cache_hits;
            out.renderer_scene_encoding_cache_misses = sample.perf.scene_encoding_cache_misses;
            out.renderer_material_quad_ops = sample.perf.material_quad_ops;
            out.renderer_material_sampled_quad_ops = sample.perf.material_sampled_quad_ops;
            out.renderer_material_distinct = sample.perf.material_distinct;
            out.renderer_material_unknown_ids = sample.perf.material_unknown_ids;
            out.renderer_material_degraded_due_to_budget =
                sample.perf.material_degraded_due_to_budget;

            out.renderer_custom_effect_v1_steps_requested =
                sample.perf.custom_effect_v1_steps_requested;
            out.renderer_custom_effect_v1_passes_emitted =
                sample.perf.custom_effect_v1_passes_emitted;
            out.renderer_custom_effect_v2_steps_requested =
                sample.perf.custom_effect_v2_steps_requested;
            out.renderer_custom_effect_v2_passes_emitted =
                sample.perf.custom_effect_v2_passes_emitted;
            out.renderer_custom_effect_v3_steps_requested =
                sample.perf.custom_effect_v3_steps_requested;
            out.renderer_custom_effect_v3_passes_emitted =
                sample.perf.custom_effect_v3_passes_emitted;

            let effects = sample.perf.effect_degradations;
            out.renderer_custom_effect_v3_sources_raw_requested =
                effects.custom_effect_v3_sources.raw_requested;
            out.renderer_custom_effect_v3_sources_raw_distinct =
                effects.custom_effect_v3_sources.raw_distinct;
            out.renderer_custom_effect_v3_sources_raw_aliased_to_src =
                effects.custom_effect_v3_sources.raw_aliased_to_src;
            out.renderer_custom_effect_v3_sources_pyramid_requested =
                effects.custom_effect_v3_sources.pyramid_requested;
            out.renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2 =
                effects.custom_effect_v3_sources.pyramid_applied_levels_ge2;
            out.renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero =
                effects.custom_effect_v3_sources.pyramid_degraded_to_one_budget_zero;
            out.renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_insufficient =
                effects
                    .custom_effect_v3_sources
                    .pyramid_degraded_to_one_budget_insufficient;

            out.renderer_backdrop_source_groups_requested = effects.backdrop_source_groups.requested;
            out.renderer_backdrop_source_groups_applied_raw = effects.backdrop_source_groups.applied_raw;
            out.renderer_backdrop_source_groups_raw_degraded_budget_zero =
                effects.backdrop_source_groups.raw_degraded_budget_zero;
            out.renderer_backdrop_source_groups_raw_degraded_budget_insufficient =
                effects.backdrop_source_groups.raw_degraded_budget_insufficient;
            out.renderer_backdrop_source_groups_raw_degraded_target_exhausted =
                effects.backdrop_source_groups.raw_degraded_target_exhausted;
            out.renderer_backdrop_source_groups_pyramid_requested =
                effects.backdrop_source_groups.pyramid_requested;
            out.renderer_backdrop_source_groups_pyramid_applied_levels_ge2 =
                effects.backdrop_source_groups.pyramid_applied_levels_ge2;
            out.renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_zero =
                effects.backdrop_source_groups.pyramid_degraded_to_one_budget_zero;
            out.renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_insufficient =
                effects.backdrop_source_groups.pyramid_degraded_to_one_budget_insufficient;
            out.renderer_backdrop_source_groups_pyramid_skipped_raw_unavailable =
                effects.backdrop_source_groups.pyramid_skipped_raw_unavailable;
        }

        out
    }
}
