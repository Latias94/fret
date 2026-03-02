#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsReport {
    sort: BundleStatsSort,
    warmup_frames: u64,
    derived_from_frames_index: bool,
    pub(super) windows: u32,
    pub(super) snapshots: u32,
    pub(super) snapshots_considered: u32,
    pub(super) snapshots_skipped_warmup: u32,
    pub(super) snapshots_with_model_changes: u32,
    pub(super) snapshots_with_global_changes: u32,
    snapshots_with_propagated_model_changes: u32,
    snapshots_with_propagated_global_changes: u32,
    pub(super) snapshots_with_hover_layout_invalidations: u32,
    /// Whether the bundle includes `pointer.move` events (so the derived "pointer move" frame set
    /// can be identified from the event log rather than inferred from dispatch-only frames).
    pub(super) pointer_move_frames_present: bool,
    /// Count of snapshots in the derived "pointer move" (or fallback) frame set.
    pub(super) pointer_move_frames_considered: u32,
    /// Max dispatch time (us) across the derived "pointer move" (or fallback) frame set.
    pub(super) pointer_move_max_dispatch_time_us: u64,
    /// Snapshot identity for `pointer_move_max_dispatch_time_us`.
    pub(super) pointer_move_max_dispatch_window: u64,
    pub(super) pointer_move_max_dispatch_tick_id: u64,
    pub(super) pointer_move_max_dispatch_frame_id: u64,
    /// Max hit-test time (us) across the derived "pointer move" (or fallback) frame set.
    pub(super) pointer_move_max_hit_test_time_us: u64,
    /// Snapshot identity for `pointer_move_max_hit_test_time_us`.
    pub(super) pointer_move_max_hit_test_window: u64,
    pub(super) pointer_move_max_hit_test_tick_id: u64,
    pub(super) pointer_move_max_hit_test_frame_id: u64,
    /// Number of snapshots within the derived "pointer move" (or fallback) frame set that had
    /// propagated global changes (`debug.stats.global_change_globals > 0`).
    pub(super) pointer_move_snapshots_with_global_changes: u32,
    sum_layout_collect_roots_time_us: u64,
    sum_layout_invalidate_scroll_handle_bindings_time_us: u64,
    sum_layout_expand_view_cache_invalidations_time_us: u64,
    sum_layout_request_build_roots_time_us: u64,
    sum_layout_roots_time_us: u64,
    sum_layout_collapse_layout_observations_time_us: u64,
    sum_layout_time_us: u64,
    sum_layout_view_cache_time_us: u64,
    sum_layout_prepaint_after_layout_time_us: u64,
    sum_layout_observation_record_time_us: u64,
    sum_layout_observation_record_models_items: u64,
    sum_layout_observation_record_globals_items: u64,
    sum_prepaint_time_us: u64,
    sum_paint_time_us: u64,
    sum_total_time_us: u64,
    sum_ui_thread_cpu_time_us: u64,
    sum_ui_thread_cpu_cycle_time_delta_cycles: u64,
    sum_layout_engine_solve_time_us: u64,
    sum_cache_roots: u64,
    sum_cache_roots_reused: u64,
    sum_cache_replayed_ops: u64,
    pub(super) sum_invalidation_walk_calls: u64,
    pub(super) sum_invalidation_walk_nodes: u64,
    sum_model_change_invalidation_roots: u64,
    sum_global_change_invalidation_roots: u64,
    pub(super) sum_hover_layout_invalidations: u64,
    max_layout_collect_roots_time_us: u64,
    max_layout_invalidate_scroll_handle_bindings_time_us: u64,
    max_layout_expand_view_cache_invalidations_time_us: u64,
    max_layout_request_build_roots_time_us: u64,
    max_layout_roots_time_us: u64,
    max_layout_collapse_layout_observations_time_us: u64,
    max_layout_time_us: u64,
    max_layout_view_cache_time_us: u64,
    max_layout_prepaint_after_layout_time_us: u64,
    max_layout_observation_record_time_us: u64,
    max_layout_observation_record_models_items: u32,
    max_layout_observation_record_globals_items: u32,
    pub(super) max_prepaint_time_us: u64,
    pub(super) max_paint_time_us: u64,
    pub(super) max_total_time_us: u64,
    pub(super) max_ui_thread_cpu_time_us: u64,
    pub(super) max_ui_thread_cpu_cycle_time_delta_cycles: u64,
    pub(super) max_layout_engine_solve_time_us: u64,
    pub(super) max_renderer_encode_scene_us: u64,
    pub(super) max_renderer_ensure_pipelines_us: u64,
    pub(super) max_renderer_plan_compile_us: u64,
    pub(super) max_renderer_upload_us: u64,
    pub(super) max_renderer_record_passes_us: u64,
    pub(super) max_renderer_encoder_finish_us: u64,
    pub(super) max_renderer_prepare_svg_us: u64,
    pub(super) max_renderer_prepare_text_us: u64,
    pub(super) max_invalidation_walk_calls: u32,
    pub(super) max_invalidation_walk_nodes: u32,
    max_model_change_invalidation_roots: u32,
    max_global_change_invalidation_roots: u32,
    pub(super) max_hover_layout_invalidations: u32,
    pub(super) p50_total_time_us: u64,
    pub(super) p95_total_time_us: u64,
    pub(super) p50_ui_thread_cpu_time_us: u64,
    pub(super) p95_ui_thread_cpu_time_us: u64,
    pub(super) p50_ui_thread_cpu_cycle_time_delta_cycles: u64,
    pub(super) p95_ui_thread_cpu_cycle_time_delta_cycles: u64,
    pub(super) p50_layout_time_us: u64,
    pub(super) p95_layout_time_us: u64,
    pub(super) p50_layout_collect_roots_time_us: u64,
    pub(super) p95_layout_collect_roots_time_us: u64,
    pub(super) p50_layout_request_build_roots_time_us: u64,
    pub(super) p95_layout_request_build_roots_time_us: u64,
    pub(super) p50_layout_roots_time_us: u64,
    pub(super) p95_layout_roots_time_us: u64,
    pub(super) p50_layout_view_cache_time_us: u64,
    pub(super) p95_layout_view_cache_time_us: u64,
    pub(super) p50_layout_collapse_layout_observations_time_us: u64,
    pub(super) p95_layout_collapse_layout_observations_time_us: u64,
    pub(super) p50_layout_prepaint_after_layout_time_us: u64,
    pub(super) p95_layout_prepaint_after_layout_time_us: u64,
    pub(super) p50_prepaint_time_us: u64,
    pub(super) p95_prepaint_time_us: u64,
    pub(super) p50_paint_time_us: u64,
    pub(super) p95_paint_time_us: u64,
    pub(super) p50_paint_input_context_time_us: u64,
    pub(super) p95_paint_input_context_time_us: u64,
    pub(super) p50_paint_scroll_handle_invalidation_time_us: u64,
    pub(super) p95_paint_scroll_handle_invalidation_time_us: u64,
    pub(super) p50_paint_collect_roots_time_us: u64,
    pub(super) p95_paint_collect_roots_time_us: u64,
    pub(super) p50_paint_publish_text_input_snapshot_time_us: u64,
    pub(super) p95_paint_publish_text_input_snapshot_time_us: u64,
    pub(super) p50_paint_collapse_observations_time_us: u64,
    pub(super) p95_paint_collapse_observations_time_us: u64,
    pub(super) p50_layout_engine_solve_time_us: u64,
    pub(super) p95_layout_engine_solve_time_us: u64,
    pub(super) p50_dispatch_time_us: u64,
    pub(super) p95_dispatch_time_us: u64,
    pub(super) p50_hit_test_time_us: u64,
    pub(super) p95_hit_test_time_us: u64,
    pub(super) p50_paint_widget_time_us: u64,
    pub(super) p95_paint_widget_time_us: u64,
    pub(super) p50_paint_text_prepare_time_us: u64,
    pub(super) p95_paint_text_prepare_time_us: u64,
    pub(super) p50_renderer_encode_scene_us: u64,
    pub(super) p95_renderer_encode_scene_us: u64,
    pub(super) p50_renderer_ensure_pipelines_us: u64,
    pub(super) p95_renderer_ensure_pipelines_us: u64,
    pub(super) p50_renderer_plan_compile_us: u64,
    pub(super) p95_renderer_plan_compile_us: u64,
    pub(super) p50_renderer_upload_us: u64,
    pub(super) p95_renderer_upload_us: u64,
    pub(super) p50_renderer_record_passes_us: u64,
    pub(super) p95_renderer_record_passes_us: u64,
    pub(super) p50_renderer_encoder_finish_us: u64,
    pub(super) p95_renderer_encoder_finish_us: u64,
    pub(super) p50_renderer_prepare_svg_us: u64,
    pub(super) p95_renderer_prepare_svg_us: u64,
    pub(super) p50_renderer_prepare_text_us: u64,
    pub(super) p95_renderer_prepare_text_us: u64,
    worst_hover_layout: Option<BundleStatsWorstHoverLayout>,
    global_type_hotspots: Vec<BundleStatsGlobalTypeHotspot>,
    model_source_hotspots: Vec<BundleStatsModelSourceHotspot>,
    pub(super) top: Vec<BundleStatsSnapshotRow>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsSnapshotRow {
    pub(super) window: u64,
    pub(super) tick_id: u64,
    pub(super) frame_id: u64,
    pub(super) timestamp_unix_ms: Option<u64>,
    pub(super) frame_arena_capacity_estimate_bytes: u64,
    pub(super) frame_arena_grow_events: u32,
    pub(super) element_children_vec_pool_reuses: u32,
    pub(super) element_children_vec_pool_misses: u32,
    pub(super) ui_thread_cpu_time_us: u64,
    pub(super) ui_thread_cpu_total_time_us: u64,
    pub(super) ui_thread_cpu_cycle_time_delta_cycles: u64,
    pub(super) ui_thread_cpu_cycle_time_total_cycles: u64,
    pub(super) layout_time_us: u64,
    pub(super) layout_collect_roots_time_us: u64,
    pub(super) layout_invalidate_scroll_handle_bindings_time_us: u64,
    pub(super) layout_expand_view_cache_invalidations_time_us: u64,
    pub(super) layout_request_build_roots_time_us: u64,
    pub(super) layout_roots_time_us: u64,
    pub(super) layout_pending_barrier_relayouts_time_us: u64,
    pub(super) layout_barrier_relayouts_time_us: u64,
    pub(super) layout_repair_view_cache_bounds_time_us: u64,
    pub(super) layout_contained_view_cache_roots_time_us: u64,
    pub(super) layout_collapse_layout_observations_time_us: u64,
    pub(super) layout_observation_record_time_us: u64,
    pub(super) layout_observation_record_models_items: u32,
    pub(super) layout_observation_record_globals_items: u32,
    pub(super) layout_view_cache_time_us: u64,
    pub(super) layout_semantics_refresh_time_us: u64,
    pub(super) layout_focus_repair_time_us: u64,
    pub(super) layout_deferred_cleanup_time_us: u64,
    pub(super) layout_prepaint_after_layout_time_us: u64,
    pub(super) layout_skipped_engine_frame: bool,
    pub(super) layout_fast_path_taken: bool,
    pub(super) prepaint_time_us: u64,
    pub(super) paint_time_us: u64,
    pub(super) paint_record_visual_bounds_time_us: u64,
    pub(super) paint_record_visual_bounds_calls: u32,
    pub(super) paint_cache_key_time_us: u64,
    pub(super) paint_cache_hit_check_time_us: u64,
    pub(super) paint_widget_time_us: u64,
    pub(super) paint_observation_record_time_us: u64,
    pub(super) paint_host_widget_observed_models_time_us: u64,
    pub(super) paint_host_widget_observed_models_items: u32,
    pub(super) paint_host_widget_observed_globals_time_us: u64,
    pub(super) paint_host_widget_observed_globals_items: u32,
    pub(super) paint_host_widget_instance_lookup_time_us: u64,
    pub(super) paint_host_widget_instance_lookup_calls: u32,
    pub(super) paint_text_prepare_time_us: u64,
    pub(super) paint_text_prepare_calls: u32,
    pub(super) paint_text_prepare_reason_blob_missing: u32,
    pub(super) paint_text_prepare_reason_scale_changed: u32,
    pub(super) paint_text_prepare_reason_text_changed: u32,
    pub(super) paint_text_prepare_reason_rich_changed: u32,
    pub(super) paint_text_prepare_reason_style_changed: u32,
    pub(super) paint_text_prepare_reason_wrap_changed: u32,
    pub(super) paint_text_prepare_reason_overflow_changed: u32,
    pub(super) paint_text_prepare_reason_width_changed: u32,
    pub(super) paint_text_prepare_reason_font_stack_changed: u32,
    pub(super) paint_input_context_time_us: u64,
    pub(super) paint_scroll_handle_invalidation_time_us: u64,
    pub(super) paint_collect_roots_time_us: u64,
    pub(super) paint_publish_text_input_snapshot_time_us: u64,
    pub(super) paint_collapse_observations_time_us: u64,
    pub(super) dispatch_time_us: u64,
    pub(super) dispatch_pointer_events: u32,
    pub(super) dispatch_pointer_event_time_us: u64,
    pub(super) dispatch_timer_events: u32,
    pub(super) dispatch_timer_event_time_us: u64,
    pub(super) dispatch_timer_targeted_events: u32,
    pub(super) dispatch_timer_targeted_time_us: u64,
    pub(super) dispatch_timer_broadcast_events: u32,
    pub(super) dispatch_timer_broadcast_time_us: u64,
    pub(super) dispatch_timer_broadcast_layers_visited: u32,
    pub(super) dispatch_timer_broadcast_rebuild_visible_layers_time_us: u64,
    pub(super) dispatch_timer_broadcast_loop_time_us: u64,
    pub(super) dispatch_timer_slowest_event_time_us: u64,
    pub(super) dispatch_timer_slowest_token: Option<u64>,
    pub(super) dispatch_timer_slowest_was_broadcast: bool,
    pub(super) dispatch_other_events: u32,
    pub(super) dispatch_other_event_time_us: u64,
    pub(super) hit_test_time_us: u64,
    pub(super) dispatch_hover_update_time_us: u64,
    pub(super) dispatch_scroll_handle_invalidation_time_us: u64,
    pub(super) dispatch_active_layers_time_us: u64,
    pub(super) dispatch_input_context_time_us: u64,
    pub(super) dispatch_event_chain_build_time_us: u64,
    pub(super) dispatch_widget_capture_time_us: u64,
    pub(super) dispatch_widget_bubble_time_us: u64,
    pub(super) dispatch_cursor_query_time_us: u64,
    pub(super) dispatch_pointer_move_layer_observers_time_us: u64,
    pub(super) dispatch_synth_hover_observer_time_us: u64,
    pub(super) dispatch_cursor_effect_time_us: u64,
    pub(super) dispatch_post_dispatch_snapshot_time_us: u64,
    pub(super) dispatch_events: u32,
    pub(super) hit_test_queries: u32,
    pub(super) hit_test_bounds_tree_queries: u32,
    pub(super) hit_test_bounds_tree_disabled: u32,
    pub(super) hit_test_bounds_tree_misses: u32,
    pub(super) hit_test_bounds_tree_hits: u32,
    pub(super) hit_test_bounds_tree_candidate_rejected: u32,
    pub(super) hit_test_cached_path_time_us: u64,
    pub(super) hit_test_bounds_tree_query_time_us: u64,
    pub(super) hit_test_candidate_self_only_time_us: u64,
    pub(super) hit_test_fallback_traversal_time_us: u64,
    pub(super) total_time_us: u64,
    pub(super) layout_nodes_performed: u32,
    pub(super) paint_nodes_performed: u32,
    pub(super) paint_cache_misses: u32,
    pub(super) paint_cache_replay_time_us: u64,
    pub(super) paint_cache_bounds_translate_time_us: u64,
    pub(super) paint_cache_bounds_translated_nodes: u32,
    pub(super) renderer_tick_id: u64,
    pub(super) renderer_frame_id: u64,
    pub(super) renderer_encode_scene_us: u64,
    pub(super) renderer_ensure_pipelines_us: u64,
    pub(super) renderer_plan_compile_us: u64,
    pub(super) renderer_upload_us: u64,
    pub(super) renderer_record_passes_us: u64,
    pub(super) renderer_encoder_finish_us: u64,
    pub(super) renderer_prepare_text_us: u64,
    pub(super) renderer_prepare_svg_us: u64,
    pub(super) renderer_svg_upload_bytes: u64,
    pub(super) renderer_image_upload_bytes: u64,

    pub(super) renderer_render_target_updates_ingest_unknown: u64,
    pub(super) renderer_render_target_updates_ingest_owned: u64,
    pub(super) renderer_render_target_updates_ingest_external_zero_copy: u64,
    pub(super) renderer_render_target_updates_ingest_gpu_copy: u64,
    pub(super) renderer_render_target_updates_ingest_cpu_upload: u64,
    pub(super) renderer_render_target_updates_requested_ingest_unknown: u64,
    pub(super) renderer_render_target_updates_requested_ingest_owned: u64,
    pub(super) renderer_render_target_updates_requested_ingest_external_zero_copy: u64,
    pub(super) renderer_render_target_updates_requested_ingest_gpu_copy: u64,
    pub(super) renderer_render_target_updates_requested_ingest_cpu_upload: u64,
    pub(super) renderer_render_target_updates_ingest_fallbacks: u64,

    pub(super) renderer_viewport_draw_calls: u64,
    pub(super) renderer_viewport_draw_calls_ingest_unknown: u64,
    pub(super) renderer_viewport_draw_calls_ingest_owned: u64,
    pub(super) renderer_viewport_draw_calls_ingest_external_zero_copy: u64,
    pub(super) renderer_viewport_draw_calls_ingest_gpu_copy: u64,
    pub(super) renderer_viewport_draw_calls_ingest_cpu_upload: u64,
    pub(super) renderer_svg_raster_budget_bytes: u64,
    pub(super) renderer_svg_rasters_live: u64,
    pub(super) renderer_svg_standalone_bytes_live: u64,
    pub(super) renderer_svg_mask_atlas_pages_live: u64,
    pub(super) renderer_svg_mask_atlas_bytes_live: u64,
    pub(super) renderer_svg_mask_atlas_used_px: u64,
    pub(super) renderer_svg_mask_atlas_capacity_px: u64,
    pub(super) renderer_svg_raster_cache_hits: u64,
    pub(super) renderer_svg_raster_cache_misses: u64,
    pub(super) renderer_svg_raster_budget_evictions: u64,
    pub(super) renderer_svg_mask_atlas_page_evictions: u64,
    pub(super) renderer_svg_mask_atlas_entries_evicted: u64,
    pub(super) renderer_text_atlas_upload_bytes: u64,
    pub(super) renderer_text_atlas_evicted_pages: u64,
    pub(super) renderer_intermediate_budget_bytes: u64,
    pub(super) renderer_intermediate_full_target_bytes: u64,
    pub(super) renderer_render_plan_effect_chain_budget_samples: u64,
    pub(super) renderer_render_plan_effect_chain_effective_budget_min_bytes: u64,
    pub(super) renderer_render_plan_effect_chain_effective_budget_max_bytes: u64,
    pub(super) renderer_render_plan_effect_chain_other_live_max_bytes: u64,
    pub(super) renderer_render_plan_custom_effect_chain_budget_samples: u64,
    pub(super) renderer_render_plan_custom_effect_chain_effective_budget_min_bytes: u64,
    pub(super) renderer_render_plan_custom_effect_chain_effective_budget_max_bytes: u64,
    pub(super) renderer_render_plan_custom_effect_chain_other_live_max_bytes: u64,
    pub(super) renderer_render_plan_custom_effect_chain_base_required_max_bytes: u64,
    pub(super) renderer_render_plan_custom_effect_chain_optional_required_max_bytes: u64,
    pub(super) renderer_render_plan_custom_effect_chain_base_required_full_targets_max: u64,
    pub(super) renderer_render_plan_custom_effect_chain_optional_mask_max_bytes: u64,
    pub(super) renderer_render_plan_custom_effect_chain_optional_pyramid_max_bytes: u64,
    pub(super) renderer_intermediate_in_use_bytes: u64,
    pub(super) renderer_intermediate_peak_in_use_bytes: u64,
    pub(super) renderer_intermediate_release_targets: u64,
    pub(super) renderer_intermediate_pool_allocations: u64,
    pub(super) renderer_intermediate_pool_reuses: u64,
    pub(super) renderer_intermediate_pool_releases: u64,
    pub(super) renderer_intermediate_pool_evictions: u64,
    pub(super) renderer_intermediate_pool_free_bytes: u64,
    pub(super) renderer_intermediate_pool_free_textures: u64,
    pub(super) renderer_draw_calls: u64,
    pub(super) renderer_pipeline_switches: u64,
    pub(super) renderer_bind_group_switches: u64,
    pub(super) renderer_scissor_sets: u64,
    pub(super) renderer_scene_encoding_cache_misses: u64,
    pub(super) renderer_material_quad_ops: u64,
    pub(super) renderer_material_sampled_quad_ops: u64,
    pub(super) renderer_material_distinct: u64,
    pub(super) renderer_material_unknown_ids: u64,
    pub(super) renderer_material_degraded_due_to_budget: u64,
    pub(super) renderer_custom_effect_v1_steps_requested: u64,
    pub(super) renderer_custom_effect_v1_passes_emitted: u64,
    pub(super) renderer_custom_effect_v2_steps_requested: u64,
    pub(super) renderer_custom_effect_v2_passes_emitted: u64,
    pub(super) renderer_custom_effect_v3_steps_requested: u64,
    pub(super) renderer_custom_effect_v3_passes_emitted: u64,
    pub(super) renderer_custom_effect_v3_sources_raw_requested: u64,
    pub(super) renderer_custom_effect_v3_sources_raw_distinct: u64,
    pub(super) renderer_custom_effect_v3_sources_raw_aliased_to_src: u64,
    pub(super) renderer_custom_effect_v3_sources_pyramid_requested: u64,
    pub(super) renderer_custom_effect_v3_sources_pyramid_applied_levels_ge2: u64,
    pub(super) renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_zero: u64,
    pub(super) renderer_custom_effect_v3_sources_pyramid_degraded_to_one_budget_insufficient: u64,
    pub(super) renderer_backdrop_source_groups_requested: u64,
    pub(super) renderer_backdrop_source_groups_applied_raw: u64,
    pub(super) renderer_backdrop_source_groups_raw_degraded_budget_zero: u64,
    pub(super) renderer_backdrop_source_groups_raw_degraded_budget_insufficient: u64,
    pub(super) renderer_backdrop_source_groups_raw_degraded_target_exhausted: u64,
    pub(super) renderer_backdrop_source_groups_pyramid_requested: u64,
    pub(super) renderer_backdrop_source_groups_pyramid_applied_levels_ge2: u64,
    pub(super) renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_zero: u64,
    pub(super) renderer_backdrop_source_groups_pyramid_degraded_to_one_budget_insufficient: u64,
    pub(super) renderer_backdrop_source_groups_pyramid_skipped_raw_unavailable: u64,
    pub(super) layout_engine_solves: u64,
    pub(super) layout_engine_solve_time_us: u64,
    pub(super) changed_models: u32,
    pub(super) changed_globals: u32,
    pub(super) changed_global_types_sample: Vec<String>,
    pub(super) propagated_model_change_models: u32,
    pub(super) propagated_model_change_observation_edges: u32,
    pub(super) propagated_model_change_unobserved_models: u32,
    pub(super) propagated_global_change_globals: u32,
    pub(super) propagated_global_change_observation_edges: u32,
    pub(super) propagated_global_change_unobserved_globals: u32,
    pub(super) invalidation_walk_calls: u32,
    pub(super) invalidation_walk_nodes: u32,
    pub(super) model_change_invalidation_roots: u32,
    pub(super) global_change_invalidation_roots: u32,
    pub(super) invalidation_walk_calls_model_change: u32,
    pub(super) invalidation_walk_nodes_model_change: u32,
    pub(super) invalidation_walk_calls_global_change: u32,
    pub(super) invalidation_walk_nodes_global_change: u32,
    pub(super) invalidation_walk_calls_hover: u32,
    pub(super) invalidation_walk_nodes_hover: u32,
    pub(super) invalidation_walk_calls_focus: u32,
    pub(super) invalidation_walk_nodes_focus: u32,
    pub(super) invalidation_walk_calls_other: u32,
    pub(super) invalidation_walk_nodes_other: u32,
    pub(super) top_invalidation_walks: Vec<BundleStatsInvalidationWalk>,
    pub(super) hover_pressable_target_changes: u32,
    pub(super) hover_hover_region_target_changes: u32,
    pub(super) hover_declarative_instance_changes: u32,
    pub(super) hover_declarative_hit_test_invalidations: u32,
    pub(super) hover_declarative_layout_invalidations: u32,
    pub(super) hover_declarative_paint_invalidations: u32,
    pub(super) top_hover_declarative_invalidations:
        Vec<BundleStatsHoverDeclarativeInvalidationHotspot>,
    pub(super) cache_roots: u32,
    pub(super) cache_roots_reused: u32,
    pub(super) cache_roots_contained_relayout: u32,
    pub(super) cache_replayed_ops: u64,
    pub(super) view_cache_contained_relayouts: u32,
    pub(super) view_cache_roots_total: u32,
    pub(super) view_cache_roots_reused: u32,
    pub(super) view_cache_roots_first_mount: u32,
    pub(super) view_cache_roots_node_recreated: u32,
    pub(super) view_cache_roots_cache_key_mismatch: u32,
    pub(super) view_cache_roots_not_marked_reuse_root: u32,
    pub(super) view_cache_roots_needs_rerender: u32,
    pub(super) view_cache_roots_layout_invalidated: u32,
    pub(super) view_cache_roots_manual: u32,
    pub(super) set_children_barrier_writes: u32,
    pub(super) barrier_relayouts_scheduled: u32,
    pub(super) barrier_relayouts_performed: u32,
    pub(super) virtual_list_visible_range_checks: u32,
    pub(super) virtual_list_visible_range_refreshes: u32,
    pub(super) top_cache_roots: Vec<BundleStatsCacheRoot>,
    pub(super) top_contained_relayout_cache_roots: Vec<BundleStatsCacheRoot>,
    pub(super) top_layout_engine_solves: Vec<BundleStatsLayoutEngineSolve>,
    pub(super) layout_hotspots: Vec<BundleStatsLayoutHotspot>,
    pub(super) widget_measure_hotspots: Vec<BundleStatsWidgetMeasureHotspot>,
    pub(super) paint_widget_hotspots: Vec<BundleStatsPaintWidgetHotspot>,
    pub(super) paint_text_prepare_hotspots: Vec<BundleStatsPaintTextPrepareHotspot>,
    pub(super) model_change_hotspots: Vec<BundleStatsModelChangeHotspot>,
    pub(super) model_change_unobserved: Vec<BundleStatsModelChangeUnobserved>,
    pub(super) global_change_hotspots: Vec<BundleStatsGlobalChangeHotspot>,
    pub(super) global_change_unobserved: Vec<BundleStatsGlobalChangeUnobserved>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) element_path: Option<String>,
    pub(super) widget_type: Option<String>,
    pub(super) layout_time_us: u64,
    pub(super) inclusive_time_us: u64,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsWidgetMeasureHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) element_path: Option<String>,
    pub(super) widget_type: Option<String>,
    pub(super) measure_time_us: u64,
    pub(super) inclusive_time_us: u64,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsPaintWidgetHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) widget_type: Option<String>,
    pub(super) paint_time_us: u64,
    pub(super) inclusive_time_us: u64,
    pub(super) inclusive_scene_ops_delta: u32,
    pub(super) exclusive_scene_ops_delta: u32,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsPaintTextPrepareHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) prepare_time_us: u64,
    pub(super) text_len: u32,
    pub(super) max_width: Option<f32>,
    pub(super) wrap: Option<String>,
    pub(super) overflow: Option<String>,
    pub(super) scale_factor: Option<f32>,
    pub(super) reasons_mask: u16,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsHoverDeclarativeInvalidationHotspot {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) hit_test: u32,
    pub(super) layout: u32,
    pub(super) paint: u32,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsWorstHoverLayout {
    window: u64,
    tick_id: u64,
    frame_id: u64,
    hover_declarative_layout_invalidations: u32,
    hotspots: Vec<BundleStatsHoverDeclarativeInvalidationHotspot>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsInvalidationWalk {
    pub(super) root_node: u64,
    pub(super) root_element: Option<u64>,
    pub(super) root_element_path: Option<String>,
    pub(super) kind: Option<String>,
    pub(super) source: Option<String>,
    pub(super) detail: Option<String>,
    pub(super) walked_nodes: u32,
    pub(super) truncated_at: Option<u64>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsCacheRoot {
    pub(super) root_node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_path: Option<String>,
    pub(super) reused: bool,
    pub(super) contained_layout: bool,
    pub(super) contained_relayout_in_frame: bool,
    pub(super) paint_replayed_ops: u32,
    pub(super) reuse_reason: Option<String>,
    pub(super) root_in_semantics: Option<bool>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineSolve {
    pub(super) root_node: u64,
    pub(super) root_element: Option<u64>,
    pub(super) root_element_kind: Option<String>,
    pub(super) root_element_path: Option<String>,
    pub(super) solve_time_us: u64,
    pub(super) measure_calls: u64,
    pub(super) measure_cache_hits: u64,
    pub(super) measure_time_us: u64,
    pub(super) top_measures: Vec<BundleStatsLayoutEngineMeasureHotspot>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineMeasureHotspot {
    pub(super) node: u64,
    pub(super) measure_time_us: u64,
    pub(super) calls: u64,
    pub(super) cache_hits: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) top_children: Vec<BundleStatsLayoutEngineMeasureChildHotspot>,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineMeasureChildHotspot {
    pub(super) child: u64,
    pub(super) measure_time_us: u64,
    pub(super) calls: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsModelChangeHotspot {
    model: u64,
    observation_edges: u32,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsModelChangeUnobserved {
    model: u64,
    created_type: Option<String>,
    created_at: Option<String>,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsGlobalChangeHotspot {
    type_name: String,
    observation_edges: u32,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsGlobalChangeUnobserved {
    type_name: String,
    changed_at: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsGlobalTypeHotspot {
    type_name: String,
    count: u64,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsModelSourceHotspot {
    source: String,
    count: u64,
}

impl BundleStatsReport {
    pub(crate) fn derived_from_frames_index(&self) -> bool {
        self.derived_from_frames_index
    }

    pub(super) fn print_human_brief(&self, bundle_path: &Path) {
        println!("bundle: {}", bundle_path.display());
        if self.derived_from_frames_index {
            println!(
                "note: derived from frames.index.json (tail-limited); some counters/percentiles may be missing or zero"
            );
        }
        println!(
            "windows={} snapshots={} considered={} warmup_skipped={} model_changes={} global_changes={} propagated_model_changes={} propagated_global_changes={}",
            self.windows,
            self.snapshots,
            self.snapshots_considered,
            self.snapshots_skipped_warmup,
            self.snapshots_with_model_changes,
            self.snapshots_with_global_changes,
            self.snapshots_with_propagated_model_changes,
            self.snapshots_with_propagated_global_changes
        );
        if self.warmup_frames > 0 {
            println!("warmup_frames={}", self.warmup_frames);
        }
        println!("sort={}", self.sort.as_str());
        println!(
            "time sum (us): total={} layout={} prepaint={} paint={}",
            self.sum_total_time_us,
            self.sum_layout_time_us,
            self.sum_prepaint_time_us,
            self.sum_paint_time_us
        );
        if self.derived_from_frames_index {
            println!(
                "time p50/p95 (us): total={}/{} layout={}/{} prepaint={}/{} paint={}/{}",
                self.p50_total_time_us,
                self.p95_total_time_us,
                self.p50_layout_time_us,
                self.p95_layout_time_us,
                self.p50_prepaint_time_us,
                self.p95_prepaint_time_us,
                self.p50_paint_time_us,
                self.p95_paint_time_us
            );
            println!(
                "invalidation sum: walk.calls={} walk.nodes={}",
                self.sum_invalidation_walk_calls, self.sum_invalidation_walk_nodes
            );
        } else {
            println!(
                "time p50/p95 (us): total={}/{} cpu_time={}/{} layout={}/{} prepaint={}/{} paint={}/{} dispatch={}/{} hit_test={}/{}",
                self.p50_total_time_us,
                self.p95_total_time_us,
                self.p50_ui_thread_cpu_time_us,
                self.p95_ui_thread_cpu_time_us,
                self.p50_layout_time_us,
                self.p95_layout_time_us,
                self.p50_prepaint_time_us,
                self.p95_prepaint_time_us,
                self.p50_paint_time_us,
                self.p95_paint_time_us,
                self.p50_dispatch_time_us,
                self.p95_dispatch_time_us,
                self.p50_hit_test_time_us,
                self.p95_hit_test_time_us
            );
            println!(
                "hot p50/p95 (us): layout.engine_solve={}/{} paint.widget={}/{} paint.text_prepare={}/{}",
                self.p50_layout_engine_solve_time_us,
                self.p95_layout_engine_solve_time_us,
                self.p50_paint_widget_time_us,
                self.p95_paint_widget_time_us,
                self.p50_paint_text_prepare_time_us,
                self.p95_paint_text_prepare_time_us
            );
        }
        if self.p95_renderer_encode_scene_us > 0
            || self.p95_renderer_upload_us > 0
            || self.p95_renderer_record_passes_us > 0
            || self.p95_renderer_encoder_finish_us > 0
            || self.p95_renderer_prepare_text_us > 0
            || self.p95_renderer_prepare_svg_us > 0
            || self.max_renderer_encode_scene_us > 0
            || self.max_renderer_upload_us > 0
            || self.max_renderer_record_passes_us > 0
            || self.max_renderer_encoder_finish_us > 0
            || self.max_renderer_prepare_text_us > 0
            || self.max_renderer_prepare_svg_us > 0
        {
            println!(
                "renderer p95/max (us): upload={}/{} record={}/{} finish={}/{} encode={}/{} text={}/{} svg={}/{}",
                self.p95_renderer_upload_us,
                self.max_renderer_upload_us,
                self.p95_renderer_record_passes_us,
                self.max_renderer_record_passes_us,
                self.p95_renderer_encoder_finish_us,
                self.max_renderer_encoder_finish_us,
                self.p95_renderer_encode_scene_us,
                self.max_renderer_encode_scene_us,
                self.p95_renderer_prepare_text_us,
                self.max_renderer_prepare_text_us,
                self.p95_renderer_prepare_svg_us,
                self.max_renderer_prepare_svg_us,
            );
        }
        if self.pointer_move_frames_present || self.pointer_move_frames_considered > 0 {
            let mode = if self.pointer_move_frames_present {
                "pointer_move"
            } else {
                "dispatch_frames_fallback"
            };
            println!(
                "derived({mode}) frames_considered={} max.us(dispatch/hit_test)={}/{} dispatch_at=window:{}/tick:{}/frame:{} hit_test_at=window:{}/tick:{}/frame:{} snapshots_with_global_changes={}",
                self.pointer_move_frames_considered,
                self.pointer_move_max_dispatch_time_us,
                self.pointer_move_max_hit_test_time_us,
                self.pointer_move_max_dispatch_window,
                self.pointer_move_max_dispatch_tick_id,
                self.pointer_move_max_dispatch_frame_id,
                self.pointer_move_max_hit_test_window,
                self.pointer_move_max_hit_test_tick_id,
                self.pointer_move_max_hit_test_frame_id,
                self.pointer_move_snapshots_with_global_changes
            );
        }

        if self.top.is_empty() {
            return;
        }

        println!("top (sort={}):", self.sort.as_str());
        for row in &self.top {
            let ts = row
                .timestamp_unix_ms
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string());
            let mut line = format!(
                "  window={} tick={} frame={} ts={} cpu.us={} cpu.cycles={} time.us(total/layout/prepaint/paint)={}/{}/{}/{} layout.solve_us={} paint.cache_misses={} layout.nodes={} paint.nodes={} paint.elem_bounds_us={} paint.elem_bounds_calls={} cache_roots={} cache.reused={} cache.replayed_ops={} cache.replay_us={} cache.translate_us={} cache.translate_nodes={} contained_relayouts={} cache.contained_relayout_roots={} barrier(set_children/scheduled/performed)={}/{}/{} vlist(range_checks/refreshes)={}/{} inv.calls={} inv.nodes={} by_src.calls(hover/focus/other)={}/{}/{} by_src.nodes(hover/focus/other)={}/{}/{} hover.decl_inv(layout/hit/paint)={}/{}/{} roots.model={} roots.global={} changed.models={} changed.globals={} propagated.models={} propagated.edges={} unobs.models={} propagated.globals={} propagated.global_edges={} unobs.globals={}",
                row.window,
                row.tick_id,
                row.frame_id,
                ts,
                row.ui_thread_cpu_time_us,
                row.ui_thread_cpu_cycle_time_delta_cycles,
                row.total_time_us,
                row.layout_time_us,
                row.prepaint_time_us,
                row.paint_time_us,
                row.layout_engine_solve_time_us,
                row.paint_cache_misses,
                row.layout_nodes_performed,
                row.paint_nodes_performed,
                row.paint_record_visual_bounds_time_us,
                row.paint_record_visual_bounds_calls,
                row.cache_roots,
                row.cache_roots_reused,
                row.cache_replayed_ops,
                row.paint_cache_replay_time_us,
                row.paint_cache_bounds_translate_time_us,
                row.paint_cache_bounds_translated_nodes,
                row.view_cache_contained_relayouts,
                row.cache_roots_contained_relayout,
                row.set_children_barrier_writes,
                row.barrier_relayouts_scheduled,
                row.barrier_relayouts_performed,
                row.virtual_list_visible_range_checks,
                row.virtual_list_visible_range_refreshes,
                row.invalidation_walk_calls,
                row.invalidation_walk_nodes,
                row.invalidation_walk_calls_hover,
                row.invalidation_walk_calls_focus,
                row.invalidation_walk_calls_other,
                row.invalidation_walk_nodes_hover,
                row.invalidation_walk_nodes_focus,
                row.invalidation_walk_nodes_other,
                row.hover_declarative_layout_invalidations,
                row.hover_declarative_hit_test_invalidations,
                row.hover_declarative_paint_invalidations,
                row.model_change_invalidation_roots,
                row.global_change_invalidation_roots,
                row.changed_models,
                row.changed_globals,
                row.propagated_model_change_models,
                row.propagated_model_change_observation_edges,
                row.propagated_model_change_unobserved_models,
                row.propagated_global_change_globals,
                row.propagated_global_change_observation_edges,
                row.propagated_global_change_unobserved_globals
            );
            if row.renderer_encode_scene_us > 0
                || row.renderer_prepare_text_us > 0
                || row.renderer_prepare_svg_us > 0
                || row.renderer_upload_us > 0
                || row.renderer_record_passes_us > 0
            {
                line.push_str(&format!(
                    " renderer.us(encode/ensure/plan/upload/record/finish/svg/text)={}/{}/{}/{}/{}/{}/{}/{}",
                    row.renderer_encode_scene_us,
                    row.renderer_ensure_pipelines_us,
                    row.renderer_plan_compile_us,
                    row.renderer_upload_us,
                    row.renderer_record_passes_us,
                    row.renderer_encoder_finish_us,
                    row.renderer_prepare_svg_us,
                    row.renderer_prepare_text_us,
                ));
            }
            println!("{line}");
        }
    }

    pub(super) fn print_human(&self, bundle_path: &Path) {
        println!("bundle: {}", bundle_path.display());
        if self.derived_from_frames_index {
            println!(
                "note: derived from frames.index.json (tail-limited); some counters/percentiles may be missing or zero"
            );
        }
        println!(
            "windows={} snapshots={} considered={} warmup_skipped={} model_changes={} global_changes={} propagated_model_changes={} propagated_global_changes={}",
            self.windows,
            self.snapshots,
            self.snapshots_considered,
            self.snapshots_skipped_warmup,
            self.snapshots_with_model_changes,
            self.snapshots_with_global_changes,
            self.snapshots_with_propagated_model_changes,
            self.snapshots_with_propagated_global_changes
        );
        if self.warmup_frames > 0 {
            println!("warmup_frames={}", self.warmup_frames);
        }
        println!("sort={}", self.sort.as_str());
        println!(
            "time sum (us): total={} layout={} prepaint={} paint={}",
            self.sum_total_time_us,
            self.sum_layout_time_us,
            self.sum_prepaint_time_us,
            self.sum_paint_time_us
        );
        if self.derived_from_frames_index {
            println!(
                "time p50/p95 (us): total={}/{} layout={}/{} prepaint={}/{} paint={}/{}",
                self.p50_total_time_us,
                self.p95_total_time_us,
                self.p50_layout_time_us,
                self.p95_layout_time_us,
                self.p50_prepaint_time_us,
                self.p95_prepaint_time_us,
                self.p50_paint_time_us,
                self.p95_paint_time_us
            );
            println!(
                "invalidation sum: walk.calls={} walk.nodes={}",
                self.sum_invalidation_walk_calls, self.sum_invalidation_walk_nodes
            );
        } else {
            println!(
                "time p50/p95 (us): total={}/{} cpu_time={}/{} layout={}/{} prepaint={}/{} paint={}/{} dispatch={}/{} hit_test={}/{}",
                self.p50_total_time_us,
                self.p95_total_time_us,
                self.p50_ui_thread_cpu_time_us,
                self.p95_ui_thread_cpu_time_us,
                self.p50_layout_time_us,
                self.p95_layout_time_us,
                self.p50_prepaint_time_us,
                self.p95_prepaint_time_us,
                self.p50_paint_time_us,
                self.p95_paint_time_us,
                self.p50_dispatch_time_us,
                self.p95_dispatch_time_us,
                self.p50_hit_test_time_us,
                self.p95_hit_test_time_us
            );
        }
        if self.p50_ui_thread_cpu_cycle_time_delta_cycles > 0
            || self.p95_ui_thread_cpu_cycle_time_delta_cycles > 0
            || self.max_ui_thread_cpu_cycle_time_delta_cycles > 0
        {
            println!(
                "cpu cycles p50/p95/max: {}/{}/{}",
                self.p50_ui_thread_cpu_cycle_time_delta_cycles,
                self.p95_ui_thread_cpu_cycle_time_delta_cycles,
                self.max_ui_thread_cpu_cycle_time_delta_cycles
            );
        }
        if !self.derived_from_frames_index {
            println!(
                "hot p50/p95 (us): layout.engine_solve={}/{} paint.widget={}/{} paint.text_prepare={}/{}",
                self.p50_layout_engine_solve_time_us,
                self.p95_layout_engine_solve_time_us,
                self.p50_paint_widget_time_us,
                self.p95_paint_widget_time_us,
                self.p50_paint_text_prepare_time_us,
                self.p95_paint_text_prepare_time_us
            );
        }
        if self.p50_renderer_encode_scene_us > 0
            || self.p95_renderer_encode_scene_us > 0
            || self.p50_renderer_upload_us > 0
            || self.p95_renderer_upload_us > 0
            || self.p50_renderer_record_passes_us > 0
            || self.p95_renderer_record_passes_us > 0
        {
            println!(
                "renderer p50/p95 (us): encode={}/{} ensure={}/{} plan={}/{} upload={}/{} record={}/{} finish={}/{} svg={}/{} text={}/{}",
                self.p50_renderer_encode_scene_us,
                self.p95_renderer_encode_scene_us,
                self.p50_renderer_ensure_pipelines_us,
                self.p95_renderer_ensure_pipelines_us,
                self.p50_renderer_plan_compile_us,
                self.p95_renderer_plan_compile_us,
                self.p50_renderer_upload_us,
                self.p95_renderer_upload_us,
                self.p50_renderer_record_passes_us,
                self.p95_renderer_record_passes_us,
                self.p50_renderer_encoder_finish_us,
                self.p95_renderer_encoder_finish_us,
                self.p50_renderer_prepare_svg_us,
                self.p95_renderer_prepare_svg_us,
                self.p50_renderer_prepare_text_us,
                self.p95_renderer_prepare_text_us,
            );
        }
        println!(
            "layout breakdown p50/p95 (us): roots={}/{} request_build_roots={}/{} view_cache={}/{} collapse_obs={}/{} prepaint_after_layout={}/{}",
            self.p50_layout_roots_time_us,
            self.p95_layout_roots_time_us,
            self.p50_layout_request_build_roots_time_us,
            self.p95_layout_request_build_roots_time_us,
            self.p50_layout_view_cache_time_us,
            self.p95_layout_view_cache_time_us,
            self.p50_layout_collapse_layout_observations_time_us,
            self.p95_layout_collapse_layout_observations_time_us,
            self.p50_layout_prepaint_after_layout_time_us,
            self.p95_layout_prepaint_after_layout_time_us
        );
        println!(
            "paint breakdown p50/p95 (us): input_ctx={}/{} scroll_inv={}/{} collect_roots={}/{} text_snapshot={}/{} collapse={}/{}",
            self.p50_paint_input_context_time_us,
            self.p95_paint_input_context_time_us,
            self.p50_paint_scroll_handle_invalidation_time_us,
            self.p95_paint_scroll_handle_invalidation_time_us,
            self.p50_paint_collect_roots_time_us,
            self.p95_paint_collect_roots_time_us,
            self.p50_paint_publish_text_input_snapshot_time_us,
            self.p95_paint_publish_text_input_snapshot_time_us,
            self.p50_paint_collapse_observations_time_us,
            self.p95_paint_collapse_observations_time_us
        );
        if self.sum_layout_observation_record_time_us > 0
            || self.sum_layout_observation_record_models_items > 0
            || self.sum_layout_observation_record_globals_items > 0
            || self.max_layout_observation_record_time_us > 0
        {
            println!(
                "layout obs_record sum (us): time={} items(models/globals)={}/{}",
                self.sum_layout_observation_record_time_us,
                self.sum_layout_observation_record_models_items,
                self.sum_layout_observation_record_globals_items
            );
            println!(
                "layout obs_record max (us): time={} items(models/globals)={}/{}",
                self.max_layout_observation_record_time_us,
                self.max_layout_observation_record_models_items,
                self.max_layout_observation_record_globals_items
            );
        }
        println!(
            "time max (us): total={} layout={} prepaint={} paint={}",
            self.max_total_time_us,
            self.max_layout_time_us,
            self.max_prepaint_time_us,
            self.max_paint_time_us
        );
        if self.max_renderer_encode_scene_us > 0
            || self.max_renderer_upload_us > 0
            || self.max_renderer_record_passes_us > 0
        {
            println!(
                "renderer max (us): encode={} ensure={} plan={} upload={} record={} finish={} svg={} text={}",
                self.max_renderer_encode_scene_us,
                self.max_renderer_ensure_pipelines_us,
                self.max_renderer_plan_compile_us,
                self.max_renderer_upload_us,
                self.max_renderer_record_passes_us,
                self.max_renderer_encoder_finish_us,
                self.max_renderer_prepare_svg_us,
                self.max_renderer_prepare_text_us,
            );
        }
        println!(
            "cache roots sum: roots={} reused={} replayed_ops={}",
            self.sum_cache_roots, self.sum_cache_roots_reused, self.sum_cache_replayed_ops
        );
        println!(
            "invalidation sum: calls={} nodes={}",
            self.sum_invalidation_walk_calls, self.sum_invalidation_walk_nodes
        );
        println!(
            "invalidation max: calls={} nodes={}",
            self.max_invalidation_walk_calls, self.max_invalidation_walk_nodes
        );
        println!(
            "roots sum: model={} global={}",
            self.sum_model_change_invalidation_roots, self.sum_global_change_invalidation_roots
        );
        println!(
            "roots max: model={} global={}",
            self.max_model_change_invalidation_roots, self.max_global_change_invalidation_roots
        );
        if self.sum_hover_layout_invalidations > 0 || self.max_hover_layout_invalidations > 0 {
            println!(
                "hover decl layout invalidations: sum={} max_per_frame={} frames_with_hover_layout={}",
                self.sum_hover_layout_invalidations,
                self.max_hover_layout_invalidations,
                self.snapshots_with_hover_layout_invalidations
            );
        }

        if !self.global_type_hotspots.is_empty() {
            let items: Vec<String> = self
                .global_type_hotspots
                .iter()
                .map(|h| format!("{}={}", h.type_name, h.count))
                .collect();
            println!("changed_globals_top: {}", items.join(" | "));
        }
        if !self.model_source_hotspots.is_empty() {
            let items: Vec<String> = self
                .model_source_hotspots
                .iter()
                .map(|h| format!("{}={}", h.source, h.count))
                .collect();
            println!("changed_models_top: {}", items.join(" | "));
        }

        if self.pointer_move_frames_present || self.pointer_move_frames_considered > 0 {
            let mode = if self.pointer_move_frames_present {
                "pointer_move"
            } else {
                "dispatch_frames_fallback"
            };
            println!(
                "derived({mode}) frames_considered={} max.us(dispatch/hit_test)={}/{} dispatch_at=window:{}/tick:{}/frame:{} hit_test_at=window:{}/tick:{}/frame:{} snapshots_with_global_changes={}",
                self.pointer_move_frames_considered,
                self.pointer_move_max_dispatch_time_us,
                self.pointer_move_max_hit_test_time_us,
                self.pointer_move_max_dispatch_window,
                self.pointer_move_max_dispatch_tick_id,
                self.pointer_move_max_dispatch_frame_id,
                self.pointer_move_max_hit_test_window,
                self.pointer_move_max_hit_test_tick_id,
                self.pointer_move_max_hit_test_frame_id,
                self.pointer_move_snapshots_with_global_changes
            );
        }

        if self.top.is_empty() {
            return;
        }

        println!("top (sort={}):", self.sort.as_str());
        for row in &self.top {
            let ts = row
                .timestamp_unix_ms
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string());
            let mut line = format!(
                "  window={} tick={} frame={} ts={} cpu.us={} cpu.cycles={} time.us(total/layout/prepaint/paint)={}/{}/{}/{} layout.solve_us={} paint.cache_misses={} layout.nodes={} paint.nodes={} paint.elem_bounds_us={} paint.elem_bounds_calls={} cache_roots={} cache.reused={} cache.replayed_ops={} cache.replay_us={} cache.translate_us={} cache.translate_nodes={} contained_relayouts={} cache.contained_relayout_roots={} barrier(set_children/scheduled/performed)={}/{}/{} vlist(range_checks/refreshes)={}/{} inv.calls={} inv.nodes={} by_src.calls(hover/focus/other)={}/{}/{} by_src.nodes(hover/focus/other)={}/{}/{} hover.decl_inv(layout/hit/paint)={}/{}/{} roots.model={} roots.global={} changed.models={} changed.globals={} propagated.models={} propagated.edges={} unobs.models={} propagated.globals={} propagated.global_edges={} unobs.globals={}",
                row.window,
                row.tick_id,
                row.frame_id,
                ts,
                row.ui_thread_cpu_time_us,
                row.ui_thread_cpu_cycle_time_delta_cycles,
                row.total_time_us,
                row.layout_time_us,
                row.prepaint_time_us,
                row.paint_time_us,
                row.layout_engine_solve_time_us,
                row.paint_cache_misses,
                row.layout_nodes_performed,
                row.paint_nodes_performed,
                row.paint_record_visual_bounds_time_us,
                row.paint_record_visual_bounds_calls,
                row.cache_roots,
                row.cache_roots_reused,
                row.cache_replayed_ops,
                row.paint_cache_replay_time_us,
                row.paint_cache_bounds_translate_time_us,
                row.paint_cache_bounds_translated_nodes,
                row.view_cache_contained_relayouts,
                row.cache_roots_contained_relayout,
                row.set_children_barrier_writes,
                row.barrier_relayouts_scheduled,
                row.barrier_relayouts_performed,
                row.virtual_list_visible_range_checks,
                row.virtual_list_visible_range_refreshes,
                row.invalidation_walk_calls,
                row.invalidation_walk_nodes,
                row.invalidation_walk_calls_hover,
                row.invalidation_walk_calls_focus,
                row.invalidation_walk_calls_other,
                row.invalidation_walk_nodes_hover,
                row.invalidation_walk_nodes_focus,
                row.invalidation_walk_nodes_other,
                row.hover_declarative_layout_invalidations,
                row.hover_declarative_hit_test_invalidations,
                row.hover_declarative_paint_invalidations,
                row.model_change_invalidation_roots,
                row.global_change_invalidation_roots,
                row.changed_models,
                row.changed_globals,
                row.propagated_model_change_models,
                row.propagated_model_change_observation_edges,
                row.propagated_model_change_unobserved_models,
                row.propagated_global_change_globals,
                row.propagated_global_change_observation_edges,
                row.propagated_global_change_unobserved_globals
            );
            if row.renderer_encode_scene_us > 0
                || row.renderer_prepare_text_us > 0
                || row.renderer_prepare_svg_us > 0
                || row.renderer_upload_us > 0
                || row.renderer_record_passes_us > 0
            {
                line.push_str(&format!(
                    " renderer.us(encode/ensure/plan/upload/record/finish/svg/text)={}/{}/{}/{}/{}/{}/{}/{}",
                    row.renderer_encode_scene_us,
                    row.renderer_ensure_pipelines_us,
                    row.renderer_plan_compile_us,
                    row.renderer_upload_us,
                    row.renderer_record_passes_us,
                    row.renderer_encoder_finish_us,
                    row.renderer_prepare_svg_us,
                    row.renderer_prepare_text_us,
                ));
            }
            println!("{line}");
            if row.layout_observation_record_time_us > 0
                || row.layout_observation_record_models_items > 0
                || row.layout_observation_record_globals_items > 0
            {
                println!(
                    "    layout_obs_record.us(time)={} items(models/globals)={}/{}",
                    row.layout_observation_record_time_us,
                    row.layout_observation_record_models_items,
                    row.layout_observation_record_globals_items
                );
            }
            if row.layout_roots_time_us > 0
                || row.layout_request_build_roots_time_us > 0
                || row.layout_view_cache_time_us > 0
                || row.layout_collapse_layout_observations_time_us > 0
                || row.layout_prepaint_after_layout_time_us > 0
                || row.layout_expand_view_cache_invalidations_time_us > 0
            {
                println!(
                    "    layout_breakdown.us(roots/request_build_roots/view_cache/collapse_obs/prepaint_after_layout)={}/{}/{}/{}/{} view_cache_inv_us={}",
                    row.layout_roots_time_us,
                    row.layout_request_build_roots_time_us,
                    row.layout_view_cache_time_us,
                    row.layout_collapse_layout_observations_time_us,
                    row.layout_prepaint_after_layout_time_us,
                    row.layout_expand_view_cache_invalidations_time_us,
                );
            }
            if row.paint_input_context_time_us > 0
                || row.paint_scroll_handle_invalidation_time_us > 0
                || row.paint_collect_roots_time_us > 0
                || row.paint_publish_text_input_snapshot_time_us > 0
                || row.paint_collapse_observations_time_us > 0
            {
                println!(
                    "    paint_breakdown.us(input_ctx/scroll_inv/collect_roots/text_snapshot/collapse)={}/{}/{}/{}/{}",
                    row.paint_input_context_time_us,
                    row.paint_scroll_handle_invalidation_time_us,
                    row.paint_collect_roots_time_us,
                    row.paint_publish_text_input_snapshot_time_us,
                    row.paint_collapse_observations_time_us
                );
            }
            if row.paint_cache_key_time_us > 0
                || row.paint_cache_hit_check_time_us > 0
                || row.paint_widget_time_us > 0
                || row.paint_observation_record_time_us > 0
            {
                println!(
                    "    paint_node.us(cache_key/hit_check/widget/obs_record)={}/{}/{}/{}",
                    row.paint_cache_key_time_us,
                    row.paint_cache_hit_check_time_us,
                    row.paint_widget_time_us,
                    row.paint_observation_record_time_us
                );
            }
            if row.paint_host_widget_observed_models_time_us > 0
                || row.paint_host_widget_observed_globals_time_us > 0
                || row.paint_host_widget_instance_lookup_time_us > 0
            {
                println!(
                    "    paint_host_widget.us(models/globals/instance)={}/{}/{} items={}/{} calls={}",
                    row.paint_host_widget_observed_models_time_us,
                    row.paint_host_widget_observed_globals_time_us,
                    row.paint_host_widget_instance_lookup_time_us,
                    row.paint_host_widget_observed_models_items,
                    row.paint_host_widget_observed_globals_items,
                    row.paint_host_widget_instance_lookup_calls,
                );
            }
            if row.paint_text_prepare_time_us > 0 || row.paint_text_prepare_calls > 0 {
                println!(
                    "    paint_text_prepare.us(time/calls)={}/{}",
                    row.paint_text_prepare_time_us, row.paint_text_prepare_calls
                );
                let reasons = [
                    row.paint_text_prepare_reason_blob_missing,
                    row.paint_text_prepare_reason_scale_changed,
                    row.paint_text_prepare_reason_text_changed,
                    row.paint_text_prepare_reason_rich_changed,
                    row.paint_text_prepare_reason_style_changed,
                    row.paint_text_prepare_reason_wrap_changed,
                    row.paint_text_prepare_reason_overflow_changed,
                    row.paint_text_prepare_reason_width_changed,
                    row.paint_text_prepare_reason_font_stack_changed,
                ];
                if reasons.iter().any(|&v| v > 0) {
                    println!(
                        "    paint_text_prepare.reasons(blob/scale/text/rich/style/wrap/overflow/width/font)={}/{}/{}/{}/{}/{}/{}/{}/{}",
                        row.paint_text_prepare_reason_blob_missing,
                        row.paint_text_prepare_reason_scale_changed,
                        row.paint_text_prepare_reason_text_changed,
                        row.paint_text_prepare_reason_rich_changed,
                        row.paint_text_prepare_reason_style_changed,
                        row.paint_text_prepare_reason_wrap_changed,
                        row.paint_text_prepare_reason_overflow_changed,
                        row.paint_text_prepare_reason_width_changed,
                        row.paint_text_prepare_reason_font_stack_changed,
                    );
                }
            }
            if !row.paint_text_prepare_hotspots.is_empty() {
                let items: Vec<String> = row
                    .paint_text_prepare_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!(
                            "us={} node={} kind={} len={} max_width={} wrap={} overflow={} reasons={}",
                            h.prepare_time_us,
                            h.node,
                            h.element_kind.as_deref().unwrap_or("?"),
                            h.text_len,
                            h.max_width
                                .map(|v| format!("{:.1}", v))
                                .unwrap_or_else(|| "?".to_string()),
                            h.wrap.as_deref().unwrap_or("?"),
                            h.overflow.as_deref().unwrap_or("?"),
                            format_text_prepare_reasons(h.reasons_mask),
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = h.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        s
                    })
                    .collect();
                println!("    paint_text_prepare_hotspots: {}", items.join(" | "));
            }
            if !row.paint_widget_hotspots.is_empty() {
                let items: Vec<String> = row
                    .paint_widget_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!(
                            "us={} ops={}/{} node={} kind={} type={}",
                            h.paint_time_us,
                            h.exclusive_scene_ops_delta,
                            h.inclusive_scene_ops_delta,
                            h.node,
                            h.element_kind.as_deref().unwrap_or("?"),
                            h.widget_type.as_deref().unwrap_or("?"),
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = h.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        s
                    })
                    .collect();
                println!("    paint_widget_hotspots: {}", items.join(" | "));
            }
            if !row.top_invalidation_walks.is_empty() {
                let items: Vec<String> = row
                    .top_invalidation_walks
                    .iter()
                    .take(3)
                    .map(|w| {
                        let mut s = format!(
                            "nodes={} src={} kind={} root={}",
                            w.walked_nodes,
                            w.source.as_deref().unwrap_or("?"),
                            w.kind.as_deref().unwrap_or("?"),
                            w.root_node
                        );
                        if let Some(detail) = w.detail.as_deref()
                            && !detail.is_empty()
                        {
                            s.push_str(&format!(" detail={detail}"));
                        }
                        if let Some(test_id) = w.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={}", test_id));
                        }
                        if let Some(role) = w.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={}", role));
                        }
                        if let Some(el) = w.root_element {
                            s.push_str(&format!(" element={}", el));
                        }
                        if let Some(path) = w.root_element_path.as_deref()
                            && !path.is_empty()
                        {
                            s.push_str(&format!(" element_path={}", elide_middle(path, 120)));
                        }
                        if let Some(trunc) = w.truncated_at {
                            s.push_str(&format!(" trunc_at={}", trunc));
                        }
                        s
                    })
                    .collect();
                println!("    top_walks: {}", items.join(" | "));
            }
            if !row.top_cache_roots.is_empty() {
                let items: Vec<String> = row
                    .top_cache_roots
                    .iter()
                    .take(3)
                    .map(|c| {
                        let mut s = format!(
                            "ops={} reused={} root={} reason={}",
                            c.paint_replayed_ops,
                            c.reused,
                            c.root_node,
                            c.reuse_reason.as_deref().unwrap_or("?")
                        );
                        if let Some(test_id) = c.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = c.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = c.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = c.element_path.as_deref()
                            && !path.is_empty()
                        {
                            let path = compact_debug_path(path);
                            s.push_str(&format!(" path={path}"));
                        }
                        if let Some(in_sem) = c.root_in_semantics {
                            s.push_str(&format!(" root_in_semantics={in_sem}"));
                        }
                        s
                    })
                    .collect();
                println!("    top_cache_roots: {}", items.join(" | "));
            }
            if !row.top_contained_relayout_cache_roots.is_empty() {
                let items: Vec<String> = row
                    .top_contained_relayout_cache_roots
                    .iter()
                    .take(3)
                    .map(|c| {
                        let mut s = format!(
                            "ops={} reused={} root={} reason={}",
                            c.paint_replayed_ops,
                            c.reused,
                            c.root_node,
                            c.reuse_reason.as_deref().unwrap_or("?")
                        );
                        if let Some(test_id) = c.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = c.root_role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = c.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = c.element_path.as_deref()
                            && !path.is_empty()
                        {
                            let path = compact_debug_path(path);
                            s.push_str(&format!(" path={path}"));
                        }
                        if let Some(in_sem) = c.root_in_semantics {
                            s.push_str(&format!(" root_in_semantics={in_sem}"));
                        }
                        s
                    })
                    .collect();
                println!(
                    "    top_contained_relayout_cache_roots: {}",
                    items.join(" | ")
                );
            }
            if row.hover_declarative_layout_invalidations > 0
                && !row.top_hover_declarative_invalidations.is_empty()
            {
                let items: Vec<String> = row
                    .top_hover_declarative_invalidations
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!(
                            "layout={} hit={} paint={} node={}",
                            h.layout, h.hit_test, h.paint, h.node
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            s.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            s.push_str(&format!(" role={role}"));
                        }
                        if let Some(el) = h.element {
                            s.push_str(&format!(" element={el}"));
                        }
                        s
                    })
                    .collect();
                println!("    hover_layout_hotspots: {}", items.join(" | "));
            }
            if !row.top_layout_engine_solves.is_empty() {
                let items: Vec<String> = row
                    .top_layout_engine_solves
                    .iter()
                    .take(3)
                    .map(|s| {
                        let mut out = format!(
                            "us={} measure.us={} measure.calls={} hits={} root={}",
                            s.solve_time_us,
                            s.measure_time_us,
                            s.measure_calls,
                            s.measure_cache_hits,
                            s.root_node
                        );
                        if let Some(test_id) = s.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = s.root_role.as_deref()
                            && !role.is_empty()
                        {
                            out.push_str(&format!(" role={role}"));
                        }
                        if let Some(kind) = s.root_element_kind.as_deref()
                            && !kind.is_empty()
                        {
                            out.push_str(&format!(" root.kind={kind}"));
                        }
                        if let Some(el) = s.root_element {
                            out.push_str(&format!(" root.element={el}"));
                        }
                        if let Some(path) = s.root_element_path.as_deref()
                            && !path.is_empty()
                        {
                            let path = compact_debug_path(path);
                            out.push_str(&format!(" root.path={path}"));
                        }
                        if let Some(m) = s.top_measures.first()
                            && m.measure_time_us > 0
                            && m.node != 0
                        {
                            out.push_str(&format!(
                                " top_measure.us={} node={}",
                                m.measure_time_us, m.node
                            ));
                            if let Some(kind) = m.element_kind.as_deref()
                                && !kind.is_empty()
                            {
                                out.push_str(&format!(" kind={kind}"));
                            }
                            if let Some(el) = m.element {
                                out.push_str(&format!(" element={el}"));
                            }
                            if let Some(test_id) = m.test_id.as_deref()
                                && !test_id.is_empty()
                            {
                                out.push_str(&format!(" test_id={test_id}"));
                            }
                            if let Some(role) = m.role.as_deref()
                                && !role.is_empty()
                            {
                                out.push_str(&format!(" role={role}"));
                            }
                            if let Some(c) = m.top_children.first()
                                && c.measure_time_us > 0
                                && c.child != 0
                            {
                                out.push_str(&format!(
                                    " child.us={} child={}",
                                    c.measure_time_us, c.child
                                ));
                                if let Some(kind) = c.element_kind.as_deref()
                                    && !kind.is_empty()
                                {
                                    out.push_str(&format!(" child.kind={kind}"));
                                }
                                if let Some(el) = c.element {
                                    out.push_str(&format!(" child.element={el}"));
                                }
                                if let Some(test_id) = c.test_id.as_deref()
                                    && !test_id.is_empty()
                                {
                                    out.push_str(&format!(" child.test_id={test_id}"));
                                }
                                if let Some(role) = c.role.as_deref()
                                    && !role.is_empty()
                                {
                                    out.push_str(&format!(" child.role={role}"));
                                }
                            }
                        }
                        out
                    })
                    .collect();
                println!("    top_layout_engine_solves: {}", items.join(" | "));
            }
            if !row.layout_hotspots.is_empty() {
                let items: Vec<String> = row
                    .layout_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut out = format!(
                            "us={} incl.us={} node={}",
                            h.layout_time_us, h.inclusive_time_us, h.node
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            out.push_str(&format!(" role={role}"));
                        }
                        if let Some(widget) = h.widget_type.as_deref()
                            && !widget.is_empty()
                        {
                            out.push_str(&format!(" widget={widget}"));
                        }
                        if let Some(kind) = h.element_kind.as_deref()
                            && !kind.is_empty()
                        {
                            out.push_str(&format!(" kind={kind}"));
                        }
                        if let Some(el) = h.element {
                            out.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = h.element_path.as_deref()
                            && !path.is_empty()
                        {
                            let path = compact_debug_path(path);
                            out.push_str(&format!(" path={path}"));
                        }
                        out
                    })
                    .collect();
                println!("    layout_hotspots: {}", items.join(" | "));
            }
            if !row.widget_measure_hotspots.is_empty() {
                let items: Vec<String> = row
                    .widget_measure_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut out = format!(
                            "us={} incl.us={} node={}",
                            h.measure_time_us, h.inclusive_time_us, h.node
                        );
                        if let Some(test_id) = h.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = h.role.as_deref()
                            && !role.is_empty()
                        {
                            out.push_str(&format!(" role={role}"));
                        }
                        if let Some(widget) = h.widget_type.as_deref()
                            && !widget.is_empty()
                        {
                            out.push_str(&format!(" widget={widget}"));
                        }
                        if let Some(kind) = h.element_kind.as_deref()
                            && !kind.is_empty()
                        {
                            out.push_str(&format!(" kind={kind}"));
                        }
                        if let Some(el) = h.element {
                            out.push_str(&format!(" element={el}"));
                        }
                        if let Some(path) = h.element_path.as_deref()
                            && !path.is_empty()
                        {
                            let path = compact_debug_path(path);
                            out.push_str(&format!(" path={path}"));
                        }
                        out
                    })
                    .collect();
                println!("    widget_measure_hotspots: {}", items.join(" | "));
            }
            if !row.model_change_hotspots.is_empty() {
                let items: Vec<String> = row
                    .model_change_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!("{}={}", h.model, h.observation_edges);
                        if let Some(at) = h.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    hot_models: {}", items.join(" | "));
            }
            if !row.model_change_unobserved.is_empty() {
                let items: Vec<String> = row
                    .model_change_unobserved
                    .iter()
                    .take(3)
                    .map(|u| {
                        let mut s = format!("{}", u.model);
                        if let Some(ty) = u.created_type.as_deref() {
                            s.push_str(&format!("={}", ty));
                        }
                        if let Some(at) = u.created_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        if let Some(at) = u.changed_at.as_deref() {
                            s.push_str(&format!(" changed@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    unobs_models: {}", items.join(" | "));
            }
            if !row.global_change_hotspots.is_empty() {
                let items: Vec<String> = row
                    .global_change_hotspots
                    .iter()
                    .take(3)
                    .map(|h| {
                        let mut s = format!("{}={}", h.type_name, h.observation_edges);
                        if let Some(at) = h.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    hot_globals: {}", items.join(" | "));
            }
            if !row.global_change_unobserved.is_empty() {
                let items: Vec<String> = row
                    .global_change_unobserved
                    .iter()
                    .take(3)
                    .map(|u| {
                        let mut s = u.type_name.clone();
                        if let Some(at) = u.changed_at.as_deref() {
                            s.push_str(&format!("@{}", at));
                        }
                        s
                    })
                    .collect();
                println!("    unobs_globals: {}", items.join(" | "));
            }
            if !row.changed_global_types_sample.is_empty() {
                println!(
                    "    changed_globals: {}",
                    row.changed_global_types_sample.join(" | ")
                );
            }
        }
    }

    pub(super) fn to_json(&self) -> serde_json::Value {
        use serde_json::{Map, Value};

        fn avg_us(sum: u64, n: u32) -> u64 {
            if n == 0 {
                return 0;
            }
            sum / (n as u64)
        }

        fn pct(numer: u64, denom: u64) -> f64 {
            if denom == 0 {
                return 0.0;
            }
            (numer as f64) * 100.0 / (denom as f64)
        }

        let mut root = Map::new();
        root.insert("schema_version".to_string(), Value::from(1));
        root.insert(
            "derived_from_frames_index".to_string(),
            Value::from(self.derived_from_frames_index),
        );
        root.insert("sort".to_string(), Value::from(self.sort.as_str()));
        root.insert("warmup_frames".to_string(), Value::from(self.warmup_frames));
        root.insert("windows".to_string(), Value::from(self.windows));
        root.insert("snapshots".to_string(), Value::from(self.snapshots));
        root.insert(
            "snapshots_considered".to_string(),
            Value::from(self.snapshots_considered),
        );
        root.insert(
            "snapshots_skipped_warmup".to_string(),
            Value::from(self.snapshots_skipped_warmup),
        );
        root.insert(
            "snapshots_with_model_changes".to_string(),
            Value::from(self.snapshots_with_model_changes),
        );
        root.insert(
            "snapshots_with_global_changes".to_string(),
            Value::from(self.snapshots_with_global_changes),
        );
        root.insert(
            "snapshots_with_propagated_model_changes".to_string(),
            Value::from(self.snapshots_with_propagated_model_changes),
        );
        root.insert(
            "snapshots_with_propagated_global_changes".to_string(),
            Value::from(self.snapshots_with_propagated_global_changes),
        );
        root.insert(
            "snapshots_with_hover_layout_invalidations".to_string(),
            Value::from(self.snapshots_with_hover_layout_invalidations),
        );

        root.insert(
            "pointer_move".to_string(),
            serde_json::json!({
                "frames_present": self.pointer_move_frames_present,
                "frames_considered": self.pointer_move_frames_considered,
                "max_dispatch_time_us": self.pointer_move_max_dispatch_time_us,
                "max_dispatch_at": {
                    "window": self.pointer_move_max_dispatch_window,
                    "tick_id": self.pointer_move_max_dispatch_tick_id,
                    "frame_id": self.pointer_move_max_dispatch_frame_id,
                },
                "max_hit_test_time_us": self.pointer_move_max_hit_test_time_us,
                "max_hit_test_at": {
                    "window": self.pointer_move_max_hit_test_window,
                    "tick_id": self.pointer_move_max_hit_test_tick_id,
                    "frame_id": self.pointer_move_max_hit_test_frame_id,
                },
                "snapshots_with_global_changes": self.pointer_move_snapshots_with_global_changes,
            }),
        );

        let mut sum = Map::new();
        sum.insert(
            "layout_collect_roots_time_us".to_string(),
            Value::from(self.sum_layout_collect_roots_time_us),
        );
        sum.insert(
            "layout_invalidate_scroll_handle_bindings_time_us".to_string(),
            Value::from(self.sum_layout_invalidate_scroll_handle_bindings_time_us),
        );
        sum.insert(
            "layout_expand_view_cache_invalidations_time_us".to_string(),
            Value::from(self.sum_layout_expand_view_cache_invalidations_time_us),
        );
        sum.insert(
            "layout_request_build_roots_time_us".to_string(),
            Value::from(self.sum_layout_request_build_roots_time_us),
        );
        sum.insert(
            "layout_roots_time_us".to_string(),
            Value::from(self.sum_layout_roots_time_us),
        );
        sum.insert(
            "layout_collapse_layout_observations_time_us".to_string(),
            Value::from(self.sum_layout_collapse_layout_observations_time_us),
        );
        sum.insert(
            "layout_time_us".to_string(),
            Value::from(self.sum_layout_time_us),
        );
        sum.insert(
            "layout_view_cache_time_us".to_string(),
            Value::from(self.sum_layout_view_cache_time_us),
        );
        sum.insert(
            "layout_prepaint_after_layout_time_us".to_string(),
            Value::from(self.sum_layout_prepaint_after_layout_time_us),
        );
        sum.insert(
            "layout_observation_record_time_us".to_string(),
            Value::from(self.sum_layout_observation_record_time_us),
        );
        sum.insert(
            "layout_observation_record_models_items".to_string(),
            Value::from(self.sum_layout_observation_record_models_items),
        );
        sum.insert(
            "layout_observation_record_globals_items".to_string(),
            Value::from(self.sum_layout_observation_record_globals_items),
        );
        sum.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.sum_prepaint_time_us),
        );
        sum.insert(
            "paint_time_us".to_string(),
            Value::from(self.sum_paint_time_us),
        );
        sum.insert(
            "total_time_us".to_string(),
            Value::from(self.sum_total_time_us),
        );
        sum.insert(
            "ui_thread_cpu_time_us".to_string(),
            Value::from(self.sum_ui_thread_cpu_time_us),
        );
        sum.insert(
            "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
            Value::from(self.sum_ui_thread_cpu_cycle_time_delta_cycles),
        );
        sum.insert(
            "layout_engine_solve_time_us".to_string(),
            Value::from(self.sum_layout_engine_solve_time_us),
        );
        sum.insert("cache_roots".to_string(), Value::from(self.sum_cache_roots));
        sum.insert(
            "cache_roots_reused".to_string(),
            Value::from(self.sum_cache_roots_reused),
        );
        sum.insert(
            "cache_replayed_ops".to_string(),
            Value::from(self.sum_cache_replayed_ops),
        );
        sum.insert(
            "invalidation_walk_calls".to_string(),
            Value::from(self.sum_invalidation_walk_calls),
        );
        sum.insert(
            "invalidation_walk_nodes".to_string(),
            Value::from(self.sum_invalidation_walk_nodes),
        );
        sum.insert(
            "model_change_invalidation_roots".to_string(),
            Value::from(self.sum_model_change_invalidation_roots),
        );
        sum.insert(
            "global_change_invalidation_roots".to_string(),
            Value::from(self.sum_global_change_invalidation_roots),
        );
        sum.insert(
            "hover_layout_invalidations".to_string(),
            Value::from(self.sum_hover_layout_invalidations),
        );
        root.insert("sum".to_string(), Value::Object(sum));

        let mut max = Map::new();
        max.insert(
            "layout_collect_roots_time_us".to_string(),
            Value::from(self.max_layout_collect_roots_time_us),
        );
        max.insert(
            "layout_invalidate_scroll_handle_bindings_time_us".to_string(),
            Value::from(self.max_layout_invalidate_scroll_handle_bindings_time_us),
        );
        max.insert(
            "layout_expand_view_cache_invalidations_time_us".to_string(),
            Value::from(self.max_layout_expand_view_cache_invalidations_time_us),
        );
        max.insert(
            "layout_request_build_roots_time_us".to_string(),
            Value::from(self.max_layout_request_build_roots_time_us),
        );
        max.insert(
            "layout_roots_time_us".to_string(),
            Value::from(self.max_layout_roots_time_us),
        );
        max.insert(
            "layout_collapse_layout_observations_time_us".to_string(),
            Value::from(self.max_layout_collapse_layout_observations_time_us),
        );
        max.insert(
            "layout_time_us".to_string(),
            Value::from(self.max_layout_time_us),
        );
        max.insert(
            "layout_view_cache_time_us".to_string(),
            Value::from(self.max_layout_view_cache_time_us),
        );
        max.insert(
            "layout_prepaint_after_layout_time_us".to_string(),
            Value::from(self.max_layout_prepaint_after_layout_time_us),
        );
        max.insert(
            "layout_observation_record_time_us".to_string(),
            Value::from(self.max_layout_observation_record_time_us),
        );
        max.insert(
            "layout_observation_record_models_items".to_string(),
            Value::from(self.max_layout_observation_record_models_items),
        );
        max.insert(
            "layout_observation_record_globals_items".to_string(),
            Value::from(self.max_layout_observation_record_globals_items),
        );
        max.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.max_prepaint_time_us),
        );
        max.insert(
            "paint_time_us".to_string(),
            Value::from(self.max_paint_time_us),
        );
        max.insert(
            "total_time_us".to_string(),
            Value::from(self.max_total_time_us),
        );
        max.insert(
            "ui_thread_cpu_time_us".to_string(),
            Value::from(self.max_ui_thread_cpu_time_us),
        );
        max.insert(
            "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
            Value::from(self.max_ui_thread_cpu_cycle_time_delta_cycles),
        );
        max.insert(
            "layout_engine_solve_time_us".to_string(),
            Value::from(self.max_layout_engine_solve_time_us),
        );
        max.insert(
            "renderer_encode_scene_us".to_string(),
            Value::from(self.max_renderer_encode_scene_us),
        );
        max.insert(
            "renderer_ensure_pipelines_us".to_string(),
            Value::from(self.max_renderer_ensure_pipelines_us),
        );
        max.insert(
            "renderer_plan_compile_us".to_string(),
            Value::from(self.max_renderer_plan_compile_us),
        );
        max.insert(
            "renderer_upload_us".to_string(),
            Value::from(self.max_renderer_upload_us),
        );
        max.insert(
            "renderer_record_passes_us".to_string(),
            Value::from(self.max_renderer_record_passes_us),
        );
        max.insert(
            "renderer_encoder_finish_us".to_string(),
            Value::from(self.max_renderer_encoder_finish_us),
        );
        max.insert(
            "renderer_prepare_svg_us".to_string(),
            Value::from(self.max_renderer_prepare_svg_us),
        );
        max.insert(
            "renderer_prepare_text_us".to_string(),
            Value::from(self.max_renderer_prepare_text_us),
        );
        max.insert(
            "invalidation_walk_calls".to_string(),
            Value::from(self.max_invalidation_walk_calls),
        );
        max.insert(
            "invalidation_walk_nodes".to_string(),
            Value::from(self.max_invalidation_walk_nodes),
        );
        max.insert(
            "model_change_invalidation_roots".to_string(),
            Value::from(self.max_model_change_invalidation_roots),
        );
        max.insert(
            "global_change_invalidation_roots".to_string(),
            Value::from(self.max_global_change_invalidation_roots),
        );
        max.insert(
            "hover_layout_invalidations".to_string(),
            Value::from(self.max_hover_layout_invalidations),
        );
        root.insert("max".to_string(), Value::Object(max));

        let mut avg = Map::new();
        avg.insert(
            "layout_collect_roots_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_collect_roots_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_invalidate_scroll_handle_bindings_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_invalidate_scroll_handle_bindings_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_expand_view_cache_invalidations_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_expand_view_cache_invalidations_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_request_build_roots_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_request_build_roots_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_roots_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_roots_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_collapse_layout_observations_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_collapse_layout_observations_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_time_us".to_string(),
            Value::from(avg_us(self.sum_layout_time_us, self.snapshots_considered)),
        );
        avg.insert(
            "layout_view_cache_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_view_cache_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_prepaint_after_layout_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_prepaint_after_layout_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_observation_record_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_observation_record_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_observation_record_models_items".to_string(),
            Value::from(avg_us(
                self.sum_layout_observation_record_models_items,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_observation_record_globals_items".to_string(),
            Value::from(avg_us(
                self.sum_layout_observation_record_globals_items,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "prepaint_time_us".to_string(),
            Value::from(avg_us(self.sum_prepaint_time_us, self.snapshots_considered)),
        );
        avg.insert(
            "paint_time_us".to_string(),
            Value::from(avg_us(self.sum_paint_time_us, self.snapshots_considered)),
        );
        avg.insert(
            "total_time_us".to_string(),
            Value::from(avg_us(self.sum_total_time_us, self.snapshots_considered)),
        );
        avg.insert(
            "ui_thread_cpu_time_us".to_string(),
            Value::from(avg_us(
                self.sum_ui_thread_cpu_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
            Value::from(avg_us(
                self.sum_ui_thread_cpu_cycle_time_delta_cycles,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "layout_engine_solve_time_us".to_string(),
            Value::from(avg_us(
                self.sum_layout_engine_solve_time_us,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "cache_roots".to_string(),
            Value::from(avg_us(self.sum_cache_roots, self.snapshots_considered)),
        );
        avg.insert(
            "cache_roots_reused".to_string(),
            Value::from(avg_us(
                self.sum_cache_roots_reused,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "cache_replayed_ops".to_string(),
            Value::from(avg_us(
                self.sum_cache_replayed_ops,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "invalidation_walk_calls".to_string(),
            Value::from(avg_us(
                self.sum_invalidation_walk_calls,
                self.snapshots_considered,
            )),
        );
        avg.insert(
            "invalidation_walk_nodes".to_string(),
            Value::from(avg_us(
                self.sum_invalidation_walk_nodes,
                self.snapshots_considered,
            )),
        );
        root.insert("avg".to_string(), Value::Object(avg));

        let mut p50 = Map::new();
        p50.insert(
            "total_time_us".to_string(),
            Value::from(self.p50_total_time_us),
        );
        p50.insert(
            "ui_thread_cpu_time_us".to_string(),
            Value::from(self.p50_ui_thread_cpu_time_us),
        );
        p50.insert(
            "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
            Value::from(self.p50_ui_thread_cpu_cycle_time_delta_cycles),
        );
        p50.insert(
            "layout_time_us".to_string(),
            Value::from(self.p50_layout_time_us),
        );
        p50.insert(
            "layout_collect_roots_time_us".to_string(),
            Value::from(self.p50_layout_collect_roots_time_us),
        );
        p50.insert(
            "layout_request_build_roots_time_us".to_string(),
            Value::from(self.p50_layout_request_build_roots_time_us),
        );
        p50.insert(
            "layout_roots_time_us".to_string(),
            Value::from(self.p50_layout_roots_time_us),
        );
        p50.insert(
            "layout_view_cache_time_us".to_string(),
            Value::from(self.p50_layout_view_cache_time_us),
        );
        p50.insert(
            "layout_collapse_layout_observations_time_us".to_string(),
            Value::from(self.p50_layout_collapse_layout_observations_time_us),
        );
        p50.insert(
            "layout_prepaint_after_layout_time_us".to_string(),
            Value::from(self.p50_layout_prepaint_after_layout_time_us),
        );
        p50.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.p50_prepaint_time_us),
        );
        p50.insert(
            "paint_time_us".to_string(),
            Value::from(self.p50_paint_time_us),
        );
        p50.insert(
            "paint_input_context_time_us".to_string(),
            Value::from(self.p50_paint_input_context_time_us),
        );
        p50.insert(
            "paint_scroll_handle_invalidation_time_us".to_string(),
            Value::from(self.p50_paint_scroll_handle_invalidation_time_us),
        );
        p50.insert(
            "paint_collect_roots_time_us".to_string(),
            Value::from(self.p50_paint_collect_roots_time_us),
        );
        p50.insert(
            "paint_publish_text_input_snapshot_time_us".to_string(),
            Value::from(self.p50_paint_publish_text_input_snapshot_time_us),
        );
        p50.insert(
            "paint_collapse_observations_time_us".to_string(),
            Value::from(self.p50_paint_collapse_observations_time_us),
        );
        p50.insert(
            "layout_engine_solve_time_us".to_string(),
            Value::from(self.p50_layout_engine_solve_time_us),
        );
        p50.insert(
            "dispatch_time_us".to_string(),
            Value::from(self.p50_dispatch_time_us),
        );
        p50.insert(
            "hit_test_time_us".to_string(),
            Value::from(self.p50_hit_test_time_us),
        );
        p50.insert(
            "paint_widget_time_us".to_string(),
            Value::from(self.p50_paint_widget_time_us),
        );
        p50.insert(
            "paint_text_prepare_time_us".to_string(),
            Value::from(self.p50_paint_text_prepare_time_us),
        );
        p50.insert(
            "renderer_encode_scene_us".to_string(),
            Value::from(self.p50_renderer_encode_scene_us),
        );
        p50.insert(
            "renderer_ensure_pipelines_us".to_string(),
            Value::from(self.p50_renderer_ensure_pipelines_us),
        );
        p50.insert(
            "renderer_plan_compile_us".to_string(),
            Value::from(self.p50_renderer_plan_compile_us),
        );
        p50.insert(
            "renderer_upload_us".to_string(),
            Value::from(self.p50_renderer_upload_us),
        );
        p50.insert(
            "renderer_record_passes_us".to_string(),
            Value::from(self.p50_renderer_record_passes_us),
        );
        p50.insert(
            "renderer_encoder_finish_us".to_string(),
            Value::from(self.p50_renderer_encoder_finish_us),
        );
        p50.insert(
            "renderer_prepare_svg_us".to_string(),
            Value::from(self.p50_renderer_prepare_svg_us),
        );
        p50.insert(
            "renderer_prepare_text_us".to_string(),
            Value::from(self.p50_renderer_prepare_text_us),
        );
        root.insert("p50".to_string(), Value::Object(p50));

        let mut p95 = Map::new();
        p95.insert(
            "total_time_us".to_string(),
            Value::from(self.p95_total_time_us),
        );
        p95.insert(
            "ui_thread_cpu_time_us".to_string(),
            Value::from(self.p95_ui_thread_cpu_time_us),
        );
        p95.insert(
            "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
            Value::from(self.p95_ui_thread_cpu_cycle_time_delta_cycles),
        );
        p95.insert(
            "layout_time_us".to_string(),
            Value::from(self.p95_layout_time_us),
        );
        p95.insert(
            "layout_collect_roots_time_us".to_string(),
            Value::from(self.p95_layout_collect_roots_time_us),
        );
        p95.insert(
            "layout_request_build_roots_time_us".to_string(),
            Value::from(self.p95_layout_request_build_roots_time_us),
        );
        p95.insert(
            "layout_roots_time_us".to_string(),
            Value::from(self.p95_layout_roots_time_us),
        );
        p95.insert(
            "layout_view_cache_time_us".to_string(),
            Value::from(self.p95_layout_view_cache_time_us),
        );
        p95.insert(
            "layout_collapse_layout_observations_time_us".to_string(),
            Value::from(self.p95_layout_collapse_layout_observations_time_us),
        );
        p95.insert(
            "layout_prepaint_after_layout_time_us".to_string(),
            Value::from(self.p95_layout_prepaint_after_layout_time_us),
        );
        p95.insert(
            "prepaint_time_us".to_string(),
            Value::from(self.p95_prepaint_time_us),
        );
        p95.insert(
            "paint_time_us".to_string(),
            Value::from(self.p95_paint_time_us),
        );
        p95.insert(
            "paint_input_context_time_us".to_string(),
            Value::from(self.p95_paint_input_context_time_us),
        );
        p95.insert(
            "paint_scroll_handle_invalidation_time_us".to_string(),
            Value::from(self.p95_paint_scroll_handle_invalidation_time_us),
        );
        p95.insert(
            "paint_collect_roots_time_us".to_string(),
            Value::from(self.p95_paint_collect_roots_time_us),
        );
        p95.insert(
            "paint_publish_text_input_snapshot_time_us".to_string(),
            Value::from(self.p95_paint_publish_text_input_snapshot_time_us),
        );
        p95.insert(
            "paint_collapse_observations_time_us".to_string(),
            Value::from(self.p95_paint_collapse_observations_time_us),
        );
        p95.insert(
            "layout_engine_solve_time_us".to_string(),
            Value::from(self.p95_layout_engine_solve_time_us),
        );
        p95.insert(
            "dispatch_time_us".to_string(),
            Value::from(self.p95_dispatch_time_us),
        );
        p95.insert(
            "hit_test_time_us".to_string(),
            Value::from(self.p95_hit_test_time_us),
        );
        p95.insert(
            "paint_widget_time_us".to_string(),
            Value::from(self.p95_paint_widget_time_us),
        );
        p95.insert(
            "paint_text_prepare_time_us".to_string(),
            Value::from(self.p95_paint_text_prepare_time_us),
        );
        p95.insert(
            "renderer_encode_scene_us".to_string(),
            Value::from(self.p95_renderer_encode_scene_us),
        );
        p95.insert(
            "renderer_ensure_pipelines_us".to_string(),
            Value::from(self.p95_renderer_ensure_pipelines_us),
        );
        p95.insert(
            "renderer_plan_compile_us".to_string(),
            Value::from(self.p95_renderer_plan_compile_us),
        );
        p95.insert(
            "renderer_upload_us".to_string(),
            Value::from(self.p95_renderer_upload_us),
        );
        p95.insert(
            "renderer_record_passes_us".to_string(),
            Value::from(self.p95_renderer_record_passes_us),
        );
        p95.insert(
            "renderer_encoder_finish_us".to_string(),
            Value::from(self.p95_renderer_encoder_finish_us),
        );
        p95.insert(
            "renderer_prepare_svg_us".to_string(),
            Value::from(self.p95_renderer_prepare_svg_us),
        );
        p95.insert(
            "renderer_prepare_text_us".to_string(),
            Value::from(self.p95_renderer_prepare_text_us),
        );
        root.insert("p95".to_string(), Value::Object(p95));

        root.insert(
            "budget_pct".to_string(),
            serde_json::json!({
                "layout_of_total": pct(self.sum_layout_time_us, self.sum_total_time_us),
                "prepaint_of_total": pct(self.sum_prepaint_time_us, self.sum_total_time_us),
                "paint_of_total": pct(self.sum_paint_time_us, self.sum_total_time_us),
                "layout_obs_record_of_layout": pct(self.sum_layout_observation_record_time_us, self.sum_layout_time_us),
                "layout_obs_record_of_total": pct(self.sum_layout_observation_record_time_us, self.sum_total_time_us),
            }),
        );

        let global_type_hotspots = self
            .global_type_hotspots
            .iter()
            .map(|h| {
                let mut obj = Map::new();
                obj.insert("type_name".to_string(), Value::from(h.type_name.clone()));
                obj.insert("count".to_string(), Value::from(h.count));
                Value::Object(obj)
            })
            .collect::<Vec<_>>();
        root.insert(
            "global_type_hotspots".to_string(),
            Value::Array(global_type_hotspots),
        );
        let model_source_hotspots = self
            .model_source_hotspots
            .iter()
            .map(|h| {
                let mut obj = Map::new();
                obj.insert("source".to_string(), Value::from(h.source.clone()));
                obj.insert("count".to_string(), Value::from(h.count));
                Value::Object(obj)
            })
            .collect::<Vec<_>>();
        root.insert(
            "model_source_hotspots".to_string(),
            Value::Array(model_source_hotspots),
        );

        let top = self
            .top
            .iter()
            .map(|row| {
                let mut obj = Map::new();
                obj.insert("window".to_string(), Value::from(row.window));
                obj.insert("tick_id".to_string(), Value::from(row.tick_id));
                obj.insert("frame_id".to_string(), Value::from(row.frame_id));
                obj.insert(
                    "timestamp_unix_ms".to_string(),
                    row.timestamp_unix_ms
                        .map(Value::from)
                        .unwrap_or(Value::Null),
                );
                obj.insert(
                    "ui_thread_cpu_time_us".to_string(),
                    Value::from(row.ui_thread_cpu_time_us),
                );
                obj.insert(
                    "ui_thread_cpu_total_time_us".to_string(),
                    Value::from(row.ui_thread_cpu_total_time_us),
                );
                obj.insert(
                    "ui_thread_cpu_cycle_time_delta_cycles".to_string(),
                    Value::from(row.ui_thread_cpu_cycle_time_delta_cycles),
                );
                obj.insert(
                    "ui_thread_cpu_cycle_time_total_cycles".to_string(),
                    Value::from(row.ui_thread_cpu_cycle_time_total_cycles),
                );
                obj.insert(
                    "layout_time_us".to_string(),
                    Value::from(row.layout_time_us),
                );
                obj.insert(
                    "renderer_tick_id".to_string(),
                    Value::from(row.renderer_tick_id),
                );
                obj.insert(
                    "renderer_frame_id".to_string(),
                    Value::from(row.renderer_frame_id),
                );
                obj.insert(
                    "renderer_encode_scene_us".to_string(),
                    Value::from(row.renderer_encode_scene_us),
                );
                obj.insert(
                    "renderer_ensure_pipelines_us".to_string(),
                    Value::from(row.renderer_ensure_pipelines_us),
                );
                obj.insert(
                    "renderer_plan_compile_us".to_string(),
                    Value::from(row.renderer_plan_compile_us),
                );
                obj.insert(
                    "renderer_upload_us".to_string(),
                    Value::from(row.renderer_upload_us),
                );
                obj.insert(
                    "renderer_record_passes_us".to_string(),
                    Value::from(row.renderer_record_passes_us),
                );
                obj.insert(
                    "renderer_encoder_finish_us".to_string(),
                    Value::from(row.renderer_encoder_finish_us),
                );
                obj.insert(
                    "renderer_prepare_svg_us".to_string(),
                    Value::from(row.renderer_prepare_svg_us),
                );
                obj.insert(
                    "renderer_prepare_text_us".to_string(),
                    Value::from(row.renderer_prepare_text_us),
                );
                obj.insert(
                    "prepaint_time_us".to_string(),
                    Value::from(row.prepaint_time_us),
                );
                obj.insert("paint_time_us".to_string(), Value::from(row.paint_time_us));
                obj.insert(
                    "dispatch_time_us".to_string(),
                    Value::from(row.dispatch_time_us),
                );
                obj.insert(
                    "dispatch_pointer_events".to_string(),
                    Value::from(row.dispatch_pointer_events),
                );
                obj.insert(
                    "dispatch_pointer_event_time_us".to_string(),
                    Value::from(row.dispatch_pointer_event_time_us),
                );
                obj.insert(
                    "dispatch_timer_events".to_string(),
                    Value::from(row.dispatch_timer_events),
                );
                obj.insert(
                    "dispatch_timer_event_time_us".to_string(),
                    Value::from(row.dispatch_timer_event_time_us),
                );
                obj.insert(
                    "dispatch_timer_targeted_events".to_string(),
                    Value::from(row.dispatch_timer_targeted_events),
                );
                obj.insert(
                    "dispatch_timer_targeted_time_us".to_string(),
                    Value::from(row.dispatch_timer_targeted_time_us),
                );
                obj.insert(
                    "dispatch_timer_broadcast_events".to_string(),
                    Value::from(row.dispatch_timer_broadcast_events),
                );
                obj.insert(
                    "dispatch_timer_broadcast_time_us".to_string(),
                    Value::from(row.dispatch_timer_broadcast_time_us),
                );
                obj.insert(
                    "dispatch_timer_broadcast_layers_visited".to_string(),
                    Value::from(row.dispatch_timer_broadcast_layers_visited),
                );
                obj.insert(
                    "dispatch_timer_broadcast_rebuild_visible_layers_time_us".to_string(),
                    Value::from(row.dispatch_timer_broadcast_rebuild_visible_layers_time_us),
                );
                obj.insert(
                    "dispatch_timer_broadcast_loop_time_us".to_string(),
                    Value::from(row.dispatch_timer_broadcast_loop_time_us),
                );
                obj.insert(
                    "dispatch_timer_slowest_event_time_us".to_string(),
                    Value::from(row.dispatch_timer_slowest_event_time_us),
                );
                obj.insert(
                    "dispatch_timer_slowest_token".to_string(),
                    row.dispatch_timer_slowest_token
                        .map(Value::from)
                        .unwrap_or(Value::Null),
                );
                obj.insert(
                    "dispatch_timer_slowest_was_broadcast".to_string(),
                    Value::from(row.dispatch_timer_slowest_was_broadcast),
                );
                obj.insert(
                    "dispatch_other_events".to_string(),
                    Value::from(row.dispatch_other_events),
                );
                obj.insert(
                    "dispatch_other_event_time_us".to_string(),
                    Value::from(row.dispatch_other_event_time_us),
                );
                obj.insert(
                    "hit_test_time_us".to_string(),
                    Value::from(row.hit_test_time_us),
                );
                obj.insert(
                    "dispatch_hover_update_time_us".to_string(),
                    Value::from(row.dispatch_hover_update_time_us),
                );
                obj.insert(
                    "dispatch_scroll_handle_invalidation_time_us".to_string(),
                    Value::from(row.dispatch_scroll_handle_invalidation_time_us),
                );
                obj.insert(
                    "dispatch_active_layers_time_us".to_string(),
                    Value::from(row.dispatch_active_layers_time_us),
                );
                obj.insert(
                    "dispatch_input_context_time_us".to_string(),
                    Value::from(row.dispatch_input_context_time_us),
                );
                obj.insert(
                    "dispatch_event_chain_build_time_us".to_string(),
                    Value::from(row.dispatch_event_chain_build_time_us),
                );
                obj.insert(
                    "dispatch_widget_capture_time_us".to_string(),
                    Value::from(row.dispatch_widget_capture_time_us),
                );
                obj.insert(
                    "dispatch_widget_bubble_time_us".to_string(),
                    Value::from(row.dispatch_widget_bubble_time_us),
                );
                obj.insert(
                    "dispatch_cursor_query_time_us".to_string(),
                    Value::from(row.dispatch_cursor_query_time_us),
                );
                obj.insert(
                    "dispatch_pointer_move_layer_observers_time_us".to_string(),
                    Value::from(row.dispatch_pointer_move_layer_observers_time_us),
                );
                obj.insert(
                    "dispatch_synth_hover_observer_time_us".to_string(),
                    Value::from(row.dispatch_synth_hover_observer_time_us),
                );
                obj.insert(
                    "dispatch_cursor_effect_time_us".to_string(),
                    Value::from(row.dispatch_cursor_effect_time_us),
                );
                obj.insert(
                    "dispatch_post_dispatch_snapshot_time_us".to_string(),
                    Value::from(row.dispatch_post_dispatch_snapshot_time_us),
                );
                obj.insert(
                    "dispatch_events".to_string(),
                    Value::from(row.dispatch_events),
                );
                obj.insert(
                    "hit_test_queries".to_string(),
                    Value::from(row.hit_test_queries),
                );
                obj.insert(
                    "hit_test_bounds_tree_queries".to_string(),
                    Value::from(row.hit_test_bounds_tree_queries),
                );
                obj.insert(
                    "hit_test_bounds_tree_disabled".to_string(),
                    Value::from(row.hit_test_bounds_tree_disabled),
                );
                obj.insert(
                    "hit_test_bounds_tree_misses".to_string(),
                    Value::from(row.hit_test_bounds_tree_misses),
                );
                obj.insert(
                    "hit_test_bounds_tree_hits".to_string(),
                    Value::from(row.hit_test_bounds_tree_hits),
                );
                obj.insert(
                    "hit_test_bounds_tree_candidate_rejected".to_string(),
                    Value::from(row.hit_test_bounds_tree_candidate_rejected),
                );
                obj.insert(
                    "hit_test_cached_path_time_us".to_string(),
                    Value::from(row.hit_test_cached_path_time_us),
                );
                obj.insert(
                    "hit_test_bounds_tree_query_time_us".to_string(),
                    Value::from(row.hit_test_bounds_tree_query_time_us),
                );
                obj.insert(
                    "hit_test_candidate_self_only_time_us".to_string(),
                    Value::from(row.hit_test_candidate_self_only_time_us),
                );
                obj.insert(
                    "hit_test_fallback_traversal_time_us".to_string(),
                    Value::from(row.hit_test_fallback_traversal_time_us),
                );
                obj.insert("total_time_us".to_string(), Value::from(row.total_time_us));
                obj.insert(
                    "layout_nodes_performed".to_string(),
                    Value::from(row.layout_nodes_performed),
                );
                obj.insert(
                    "paint_nodes_performed".to_string(),
                    Value::from(row.paint_nodes_performed),
                );
                obj.insert(
                    "paint_cache_misses".to_string(),
                    Value::from(row.paint_cache_misses),
                );
                obj.insert(
                    "layout_engine_solves".to_string(),
                    Value::from(row.layout_engine_solves),
                );
                obj.insert(
                    "layout_engine_solve_time_us".to_string(),
                    Value::from(row.layout_engine_solve_time_us),
                );
                obj.insert(
                    "layout_collect_roots_time_us".to_string(),
                    Value::from(row.layout_collect_roots_time_us),
                );
                obj.insert(
                    "layout_invalidate_scroll_handle_bindings_time_us".to_string(),
                    Value::from(row.layout_invalidate_scroll_handle_bindings_time_us),
                );
                obj.insert(
                    "layout_expand_view_cache_invalidations_time_us".to_string(),
                    Value::from(row.layout_expand_view_cache_invalidations_time_us),
                );
                obj.insert(
                    "layout_request_build_roots_time_us".to_string(),
                    Value::from(row.layout_request_build_roots_time_us),
                );
                obj.insert(
                    "layout_roots_time_us".to_string(),
                    Value::from(row.layout_roots_time_us),
                );
                obj.insert(
                    "layout_pending_barrier_relayouts_time_us".to_string(),
                    Value::from(row.layout_pending_barrier_relayouts_time_us),
                );
                obj.insert(
                    "layout_barrier_relayouts_time_us".to_string(),
                    Value::from(row.layout_barrier_relayouts_time_us),
                );
                obj.insert(
                    "layout_repair_view_cache_bounds_time_us".to_string(),
                    Value::from(row.layout_repair_view_cache_bounds_time_us),
                );
                obj.insert(
                    "layout_contained_view_cache_roots_time_us".to_string(),
                    Value::from(row.layout_contained_view_cache_roots_time_us),
                );
                obj.insert(
                    "layout_collapse_layout_observations_time_us".to_string(),
                    Value::from(row.layout_collapse_layout_observations_time_us),
                );
                obj.insert(
                    "layout_observation_record_time_us".to_string(),
                    Value::from(row.layout_observation_record_time_us),
                );
                obj.insert(
                    "layout_observation_record_models_items".to_string(),
                    Value::from(row.layout_observation_record_models_items),
                );
                obj.insert(
                    "layout_observation_record_globals_items".to_string(),
                    Value::from(row.layout_observation_record_globals_items),
                );
                obj.insert(
                    "layout_view_cache_time_us".to_string(),
                    Value::from(row.layout_view_cache_time_us),
                );
                obj.insert(
                    "layout_semantics_refresh_time_us".to_string(),
                    Value::from(row.layout_semantics_refresh_time_us),
                );
                obj.insert(
                    "layout_focus_repair_time_us".to_string(),
                    Value::from(row.layout_focus_repair_time_us),
                );
                obj.insert(
                    "layout_deferred_cleanup_time_us".to_string(),
                    Value::from(row.layout_deferred_cleanup_time_us),
                );
                obj.insert(
                    "layout_prepaint_after_layout_time_us".to_string(),
                    Value::from(row.layout_prepaint_after_layout_time_us),
                );
                obj.insert(
                    "layout_skipped_engine_frame".to_string(),
                    Value::from(row.layout_skipped_engine_frame),
                );
                obj.insert(
                    "layout_fast_path_taken".to_string(),
                    Value::from(row.layout_fast_path_taken),
                );
                obj.insert("cache_roots".to_string(), Value::from(row.cache_roots));
                obj.insert(
                    "cache_roots_reused".to_string(),
                    Value::from(row.cache_roots_reused),
                );
                obj.insert(
                    "cache_roots_contained_relayout".to_string(),
                    Value::from(row.cache_roots_contained_relayout),
                );
                obj.insert(
                    "cache_replayed_ops".to_string(),
                    Value::from(row.cache_replayed_ops),
                );
                obj.insert(
                    "paint_record_visual_bounds_time_us".to_string(),
                    Value::from(row.paint_record_visual_bounds_time_us),
                );
                obj.insert(
                    "paint_record_visual_bounds_calls".to_string(),
                    Value::from(row.paint_record_visual_bounds_calls),
                );
                obj.insert(
                    "paint_cache_key_time_us".to_string(),
                    Value::from(row.paint_cache_key_time_us),
                );
                obj.insert(
                    "paint_cache_hit_check_time_us".to_string(),
                    Value::from(row.paint_cache_hit_check_time_us),
                );
                obj.insert(
                    "paint_widget_time_us".to_string(),
                    Value::from(row.paint_widget_time_us),
                );
                obj.insert(
                    "paint_observation_record_time_us".to_string(),
                    Value::from(row.paint_observation_record_time_us),
                );
                obj.insert(
                    "paint_host_widget_observed_models_time_us".to_string(),
                    Value::from(row.paint_host_widget_observed_models_time_us),
                );
                obj.insert(
                    "paint_host_widget_observed_models_items".to_string(),
                    Value::from(row.paint_host_widget_observed_models_items),
                );
                obj.insert(
                    "paint_host_widget_observed_globals_time_us".to_string(),
                    Value::from(row.paint_host_widget_observed_globals_time_us),
                );
                obj.insert(
                    "paint_host_widget_observed_globals_items".to_string(),
                    Value::from(row.paint_host_widget_observed_globals_items),
                );
                obj.insert(
                    "paint_host_widget_instance_lookup_time_us".to_string(),
                    Value::from(row.paint_host_widget_instance_lookup_time_us),
                );
                obj.insert(
                    "paint_host_widget_instance_lookup_calls".to_string(),
                    Value::from(row.paint_host_widget_instance_lookup_calls),
                );
                obj.insert(
                    "paint_text_prepare_time_us".to_string(),
                    Value::from(row.paint_text_prepare_time_us),
                );
                obj.insert(
                    "paint_text_prepare_calls".to_string(),
                    Value::from(row.paint_text_prepare_calls),
                );
                obj.insert(
                    "paint_text_prepare_reason_blob_missing".to_string(),
                    Value::from(row.paint_text_prepare_reason_blob_missing),
                );
                obj.insert(
                    "paint_text_prepare_reason_scale_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_scale_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_text_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_text_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_rich_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_rich_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_style_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_style_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_wrap_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_wrap_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_overflow_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_overflow_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_width_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_width_changed),
                );
                obj.insert(
                    "paint_text_prepare_reason_font_stack_changed".to_string(),
                    Value::from(row.paint_text_prepare_reason_font_stack_changed),
                );
                obj.insert(
                    "paint_input_context_time_us".to_string(),
                    Value::from(row.paint_input_context_time_us),
                );
                obj.insert(
                    "paint_scroll_handle_invalidation_time_us".to_string(),
                    Value::from(row.paint_scroll_handle_invalidation_time_us),
                );
                obj.insert(
                    "paint_collect_roots_time_us".to_string(),
                    Value::from(row.paint_collect_roots_time_us),
                );
                obj.insert(
                    "paint_publish_text_input_snapshot_time_us".to_string(),
                    Value::from(row.paint_publish_text_input_snapshot_time_us),
                );
                obj.insert(
                    "paint_collapse_observations_time_us".to_string(),
                    Value::from(row.paint_collapse_observations_time_us),
                );
                obj.insert(
                    "paint_cache_replay_time_us".to_string(),
                    Value::from(row.paint_cache_replay_time_us),
                );
                obj.insert(
                    "paint_cache_bounds_translate_time_us".to_string(),
                    Value::from(row.paint_cache_bounds_translate_time_us),
                );
                obj.insert(
                    "paint_cache_bounds_translated_nodes".to_string(),
                    Value::from(row.paint_cache_bounds_translated_nodes),
                );
                obj.insert(
                    "changed_models".to_string(),
                    Value::from(row.changed_models),
                );
                obj.insert(
                    "changed_globals".to_string(),
                    Value::from(row.changed_globals),
                );
                obj.insert(
                    "changed_global_types_sample".to_string(),
                    Value::Array(
                        row.changed_global_types_sample
                            .iter()
                            .cloned()
                            .map(Value::from)
                            .collect(),
                    ),
                );
                obj.insert(
                    "propagated_model_change_models".to_string(),
                    Value::from(row.propagated_model_change_models),
                );
                obj.insert(
                    "propagated_model_change_observation_edges".to_string(),
                    Value::from(row.propagated_model_change_observation_edges),
                );
                obj.insert(
                    "propagated_model_change_unobserved_models".to_string(),
                    Value::from(row.propagated_model_change_unobserved_models),
                );
                obj.insert(
                    "propagated_global_change_globals".to_string(),
                    Value::from(row.propagated_global_change_globals),
                );
                obj.insert(
                    "propagated_global_change_observation_edges".to_string(),
                    Value::from(row.propagated_global_change_observation_edges),
                );
                obj.insert(
                    "propagated_global_change_unobserved_globals".to_string(),
                    Value::from(row.propagated_global_change_unobserved_globals),
                );
                obj.insert(
                    "invalidation_walk_calls".to_string(),
                    Value::from(row.invalidation_walk_calls),
                );
                obj.insert(
                    "invalidation_walk_nodes".to_string(),
                    Value::from(row.invalidation_walk_nodes),
                );
                obj.insert(
                    "model_change_invalidation_roots".to_string(),
                    Value::from(row.model_change_invalidation_roots),
                );
                obj.insert(
                    "global_change_invalidation_roots".to_string(),
                    Value::from(row.global_change_invalidation_roots),
                );
                obj.insert(
                    "invalidation_walk_calls_model_change".to_string(),
                    Value::from(row.invalidation_walk_calls_model_change),
                );
                obj.insert(
                    "invalidation_walk_nodes_model_change".to_string(),
                    Value::from(row.invalidation_walk_nodes_model_change),
                );
                obj.insert(
                    "invalidation_walk_calls_global_change".to_string(),
                    Value::from(row.invalidation_walk_calls_global_change),
                );
                obj.insert(
                    "invalidation_walk_nodes_global_change".to_string(),
                    Value::from(row.invalidation_walk_nodes_global_change),
                );
                obj.insert(
                    "invalidation_walk_calls_hover".to_string(),
                    Value::from(row.invalidation_walk_calls_hover),
                );
                obj.insert(
                    "invalidation_walk_nodes_hover".to_string(),
                    Value::from(row.invalidation_walk_nodes_hover),
                );
                obj.insert(
                    "invalidation_walk_calls_focus".to_string(),
                    Value::from(row.invalidation_walk_calls_focus),
                );
                obj.insert(
                    "invalidation_walk_nodes_focus".to_string(),
                    Value::from(row.invalidation_walk_nodes_focus),
                );
                obj.insert(
                    "invalidation_walk_calls_other".to_string(),
                    Value::from(row.invalidation_walk_calls_other),
                );
                obj.insert(
                    "invalidation_walk_nodes_other".to_string(),
                    Value::from(row.invalidation_walk_nodes_other),
                );
                obj.insert(
                    "hover_pressable_target_changes".to_string(),
                    Value::from(row.hover_pressable_target_changes),
                );
                obj.insert(
                    "hover_hover_region_target_changes".to_string(),
                    Value::from(row.hover_hover_region_target_changes),
                );
                obj.insert(
                    "hover_declarative_instance_changes".to_string(),
                    Value::from(row.hover_declarative_instance_changes),
                );
                obj.insert(
                    "hover_declarative_hit_test_invalidations".to_string(),
                    Value::from(row.hover_declarative_hit_test_invalidations),
                );
                obj.insert(
                    "hover_declarative_layout_invalidations".to_string(),
                    Value::from(row.hover_declarative_layout_invalidations),
                );
                obj.insert(
                    "hover_declarative_paint_invalidations".to_string(),
                    Value::from(row.hover_declarative_paint_invalidations),
                );

                let top_invalidation_walks = row
                    .top_invalidation_walks
                    .iter()
                    .map(|w| {
                        let mut w_obj = Map::new();
                        w_obj.insert("root_node".to_string(), Value::from(w.root_node));
                        w_obj.insert(
                            "root_element".to_string(),
                            w.root_element.map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "root_element_path".to_string(),
                            w.root_element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "kind".to_string(),
                            w.kind.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "source".to_string(),
                            w.source.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "detail".to_string(),
                            w.detail.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert("walked_nodes".to_string(), Value::from(w.walked_nodes));
                        w_obj.insert(
                            "truncated_at".to_string(),
                            w.truncated_at.map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "root_role".to_string(),
                            w.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        w_obj.insert(
                            "root_test_id".to_string(),
                            w.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(w_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_invalidation_walks".to_string(),
                    Value::Array(top_invalidation_walks),
                );

                let top_hover_declarative_invalidations = row
                    .top_hover_declarative_invalidations
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("node".to_string(), Value::from(h.node));
                        h_obj.insert(
                            "element".to_string(),
                            h.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert("hit_test".to_string(), Value::from(h.hit_test));
                        h_obj.insert("layout".to_string(), Value::from(h.layout));
                        h_obj.insert("paint".to_string(), Value::from(h.paint));
                        h_obj.insert(
                            "role".to_string(),
                            h.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "test_id".to_string(),
                            h.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_hover_declarative_invalidations".to_string(),
                    Value::Array(top_hover_declarative_invalidations),
                );

                let top_cache_roots = row
                    .top_cache_roots
                    .iter()
                    .map(|c| {
                        let mut c_obj = Map::new();
                        c_obj.insert("root_node".to_string(), Value::from(c.root_node));
                        c_obj.insert(
                            "element".to_string(),
                            c.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "element_path".to_string(),
                            c.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert("reused".to_string(), Value::from(c.reused));
                        c_obj.insert(
                            "contained_layout".to_string(),
                            Value::from(c.contained_layout),
                        );
                        c_obj.insert(
                            "contained_relayout_in_frame".to_string(),
                            Value::from(c.contained_relayout_in_frame),
                        );
                        c_obj.insert(
                            "paint_replayed_ops".to_string(),
                            Value::from(c.paint_replayed_ops),
                        );
                        c_obj.insert(
                            "reuse_reason".to_string(),
                            c.reuse_reason
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_in_semantics".to_string(),
                            c.root_in_semantics.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_role".to_string(),
                            c.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_test_id".to_string(),
                            c.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(c_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert("top_cache_roots".to_string(), Value::Array(top_cache_roots));

                let top_contained_relayout_cache_roots = row
                    .top_contained_relayout_cache_roots
                    .iter()
                    .map(|c| {
                        let mut c_obj = Map::new();
                        c_obj.insert("root_node".to_string(), Value::from(c.root_node));
                        c_obj.insert(
                            "element".to_string(),
                            c.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "element_path".to_string(),
                            c.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert("reused".to_string(), Value::from(c.reused));
                        c_obj.insert(
                            "contained_layout".to_string(),
                            Value::from(c.contained_layout),
                        );
                        c_obj.insert(
                            "contained_relayout_in_frame".to_string(),
                            Value::from(c.contained_relayout_in_frame),
                        );
                        c_obj.insert(
                            "paint_replayed_ops".to_string(),
                            Value::from(c.paint_replayed_ops),
                        );
                        c_obj.insert(
                            "reuse_reason".to_string(),
                            c.reuse_reason
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_in_semantics".to_string(),
                            c.root_in_semantics.map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_role".to_string(),
                            c.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        c_obj.insert(
                            "root_test_id".to_string(),
                            c.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(c_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_contained_relayout_cache_roots".to_string(),
                    Value::Array(top_contained_relayout_cache_roots),
                );

                let top_layout_engine_solves = row
                    .top_layout_engine_solves
                    .iter()
                    .map(|s| {
                        let mut s_obj = Map::new();
                        s_obj.insert("root_node".to_string(), Value::from(s.root_node));
                        s_obj.insert(
                            "root_element".to_string(),
                            s.root_element.map(Value::from).unwrap_or(Value::Null),
                        );
                        s_obj.insert(
                            "root_element_kind".to_string(),
                            s.root_element_kind
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        s_obj.insert(
                            "root_element_path".to_string(),
                            s.root_element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        s_obj.insert("solve_time_us".to_string(), Value::from(s.solve_time_us));
                        s_obj.insert("measure_calls".to_string(), Value::from(s.measure_calls));
                        s_obj.insert(
                            "measure_cache_hits".to_string(),
                            Value::from(s.measure_cache_hits),
                        );
                        s_obj.insert(
                            "measure_time_us".to_string(),
                            Value::from(s.measure_time_us),
                        );
                        let top_measures = s
                            .top_measures
                            .iter()
                            .map(|m| {
                                let mut m_obj = Map::new();
                                m_obj.insert("node".to_string(), Value::from(m.node));
                                m_obj.insert(
                                    "measure_time_us".to_string(),
                                    Value::from(m.measure_time_us),
                                );
                                m_obj.insert("calls".to_string(), Value::from(m.calls));
                                m_obj.insert("cache_hits".to_string(), Value::from(m.cache_hits));
                                m_obj.insert(
                                    "element".to_string(),
                                    m.element.map(Value::from).unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "element_kind".to_string(),
                                    m.element_kind
                                        .clone()
                                        .map(Value::from)
                                        .unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "role".to_string(),
                                    m.role.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                m_obj.insert(
                                    "test_id".to_string(),
                                    m.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                let top_children = m
                                    .top_children
                                    .iter()
                                    .map(|c| {
                                        let mut c_obj = Map::new();
                                        c_obj.insert("child".to_string(), Value::from(c.child));
                                        c_obj.insert(
                                            "measure_time_us".to_string(),
                                            Value::from(c.measure_time_us),
                                        );
                                        c_obj.insert("calls".to_string(), Value::from(c.calls));
                                        c_obj.insert(
                                            "element".to_string(),
                                            c.element.map(Value::from).unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "element_kind".to_string(),
                                            c.element_kind
                                                .clone()
                                                .map(Value::from)
                                                .unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "role".to_string(),
                                            c.role.clone().map(Value::from).unwrap_or(Value::Null),
                                        );
                                        c_obj.insert(
                                            "test_id".to_string(),
                                            c.test_id
                                                .clone()
                                                .map(Value::from)
                                                .unwrap_or(Value::Null),
                                        );
                                        Value::Object(c_obj)
                                    })
                                    .collect::<Vec<_>>();
                                m_obj
                                    .insert("top_children".to_string(), Value::Array(top_children));
                                Value::Object(m_obj)
                            })
                            .collect::<Vec<_>>();
                        s_obj.insert("top_measures".to_string(), Value::Array(top_measures));
                        s_obj.insert(
                            "root_role".to_string(),
                            s.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        s_obj.insert(
                            "root_test_id".to_string(),
                            s.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(s_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_layout_engine_solves".to_string(),
                    Value::Array(top_layout_engine_solves),
                );

                let layout_hotspots = row
                    .layout_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("node".to_string(), Value::from(h.node));
                        h_obj.insert(
                            "element".to_string(),
                            h.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "element_kind".to_string(),
                            h.element_kind
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "element_path".to_string(),
                            h.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "widget_type".to_string(),
                            h.widget_type
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert("layout_time_us".to_string(), Value::from(h.layout_time_us));
                        h_obj.insert(
                            "inclusive_time_us".to_string(),
                            Value::from(h.inclusive_time_us),
                        );
                        h_obj.insert(
                            "role".to_string(),
                            h.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "test_id".to_string(),
                            h.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert("layout_hotspots".to_string(), Value::Array(layout_hotspots));

                let widget_measure_hotspots = row
                    .widget_measure_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("node".to_string(), Value::from(h.node));
                        h_obj.insert(
                            "element".to_string(),
                            h.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "element_kind".to_string(),
                            h.element_kind
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "element_path".to_string(),
                            h.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "widget_type".to_string(),
                            h.widget_type
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "measure_time_us".to_string(),
                            Value::from(h.measure_time_us),
                        );
                        h_obj.insert(
                            "inclusive_time_us".to_string(),
                            Value::from(h.inclusive_time_us),
                        );
                        h_obj.insert(
                            "role".to_string(),
                            h.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "test_id".to_string(),
                            h.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "widget_measure_hotspots".to_string(),
                    Value::Array(widget_measure_hotspots),
                );

                let paint_widget_hotspots = row
                    .paint_widget_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("node".to_string(), Value::from(h.node));
                        h_obj.insert(
                            "element".to_string(),
                            h.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "element_kind".to_string(),
                            h.element_kind
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "widget_type".to_string(),
                            h.widget_type
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert("paint_time_us".to_string(), Value::from(h.paint_time_us));
                        h_obj.insert(
                            "inclusive_time_us".to_string(),
                            Value::from(h.inclusive_time_us),
                        );
                        h_obj.insert(
                            "inclusive_scene_ops_delta".to_string(),
                            Value::from(h.inclusive_scene_ops_delta),
                        );
                        h_obj.insert(
                            "exclusive_scene_ops_delta".to_string(),
                            Value::from(h.exclusive_scene_ops_delta),
                        );
                        h_obj.insert(
                            "role".to_string(),
                            h.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "test_id".to_string(),
                            h.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "paint_widget_hotspots".to_string(),
                    Value::Array(paint_widget_hotspots),
                );

                let paint_text_prepare_hotspots = row
                    .paint_text_prepare_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("node".to_string(), Value::from(h.node));
                        h_obj.insert(
                            "element".to_string(),
                            h.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "element_kind".to_string(),
                            h.element_kind
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "prepare_time_us".to_string(),
                            Value::from(h.prepare_time_us),
                        );
                        h_obj.insert("text_len".to_string(), Value::from(h.text_len));
                        h_obj.insert(
                            "max_width".to_string(),
                            h.max_width.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "wrap".to_string(),
                            h.wrap.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "overflow".to_string(),
                            h.overflow.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "scale_factor".to_string(),
                            h.scale_factor.map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert("reasons_mask".to_string(), Value::from(h.reasons_mask));
                        h_obj.insert(
                            "role".to_string(),
                            h.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        h_obj.insert(
                            "test_id".to_string(),
                            h.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "paint_text_prepare_hotspots".to_string(),
                    Value::Array(paint_text_prepare_hotspots),
                );

                let model_change_hotspots = row
                    .model_change_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("model".to_string(), Value::from(h.model));
                        h_obj.insert(
                            "observation_edges".to_string(),
                            Value::from(h.observation_edges),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "model_change_hotspots".to_string(),
                    Value::Array(model_change_hotspots),
                );

                let model_change_unobserved = row
                    .model_change_unobserved
                    .iter()
                    .map(|u| {
                        let mut u_obj = Map::new();
                        u_obj.insert("model".to_string(), Value::from(u.model));
                        u_obj.insert(
                            "created_type".to_string(),
                            u.created_type
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        u_obj.insert(
                            "created_at".to_string(),
                            u.created_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(u_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "model_change_unobserved".to_string(),
                    Value::Array(model_change_unobserved),
                );

                let global_change_hotspots = row
                    .global_change_hotspots
                    .iter()
                    .map(|h| {
                        let mut h_obj = Map::new();
                        h_obj.insert("type_name".to_string(), Value::from(h.type_name.clone()));
                        h_obj.insert(
                            "observation_edges".to_string(),
                            Value::from(h.observation_edges),
                        );
                        h_obj.insert(
                            "changed_at".to_string(),
                            h.changed_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(h_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "global_change_hotspots".to_string(),
                    Value::Array(global_change_hotspots),
                );

                let global_change_unobserved = row
                    .global_change_unobserved
                    .iter()
                    .map(|u| {
                        let mut u_obj = Map::new();
                        u_obj.insert("type_name".to_string(), Value::from(u.type_name.clone()));
                        u_obj.insert(
                            "changed_at".to_string(),
                            u.changed_at.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        Value::Object(u_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "global_change_unobserved".to_string(),
                    Value::Array(global_change_unobserved),
                );

                Value::Object(obj)
            })
            .collect::<Vec<_>>();

        root.insert("top".to_string(), Value::Array(top));
        Value::Object(root)
    }
}
