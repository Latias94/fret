use serde_json::{Map, Value};

const PAINT_WIDGET_HOTSPOT_ROW_TOP_N: usize = 3;
const PAINT_WIDGET_HOTSPOT_SUMMARY_TOP_N: usize = 16;

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsReport {
    sort: BundleStatsSort,
    warmup_frames: u64,
    derived_from_frames_index: bool,
    source_bundle_schema_version: u32,
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
    max_paint_record_visual_bounds_time_us: u64,
    max_paint_record_visual_bounds_calls: u32,
    max_paint_cache_key_time_us: u64,
    max_paint_cache_hit_check_time_us: u64,
    max_paint_observation_record_time_us: u64,
    max_paint_host_widget_observed_models_time_us: u64,
    max_paint_host_widget_observed_models_items: u32,
    max_paint_host_widget_observed_globals_time_us: u64,
    max_paint_host_widget_observed_globals_items: u32,
    max_paint_host_widget_observed_deps_calls: u32,
    max_paint_host_widget_observed_deps_empty_calls: u32,
    max_paint_host_widget_observed_models_non_empty_calls: u32,
    max_paint_host_widget_observed_globals_non_empty_calls: u32,
    max_paint_host_widget_instance_lookup_time_us: u64,
    max_paint_host_widget_instance_lookup_calls: u32,
    pub(super) max_total_time_us: u64,
    pub(super) max_ui_thread_cpu_time_us: u64,
    pub(super) max_ui_thread_cpu_cycle_time_delta_cycles: u64,
    pub(super) max_layout_engine_solve_time_us: u64,
    pub(super) max_dispatch_accounted_time_us: u64,
    pub(super) max_dispatch_unattributed_time_us: u64,
    pub(super) max_dispatch_inner_body_unattributed_time_us: u64,
    pub(super) max_dispatch_runtime_wrapper_time_us: u64,
    pub(super) max_renderer_encode_scene_us: u64,
    pub(super) max_renderer_ensure_pipelines_us: u64,
    pub(super) max_renderer_plan_compile_us: u64,
    pub(super) max_renderer_upload_us: u64,
    pub(super) max_renderer_record_passes_us: u64,
    pub(super) max_renderer_encoder_finish_us: u64,
    pub(super) max_renderer_prepare_svg_us: u64,
    pub(super) max_renderer_prepare_text_us: u64,
    pub(super) max_renderer_prepare_text_collect_pin_keys_us: u64,
    pub(super) max_renderer_prepare_text_bucket_delta_us: u64,
    pub(super) max_renderer_prepare_text_prewarm_us: u64,
    pub(super) max_renderer_prepare_text_pin_bucket_update_us: u64,
    pub(super) max_renderer_prepare_text_flush_uploads_us: u64,
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
    p50_paint_record_visual_bounds_time_us: u64,
    p95_paint_record_visual_bounds_time_us: u64,
    p50_paint_record_visual_bounds_calls: u64,
    p95_paint_record_visual_bounds_calls: u64,
    p50_paint_cache_key_time_us: u64,
    p95_paint_cache_key_time_us: u64,
    p50_paint_cache_hit_check_time_us: u64,
    p95_paint_cache_hit_check_time_us: u64,
    p50_paint_observation_record_time_us: u64,
    p95_paint_observation_record_time_us: u64,
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
    pub(super) p50_dispatch_accounted_time_us: u64,
    pub(super) p95_dispatch_accounted_time_us: u64,
    pub(super) p50_dispatch_unattributed_time_us: u64,
    pub(super) p95_dispatch_unattributed_time_us: u64,
    pub(super) p50_dispatch_inner_body_unattributed_time_us: u64,
    pub(super) p95_dispatch_inner_body_unattributed_time_us: u64,
    pub(super) p50_dispatch_runtime_wrapper_time_us: u64,
    pub(super) p95_dispatch_runtime_wrapper_time_us: u64,
    pub(super) p50_hit_test_time_us: u64,
    pub(super) p95_hit_test_time_us: u64,
    pub(super) p50_paint_widget_time_us: u64,
    pub(super) p95_paint_widget_time_us: u64,
    p50_paint_host_widget_observed_models_time_us: u64,
    p95_paint_host_widget_observed_models_time_us: u64,
    p50_paint_host_widget_observed_models_items: u64,
    p95_paint_host_widget_observed_models_items: u64,
    p50_paint_host_widget_observed_globals_time_us: u64,
    p95_paint_host_widget_observed_globals_time_us: u64,
    p50_paint_host_widget_observed_globals_items: u64,
    p95_paint_host_widget_observed_globals_items: u64,
    p50_paint_host_widget_observed_deps_calls: u64,
    p95_paint_host_widget_observed_deps_calls: u64,
    p50_paint_host_widget_observed_deps_empty_calls: u64,
    p95_paint_host_widget_observed_deps_empty_calls: u64,
    p50_paint_host_widget_observed_models_non_empty_calls: u64,
    p95_paint_host_widget_observed_models_non_empty_calls: u64,
    p50_paint_host_widget_observed_globals_non_empty_calls: u64,
    p95_paint_host_widget_observed_globals_non_empty_calls: u64,
    p50_paint_host_widget_instance_lookup_time_us: u64,
    p95_paint_host_widget_instance_lookup_time_us: u64,
    p50_paint_host_widget_instance_lookup_calls: u64,
    p95_paint_host_widget_instance_lookup_calls: u64,
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
    pub(super) p50_renderer_prepare_text_collect_pin_keys_us: u64,
    pub(super) p95_renderer_prepare_text_collect_pin_keys_us: u64,
    pub(super) p50_renderer_prepare_text_bucket_delta_us: u64,
    pub(super) p95_renderer_prepare_text_bucket_delta_us: u64,
    pub(super) p50_renderer_prepare_text_prewarm_us: u64,
    pub(super) p95_renderer_prepare_text_prewarm_us: u64,
    pub(super) p50_renderer_prepare_text_pin_bucket_update_us: u64,
    pub(super) p95_renderer_prepare_text_pin_bucket_update_us: u64,
    pub(super) p50_renderer_prepare_text_flush_uploads_us: u64,
    pub(super) p95_renderer_prepare_text_flush_uploads_us: u64,
    paint_widget_hotspot_summary: BundleStatsPaintWidgetHotspotSummary,
    code_editor_paint_perf: BundleStatsCodeEditorPaintPerfSummary,
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
    pub(super) element_children_vec_pool_grow_events: u32,
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
    pub(super) paint_host_widget_observed_deps_calls: u32,
    pub(super) paint_host_widget_observed_deps_empty_calls: u32,
    pub(super) paint_host_widget_observed_models_non_empty_calls: u32,
    pub(super) paint_host_widget_observed_globals_non_empty_calls: u32,
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
    pub(super) code_editor_paint_perf: Option<BundleStatsCodeEditorPaintPerf>,
    pub(super) dispatch_time_us: u64,
    pub(super) dispatch_inner_body_time_us: u64,
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
    pub(super) dispatch_input_state_update_time_us: u64,
    pub(super) dispatch_context_build_time_us: u64,
    pub(super) dispatch_prelude_time_us: u64,
    pub(super) dispatch_pointer_arbitration_time_us: u64,
    pub(super) dispatch_pointer_target_routing_time_us: u64,
    pub(super) dispatch_post_widget_control_flow_time_us: u64,
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
    pub(super) window_runtime_snapshot_focus_repair_time_us: u64,
    pub(super) window_runtime_snapshot_input_context_time_us: u64,
    pub(super) window_runtime_snapshot_command_availability_time_us: u64,
    pub(super) window_runtime_snapshot_widget_command_count: u32,
    pub(super) window_runtime_snapshot_command_registry_collect_time_us: u64,
    pub(super) window_runtime_snapshot_command_availability_eval_time_us: u64,
    pub(super) window_runtime_snapshot_shortcut_overlay_time_us: u64,
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
    pub(super) renderer_prepare_text_collect_pin_keys_us: u64,
    pub(super) renderer_prepare_text_bucket_delta_us: u64,
    pub(super) renderer_prepare_text_prewarm_us: u64,
    pub(super) renderer_prepare_text_pin_bucket_update_us: u64,
    pub(super) renderer_prepare_text_flush_uploads_us: u64,
    pub(super) renderer_prepare_text_scene_text_blobs: u64,
    pub(super) renderer_prepare_text_pinned_glyph_keys: u64,
    pub(super) renderer_prepare_text_prewarm_glyph_keys: u64,
    pub(super) renderer_prepare_text_retained_glyph_keys: u64,
    pub(super) renderer_prepare_text_added_glyph_keys: u64,
    pub(super) renderer_prepare_text_removed_glyph_keys: u64,
    pub(super) renderer_prepare_svg_us: u64,
    pub(super) renderer_encode_scene_stack_us: u64,
    pub(super) renderer_encode_scene_clip_us: u64,
    pub(super) renderer_encode_scene_mask_us: u64,
    pub(super) renderer_encode_scene_effect_us: u64,
    pub(super) renderer_encode_scene_quad_us: u64,
    pub(super) renderer_encode_scene_image_us: u64,
    pub(super) renderer_encode_scene_text_us: u64,
    pub(super) renderer_encode_scene_path_us: u64,
    pub(super) renderer_encode_scene_viewport_us: u64,
    pub(super) renderer_encode_scene_flush_us: u64,
    pub(super) renderer_encode_scene_text_shadow_us: u64,
    pub(super) renderer_encode_scene_text_setup_us: u64,
    pub(super) renderer_encode_scene_text_glyphs_us: u64,
    pub(super) renderer_encode_scene_text_glyph_transform_us: u64,
    pub(super) renderer_encode_scene_text_glyph_emit_us: u64,
    pub(super) renderer_encode_scene_text_group_flush_us: u64,
    pub(super) renderer_encode_scene_text_vertex_grow_events: u64,
    pub(super) renderer_encode_scene_text_transform_fast_path_glyphs: u64,
    pub(super) renderer_encode_scene_text_transform_generic_glyphs: u64,
    pub(super) renderer_encode_scene_stack_ops: u64,
    pub(super) renderer_encode_scene_clip_ops: u64,
    pub(super) renderer_encode_scene_mask_ops: u64,
    pub(super) renderer_encode_scene_effect_ops: u64,
    pub(super) renderer_encode_scene_quad_ops: u64,
    pub(super) renderer_encode_scene_image_ops: u64,
    pub(super) renderer_encode_scene_text_ops: u64,
    pub(super) renderer_encode_scene_path_ops: u64,
    pub(super) renderer_encode_scene_viewport_ops: u64,
    pub(super) renderer_encode_scene_flushes: u64,
    pub(super) renderer_svg_upload_bytes: u64,
    pub(super) renderer_image_upload_bytes: u64,
    pub(super) renderer_uniform_bytes: u64,
    pub(super) renderer_instance_bytes: u64,
    pub(super) renderer_vertex_bytes: u64,

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
    pub(super) renderer_custom_effect_v2_user_image_incompatible_fallbacks: u64,
    pub(super) renderer_custom_effect_v3_steps_requested: u64,
    pub(super) renderer_custom_effect_v3_passes_emitted: u64,
    pub(super) renderer_custom_effect_v3_user0_image_incompatible_fallbacks: u64,
    pub(super) renderer_custom_effect_v3_user1_image_incompatible_fallbacks: u64,
    pub(super) renderer_custom_effect_v3_pyramid_cache_hits: u64,
    pub(super) renderer_custom_effect_v3_pyramid_cache_misses: u64,
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
    pub(super) layout_engine_child_rect_queries: u64,
    pub(super) layout_engine_child_rect_time_us: u64,
    pub(super) layout_engine_widget_fallback_solves: u64,
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
    pub(super) layout_request_build_roots: Vec<BundleStatsLayoutRequestBuildRoot>,
    pub(super) scroll_layout_profiles: Vec<BundleStatsScrollLayoutProfile>,
    pub(super) top_layout_engine_solves: Vec<BundleStatsLayoutEngineSolve>,
    pub(super) layout_hotspots: Vec<BundleStatsLayoutHotspot>,
    pub(super) widget_measure_hotspots: Vec<BundleStatsWidgetMeasureHotspot>,
    pub(super) paint_widget_hotspots: Vec<BundleStatsPaintWidgetHotspot>,
    pub(super) paint_text_prepare_hotspots: Vec<BundleStatsPaintTextPrepareHotspot>,
    pub(super) command_availability_hotspots: Vec<BundleStatsCommandAvailabilityHotspot>,
    pub(super) model_change_hotspots: Vec<BundleStatsModelChangeHotspot>,
    pub(super) model_change_unobserved: Vec<BundleStatsModelChangeUnobserved>,
    pub(super) global_change_hotspots: Vec<BundleStatsGlobalChangeHotspot>,
    pub(super) global_change_unobserved: Vec<BundleStatsGlobalChangeUnobserved>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsCodeEditorPaintPerf {
    pub(super) frame_seq: u64,
    pub(super) visible_start: u64,
    pub(super) visible_end: u64,
    pub(super) visible_rows: u64,
    pub(super) cache_base_entries: u64,
    pub(super) cache_frame_min_entries: u64,
    pub(super) cache_effective_entries: u64,
    pub(super) rows_painted: u64,
    pub(super) rows_drew_rich: u64,
    pub(super) rows_scene_replayed: u64,
    pub(super) rows_scene_prepaint_planned: u64,
    pub(super) rows_scene_prepaint_plan_used: u64,
    pub(super) rows_scene_stored: u64,
    pub(super) rows_scene_stored_at_visible_start: u64,
    pub(super) rows_scene_stored_at_visible_end: u64,
    pub(super) row_scene_ops_stored: u64,
    pub(super) rows_scene_prepaint_edge_stored: u64,
    pub(super) row_scene_prepaint_edge_ops_stored: u64,
    pub(super) rows_scene_prepaint_candidates: u64,
    pub(super) rows_scene_prepaint_skip_no_cache: u64,
    pub(super) rows_scene_prepaint_skip_unsupported_key: u64,
    pub(super) rows_scene_prepaint_skip_preedit: u64,
    pub(super) rows_scene_prepaint_skip_syntax_empty: u64,
    pub(super) rows_scene_prepaint_skip_key_mismatch: u64,
    pub(super) rows_scene_fast_miss_no_entry: u64,
    pub(super) rows_scene_fast_miss_key_mismatch: u64,
    pub(super) rows_scene_full_miss_no_entry: u64,
    pub(super) rows_scene_full_miss_key_mismatch: u64,
    pub(super) quads_selection: u64,
    pub(super) quads_caret: u64,
    pub(super) syntax_rows_stored: u64,
    pub(super) us_total: u64,
    pub(super) us_row_text: u64,
    pub(super) us_baseline_measure: u64,
    pub(super) us_row_content_resolve: u64,
    pub(super) us_row_rich_cache_compare: u64,
    pub(super) us_row_geom_key: u64,
    pub(super) us_rich_materialize: u64,
    pub(super) us_text_draw: u64,
    pub(super) us_row_scene_key: u64,
    pub(super) us_row_scene_fast_probe: u64,
    pub(super) us_row_scene_full_probe: u64,
    pub(super) us_row_scene_fast_key_compare: u64,
    pub(super) us_row_scene_full_key_compare: u64,
    pub(super) us_row_scene_replay_touch: u64,
    pub(super) us_row_scene_replay_ops: u64,
    pub(super) us_row_scene_prepaint_plan: u64,
    pub(super) us_row_scene_prepaint_probe: u64,
    pub(super) us_row_scene_prepaint_key_compare: u64,
    pub(super) us_row_scene_capture_ops: u64,
    pub(super) us_row_scene_store: u64,
    pub(super) us_row_scene_prepaint_edge_store: u64,
    pub(super) us_row_scene_fast_path: u64,
    pub(super) us_row_scene_full_path: u64,
    pub(super) us_syntax_spans: u64,
    pub(super) us_syntax_slice: u64,
    pub(super) us_syntax_highlight: u64,
    pub(super) us_syntax_distribute: u64,
    pub(super) us_syntax_store: u64,
    pub(super) us_selection_rects: u64,
    pub(super) us_caret_x: u64,
    pub(super) us_caret_stops: u64,
    pub(super) us_caret_rect: u64,
    pub(super) us_row_geom_cache: u64,
    pub(super) us_row_geom_resolve: u64,
    pub(super) us_row_overlay: u64,
    pub(super) us_frame_overlay_prepare: u64,
    pub(super) surface_rows_iterated: u64,
    pub(super) surface_rows_with_rect: u64,
    pub(super) us_windowed_surface_paint_callback: u64,
    pub(super) us_windowed_surface_frame_lookup: u64,
    pub(super) us_windowed_surface_hook: u64,
    pub(super) us_windowed_surface_row_loop: u64,
    pub(super) us_windowed_surface_row_rect: u64,
    pub(super) us_windowed_surface_row_paint: u64,
    pub(super) us_windowed_surface_non_row: u64,
    pub(super) us_windowed_surface_row_callback_gap: u64,
    pub(super) us_torture_autoscroll: u64,
    pub(super) us_torture_overlay: u64,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsCodeEditorPaintPerfSummary {
    frames: u32,
    sum: BundleStatsCodeEditorPaintPerfTotals,
    max: BundleStatsCodeEditorPaintPerfTotals,
    p50: BundleStatsCodeEditorPaintPerfTotals,
    p95: BundleStatsCodeEditorPaintPerfTotals,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsCodeEditorPaintPerfTotals {
    rows_painted: u64,
    rows_drew_rich: u64,
    rows_scene_replayed: u64,
    rows_scene_prepaint_planned: u64,
    rows_scene_prepaint_plan_used: u64,
    rows_scene_stored: u64,
    rows_scene_stored_at_visible_start: u64,
    rows_scene_stored_at_visible_end: u64,
    row_scene_ops_stored: u64,
    rows_scene_prepaint_edge_stored: u64,
    row_scene_prepaint_edge_ops_stored: u64,
    rows_scene_prepaint_candidates: u64,
    rows_scene_prepaint_skip_no_cache: u64,
    rows_scene_prepaint_skip_unsupported_key: u64,
    rows_scene_prepaint_skip_preedit: u64,
    rows_scene_prepaint_skip_syntax_empty: u64,
    rows_scene_prepaint_skip_key_mismatch: u64,
    rows_scene_fast_miss_no_entry: u64,
    rows_scene_fast_miss_key_mismatch: u64,
    rows_scene_full_miss_no_entry: u64,
    rows_scene_full_miss_key_mismatch: u64,
    quads_selection: u64,
    quads_caret: u64,
    syntax_rows_stored: u64,
    us_total: u64,
    us_row_text: u64,
    us_baseline_measure: u64,
    us_row_content_resolve: u64,
    us_row_rich_cache_compare: u64,
    us_row_geom_key: u64,
    us_rich_materialize: u64,
    us_text_draw: u64,
    us_row_scene_key: u64,
    us_row_scene_fast_probe: u64,
    us_row_scene_full_probe: u64,
    us_row_scene_fast_key_compare: u64,
    us_row_scene_full_key_compare: u64,
    us_row_scene_replay_touch: u64,
    us_row_scene_replay_ops: u64,
    us_row_scene_prepaint_plan: u64,
    us_row_scene_prepaint_probe: u64,
    us_row_scene_prepaint_key_compare: u64,
    us_row_scene_capture_ops: u64,
    us_row_scene_store: u64,
    us_row_scene_prepaint_edge_store: u64,
    us_row_scene_fast_path: u64,
    us_row_scene_full_path: u64,
    us_syntax_spans: u64,
    us_syntax_slice: u64,
    us_syntax_highlight: u64,
    us_syntax_distribute: u64,
    us_syntax_store: u64,
    us_selection_rects: u64,
    us_caret_x: u64,
    us_caret_stops: u64,
    us_caret_rect: u64,
    us_row_geom_cache: u64,
    us_row_geom_resolve: u64,
    us_row_overlay: u64,
    us_frame_overlay_prepare: u64,
    surface_rows_iterated: u64,
    surface_rows_with_rect: u64,
    us_windowed_surface_paint_callback: u64,
    us_windowed_surface_frame_lookup: u64,
    us_windowed_surface_hook: u64,
    us_windowed_surface_row_loop: u64,
    us_windowed_surface_row_rect: u64,
    us_windowed_surface_row_paint: u64,
    us_windowed_surface_non_row: u64,
    us_windowed_surface_row_callback_gap: u64,
    us_torture_autoscroll: u64,
    us_torture_overlay: u64,
}

impl BundleStatsCodeEditorPaintPerfTotals {
    fn add_frame(&mut self, p: &BundleStatsCodeEditorPaintPerf) {
        self.rows_painted = self.rows_painted.saturating_add(p.rows_painted);
        self.rows_drew_rich = self.rows_drew_rich.saturating_add(p.rows_drew_rich);
        self.rows_scene_replayed = self
            .rows_scene_replayed
            .saturating_add(p.rows_scene_replayed);
        self.rows_scene_prepaint_planned = self
            .rows_scene_prepaint_planned
            .saturating_add(p.rows_scene_prepaint_planned);
        self.rows_scene_prepaint_plan_used = self
            .rows_scene_prepaint_plan_used
            .saturating_add(p.rows_scene_prepaint_plan_used);
        self.rows_scene_stored = self.rows_scene_stored.saturating_add(p.rows_scene_stored);
        self.rows_scene_stored_at_visible_start = self
            .rows_scene_stored_at_visible_start
            .saturating_add(p.rows_scene_stored_at_visible_start);
        self.rows_scene_stored_at_visible_end = self
            .rows_scene_stored_at_visible_end
            .saturating_add(p.rows_scene_stored_at_visible_end);
        self.row_scene_ops_stored = self
            .row_scene_ops_stored
            .saturating_add(p.row_scene_ops_stored);
        self.rows_scene_prepaint_edge_stored = self
            .rows_scene_prepaint_edge_stored
            .saturating_add(p.rows_scene_prepaint_edge_stored);
        self.row_scene_prepaint_edge_ops_stored = self
            .row_scene_prepaint_edge_ops_stored
            .saturating_add(p.row_scene_prepaint_edge_ops_stored);
        self.rows_scene_prepaint_candidates = self
            .rows_scene_prepaint_candidates
            .saturating_add(p.rows_scene_prepaint_candidates);
        self.rows_scene_prepaint_skip_no_cache = self
            .rows_scene_prepaint_skip_no_cache
            .saturating_add(p.rows_scene_prepaint_skip_no_cache);
        self.rows_scene_prepaint_skip_unsupported_key = self
            .rows_scene_prepaint_skip_unsupported_key
            .saturating_add(p.rows_scene_prepaint_skip_unsupported_key);
        self.rows_scene_prepaint_skip_preedit = self
            .rows_scene_prepaint_skip_preedit
            .saturating_add(p.rows_scene_prepaint_skip_preedit);
        self.rows_scene_prepaint_skip_syntax_empty = self
            .rows_scene_prepaint_skip_syntax_empty
            .saturating_add(p.rows_scene_prepaint_skip_syntax_empty);
        self.rows_scene_prepaint_skip_key_mismatch = self
            .rows_scene_prepaint_skip_key_mismatch
            .saturating_add(p.rows_scene_prepaint_skip_key_mismatch);
        self.rows_scene_fast_miss_no_entry = self
            .rows_scene_fast_miss_no_entry
            .saturating_add(p.rows_scene_fast_miss_no_entry);
        self.rows_scene_fast_miss_key_mismatch = self
            .rows_scene_fast_miss_key_mismatch
            .saturating_add(p.rows_scene_fast_miss_key_mismatch);
        self.rows_scene_full_miss_no_entry = self
            .rows_scene_full_miss_no_entry
            .saturating_add(p.rows_scene_full_miss_no_entry);
        self.rows_scene_full_miss_key_mismatch = self
            .rows_scene_full_miss_key_mismatch
            .saturating_add(p.rows_scene_full_miss_key_mismatch);
        self.quads_selection = self.quads_selection.saturating_add(p.quads_selection);
        self.quads_caret = self.quads_caret.saturating_add(p.quads_caret);
        self.syntax_rows_stored = self
            .syntax_rows_stored
            .saturating_add(p.syntax_rows_stored);
        self.us_total = self.us_total.saturating_add(p.us_total);
        self.us_row_text = self.us_row_text.saturating_add(p.us_row_text);
        self.us_baseline_measure = self
            .us_baseline_measure
            .saturating_add(p.us_baseline_measure);
        self.us_row_content_resolve = self
            .us_row_content_resolve
            .saturating_add(p.us_row_content_resolve);
        self.us_row_rich_cache_compare = self
            .us_row_rich_cache_compare
            .saturating_add(p.us_row_rich_cache_compare);
        self.us_row_geom_key = self.us_row_geom_key.saturating_add(p.us_row_geom_key);
        self.us_rich_materialize = self
            .us_rich_materialize
            .saturating_add(p.us_rich_materialize);
        self.us_text_draw = self.us_text_draw.saturating_add(p.us_text_draw);
        self.us_row_scene_key = self.us_row_scene_key.saturating_add(p.us_row_scene_key);
        self.us_row_scene_fast_probe = self
            .us_row_scene_fast_probe
            .saturating_add(p.us_row_scene_fast_probe);
        self.us_row_scene_full_probe = self
            .us_row_scene_full_probe
            .saturating_add(p.us_row_scene_full_probe);
        self.us_row_scene_fast_key_compare = self
            .us_row_scene_fast_key_compare
            .saturating_add(p.us_row_scene_fast_key_compare);
        self.us_row_scene_full_key_compare = self
            .us_row_scene_full_key_compare
            .saturating_add(p.us_row_scene_full_key_compare);
        self.us_row_scene_replay_touch = self
            .us_row_scene_replay_touch
            .saturating_add(p.us_row_scene_replay_touch);
        self.us_row_scene_replay_ops = self
            .us_row_scene_replay_ops
            .saturating_add(p.us_row_scene_replay_ops);
        self.us_row_scene_prepaint_plan = self
            .us_row_scene_prepaint_plan
            .saturating_add(p.us_row_scene_prepaint_plan);
        self.us_row_scene_prepaint_probe = self
            .us_row_scene_prepaint_probe
            .saturating_add(p.us_row_scene_prepaint_probe);
        self.us_row_scene_prepaint_key_compare = self
            .us_row_scene_prepaint_key_compare
            .saturating_add(p.us_row_scene_prepaint_key_compare);
        self.us_row_scene_capture_ops = self
            .us_row_scene_capture_ops
            .saturating_add(p.us_row_scene_capture_ops);
        self.us_row_scene_store = self
            .us_row_scene_store
            .saturating_add(p.us_row_scene_store);
        self.us_row_scene_prepaint_edge_store = self
            .us_row_scene_prepaint_edge_store
            .saturating_add(p.us_row_scene_prepaint_edge_store);
        self.us_row_scene_fast_path = self
            .us_row_scene_fast_path
            .saturating_add(p.us_row_scene_fast_path);
        self.us_row_scene_full_path = self
            .us_row_scene_full_path
            .saturating_add(p.us_row_scene_full_path);
        self.us_syntax_spans = self.us_syntax_spans.saturating_add(p.us_syntax_spans);
        self.us_syntax_slice = self.us_syntax_slice.saturating_add(p.us_syntax_slice);
        self.us_syntax_highlight = self
            .us_syntax_highlight
            .saturating_add(p.us_syntax_highlight);
        self.us_syntax_distribute = self
            .us_syntax_distribute
            .saturating_add(p.us_syntax_distribute);
        self.us_syntax_store = self.us_syntax_store.saturating_add(p.us_syntax_store);
        self.us_selection_rects = self
            .us_selection_rects
            .saturating_add(p.us_selection_rects);
        self.us_caret_x = self.us_caret_x.saturating_add(p.us_caret_x);
        self.us_caret_stops = self.us_caret_stops.saturating_add(p.us_caret_stops);
        self.us_caret_rect = self.us_caret_rect.saturating_add(p.us_caret_rect);
        self.us_row_geom_cache = self
            .us_row_geom_cache
            .saturating_add(p.us_row_geom_cache);
        self.us_row_geom_resolve = self
            .us_row_geom_resolve
            .saturating_add(p.us_row_geom_resolve);
        self.us_row_overlay = self.us_row_overlay.saturating_add(p.us_row_overlay);
        self.us_frame_overlay_prepare = self
            .us_frame_overlay_prepare
            .saturating_add(p.us_frame_overlay_prepare);
        self.surface_rows_iterated = self
            .surface_rows_iterated
            .saturating_add(p.surface_rows_iterated);
        self.surface_rows_with_rect = self
            .surface_rows_with_rect
            .saturating_add(p.surface_rows_with_rect);
        self.us_windowed_surface_paint_callback = self
            .us_windowed_surface_paint_callback
            .saturating_add(p.us_windowed_surface_paint_callback);
        self.us_windowed_surface_frame_lookup = self
            .us_windowed_surface_frame_lookup
            .saturating_add(p.us_windowed_surface_frame_lookup);
        self.us_windowed_surface_hook = self
            .us_windowed_surface_hook
            .saturating_add(p.us_windowed_surface_hook);
        self.us_windowed_surface_row_loop = self
            .us_windowed_surface_row_loop
            .saturating_add(p.us_windowed_surface_row_loop);
        self.us_windowed_surface_row_rect = self
            .us_windowed_surface_row_rect
            .saturating_add(p.us_windowed_surface_row_rect);
        self.us_windowed_surface_row_paint = self
            .us_windowed_surface_row_paint
            .saturating_add(p.us_windowed_surface_row_paint);
        self.us_windowed_surface_non_row = self
            .us_windowed_surface_non_row
            .saturating_add(p.us_windowed_surface_non_row);
        self.us_windowed_surface_row_callback_gap = self
            .us_windowed_surface_row_callback_gap
            .saturating_add(p.us_windowed_surface_row_callback_gap);
        self.us_torture_autoscroll = self
            .us_torture_autoscroll
            .saturating_add(p.us_torture_autoscroll);
        self.us_torture_overlay = self
            .us_torture_overlay
            .saturating_add(p.us_torture_overlay);
    }

    fn max_frame(&mut self, p: &BundleStatsCodeEditorPaintPerf) {
        self.rows_painted = self.rows_painted.max(p.rows_painted);
        self.rows_drew_rich = self.rows_drew_rich.max(p.rows_drew_rich);
        self.rows_scene_replayed = self.rows_scene_replayed.max(p.rows_scene_replayed);
        self.rows_scene_prepaint_planned = self
            .rows_scene_prepaint_planned
            .max(p.rows_scene_prepaint_planned);
        self.rows_scene_prepaint_plan_used = self
            .rows_scene_prepaint_plan_used
            .max(p.rows_scene_prepaint_plan_used);
        self.rows_scene_stored = self.rows_scene_stored.max(p.rows_scene_stored);
        self.rows_scene_stored_at_visible_start = self
            .rows_scene_stored_at_visible_start
            .max(p.rows_scene_stored_at_visible_start);
        self.rows_scene_stored_at_visible_end = self
            .rows_scene_stored_at_visible_end
            .max(p.rows_scene_stored_at_visible_end);
        self.row_scene_ops_stored = self.row_scene_ops_stored.max(p.row_scene_ops_stored);
        self.rows_scene_prepaint_edge_stored = self
            .rows_scene_prepaint_edge_stored
            .max(p.rows_scene_prepaint_edge_stored);
        self.row_scene_prepaint_edge_ops_stored = self
            .row_scene_prepaint_edge_ops_stored
            .max(p.row_scene_prepaint_edge_ops_stored);
        self.rows_scene_prepaint_candidates = self
            .rows_scene_prepaint_candidates
            .max(p.rows_scene_prepaint_candidates);
        self.rows_scene_prepaint_skip_no_cache = self
            .rows_scene_prepaint_skip_no_cache
            .max(p.rows_scene_prepaint_skip_no_cache);
        self.rows_scene_prepaint_skip_unsupported_key = self
            .rows_scene_prepaint_skip_unsupported_key
            .max(p.rows_scene_prepaint_skip_unsupported_key);
        self.rows_scene_prepaint_skip_preedit = self
            .rows_scene_prepaint_skip_preedit
            .max(p.rows_scene_prepaint_skip_preedit);
        self.rows_scene_prepaint_skip_syntax_empty = self
            .rows_scene_prepaint_skip_syntax_empty
            .max(p.rows_scene_prepaint_skip_syntax_empty);
        self.rows_scene_prepaint_skip_key_mismatch = self
            .rows_scene_prepaint_skip_key_mismatch
            .max(p.rows_scene_prepaint_skip_key_mismatch);
        self.rows_scene_fast_miss_no_entry = self
            .rows_scene_fast_miss_no_entry
            .max(p.rows_scene_fast_miss_no_entry);
        self.rows_scene_fast_miss_key_mismatch = self
            .rows_scene_fast_miss_key_mismatch
            .max(p.rows_scene_fast_miss_key_mismatch);
        self.rows_scene_full_miss_no_entry = self
            .rows_scene_full_miss_no_entry
            .max(p.rows_scene_full_miss_no_entry);
        self.rows_scene_full_miss_key_mismatch = self
            .rows_scene_full_miss_key_mismatch
            .max(p.rows_scene_full_miss_key_mismatch);
        self.quads_selection = self.quads_selection.max(p.quads_selection);
        self.quads_caret = self.quads_caret.max(p.quads_caret);
        self.syntax_rows_stored = self.syntax_rows_stored.max(p.syntax_rows_stored);
        self.us_total = self.us_total.max(p.us_total);
        self.us_row_text = self.us_row_text.max(p.us_row_text);
        self.us_baseline_measure = self.us_baseline_measure.max(p.us_baseline_measure);
        self.us_row_content_resolve = self
            .us_row_content_resolve
            .max(p.us_row_content_resolve);
        self.us_row_rich_cache_compare = self
            .us_row_rich_cache_compare
            .max(p.us_row_rich_cache_compare);
        self.us_row_geom_key = self.us_row_geom_key.max(p.us_row_geom_key);
        self.us_rich_materialize = self.us_rich_materialize.max(p.us_rich_materialize);
        self.us_text_draw = self.us_text_draw.max(p.us_text_draw);
        self.us_row_scene_key = self.us_row_scene_key.max(p.us_row_scene_key);
        self.us_row_scene_fast_probe = self
            .us_row_scene_fast_probe
            .max(p.us_row_scene_fast_probe);
        self.us_row_scene_full_probe = self
            .us_row_scene_full_probe
            .max(p.us_row_scene_full_probe);
        self.us_row_scene_fast_key_compare = self
            .us_row_scene_fast_key_compare
            .max(p.us_row_scene_fast_key_compare);
        self.us_row_scene_full_key_compare = self
            .us_row_scene_full_key_compare
            .max(p.us_row_scene_full_key_compare);
        self.us_row_scene_replay_touch = self
            .us_row_scene_replay_touch
            .max(p.us_row_scene_replay_touch);
        self.us_row_scene_replay_ops = self.us_row_scene_replay_ops.max(p.us_row_scene_replay_ops);
        self.us_row_scene_prepaint_plan = self
            .us_row_scene_prepaint_plan
            .max(p.us_row_scene_prepaint_plan);
        self.us_row_scene_prepaint_probe = self
            .us_row_scene_prepaint_probe
            .max(p.us_row_scene_prepaint_probe);
        self.us_row_scene_prepaint_key_compare = self
            .us_row_scene_prepaint_key_compare
            .max(p.us_row_scene_prepaint_key_compare);
        self.us_row_scene_capture_ops = self
            .us_row_scene_capture_ops
            .max(p.us_row_scene_capture_ops);
        self.us_row_scene_store = self.us_row_scene_store.max(p.us_row_scene_store);
        self.us_row_scene_prepaint_edge_store = self
            .us_row_scene_prepaint_edge_store
            .max(p.us_row_scene_prepaint_edge_store);
        self.us_row_scene_fast_path = self.us_row_scene_fast_path.max(p.us_row_scene_fast_path);
        self.us_row_scene_full_path = self.us_row_scene_full_path.max(p.us_row_scene_full_path);
        self.us_syntax_spans = self.us_syntax_spans.max(p.us_syntax_spans);
        self.us_syntax_slice = self.us_syntax_slice.max(p.us_syntax_slice);
        self.us_syntax_highlight = self.us_syntax_highlight.max(p.us_syntax_highlight);
        self.us_syntax_distribute = self.us_syntax_distribute.max(p.us_syntax_distribute);
        self.us_syntax_store = self.us_syntax_store.max(p.us_syntax_store);
        self.us_selection_rects = self.us_selection_rects.max(p.us_selection_rects);
        self.us_caret_x = self.us_caret_x.max(p.us_caret_x);
        self.us_caret_stops = self.us_caret_stops.max(p.us_caret_stops);
        self.us_caret_rect = self.us_caret_rect.max(p.us_caret_rect);
        self.us_row_geom_cache = self.us_row_geom_cache.max(p.us_row_geom_cache);
        self.us_row_geom_resolve = self.us_row_geom_resolve.max(p.us_row_geom_resolve);
        self.us_row_overlay = self.us_row_overlay.max(p.us_row_overlay);
        self.us_frame_overlay_prepare = self
            .us_frame_overlay_prepare
            .max(p.us_frame_overlay_prepare);
        self.surface_rows_iterated = self.surface_rows_iterated.max(p.surface_rows_iterated);
        self.surface_rows_with_rect = self.surface_rows_with_rect.max(p.surface_rows_with_rect);
        self.us_windowed_surface_paint_callback = self
            .us_windowed_surface_paint_callback
            .max(p.us_windowed_surface_paint_callback);
        self.us_windowed_surface_frame_lookup = self
            .us_windowed_surface_frame_lookup
            .max(p.us_windowed_surface_frame_lookup);
        self.us_windowed_surface_hook = self
            .us_windowed_surface_hook
            .max(p.us_windowed_surface_hook);
        self.us_windowed_surface_row_loop = self
            .us_windowed_surface_row_loop
            .max(p.us_windowed_surface_row_loop);
        self.us_windowed_surface_row_rect = self
            .us_windowed_surface_row_rect
            .max(p.us_windowed_surface_row_rect);
        self.us_windowed_surface_row_paint = self
            .us_windowed_surface_row_paint
            .max(p.us_windowed_surface_row_paint);
        self.us_windowed_surface_non_row = self
            .us_windowed_surface_non_row
            .max(p.us_windowed_surface_non_row);
        self.us_windowed_surface_row_callback_gap = self
            .us_windowed_surface_row_callback_gap
            .max(p.us_windowed_surface_row_callback_gap);
        self.us_torture_autoscroll = self.us_torture_autoscroll.max(p.us_torture_autoscroll);
        self.us_torture_overlay = self.us_torture_overlay.max(p.us_torture_overlay);
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "rows_painted": self.rows_painted,
            "rows_drew_rich": self.rows_drew_rich,
            "rows_scene_replayed": self.rows_scene_replayed,
            "rows_scene_prepaint_planned": self.rows_scene_prepaint_planned,
            "rows_scene_prepaint_plan_used": self.rows_scene_prepaint_plan_used,
            "rows_scene_stored": self.rows_scene_stored,
            "rows_scene_stored_at_visible_start": self.rows_scene_stored_at_visible_start,
            "rows_scene_stored_at_visible_end": self.rows_scene_stored_at_visible_end,
            "row_scene_ops_stored": self.row_scene_ops_stored,
            "rows_scene_prepaint_edge_stored": self.rows_scene_prepaint_edge_stored,
            "row_scene_prepaint_edge_ops_stored": self.row_scene_prepaint_edge_ops_stored,
            "rows_scene_prepaint_candidates": self.rows_scene_prepaint_candidates,
            "rows_scene_prepaint_skip_no_cache": self.rows_scene_prepaint_skip_no_cache,
            "rows_scene_prepaint_skip_unsupported_key": self.rows_scene_prepaint_skip_unsupported_key,
            "rows_scene_prepaint_skip_preedit": self.rows_scene_prepaint_skip_preedit,
            "rows_scene_prepaint_skip_syntax_empty": self.rows_scene_prepaint_skip_syntax_empty,
            "rows_scene_prepaint_skip_key_mismatch": self.rows_scene_prepaint_skip_key_mismatch,
            "rows_scene_fast_miss_no_entry": self.rows_scene_fast_miss_no_entry,
            "rows_scene_fast_miss_key_mismatch": self.rows_scene_fast_miss_key_mismatch,
            "rows_scene_full_miss_no_entry": self.rows_scene_full_miss_no_entry,
            "rows_scene_full_miss_key_mismatch": self.rows_scene_full_miss_key_mismatch,
            "quads_selection": self.quads_selection,
            "quads_caret": self.quads_caret,
            "syntax_rows_stored": self.syntax_rows_stored,
            "us_total": self.us_total,
            "us_row_text": self.us_row_text,
            "us_baseline_measure": self.us_baseline_measure,
            "us_row_content_resolve": self.us_row_content_resolve,
            "us_row_rich_cache_compare": self.us_row_rich_cache_compare,
            "us_row_geom_key": self.us_row_geom_key,
            "us_rich_materialize": self.us_rich_materialize,
            "us_text_draw": self.us_text_draw,
            "us_row_scene_key": self.us_row_scene_key,
            "us_row_scene_fast_probe": self.us_row_scene_fast_probe,
            "us_row_scene_full_probe": self.us_row_scene_full_probe,
            "us_row_scene_fast_key_compare": self.us_row_scene_fast_key_compare,
            "us_row_scene_full_key_compare": self.us_row_scene_full_key_compare,
            "us_row_scene_replay_touch": self.us_row_scene_replay_touch,
            "us_row_scene_replay_ops": self.us_row_scene_replay_ops,
            "us_row_scene_prepaint_plan": self.us_row_scene_prepaint_plan,
            "us_row_scene_prepaint_probe": self.us_row_scene_prepaint_probe,
            "us_row_scene_prepaint_key_compare": self.us_row_scene_prepaint_key_compare,
            "us_row_scene_capture_ops": self.us_row_scene_capture_ops,
            "us_row_scene_store": self.us_row_scene_store,
            "us_row_scene_prepaint_edge_store": self.us_row_scene_prepaint_edge_store,
            "us_row_scene_fast_path": self.us_row_scene_fast_path,
            "us_row_scene_full_path": self.us_row_scene_full_path,
            "us_syntax_spans": self.us_syntax_spans,
            "us_syntax_slice": self.us_syntax_slice,
            "us_syntax_highlight": self.us_syntax_highlight,
            "us_syntax_distribute": self.us_syntax_distribute,
            "us_syntax_store": self.us_syntax_store,
            "us_selection_rects": self.us_selection_rects,
            "us_caret_x": self.us_caret_x,
            "us_caret_stops": self.us_caret_stops,
            "us_caret_rect": self.us_caret_rect,
            "us_row_geom_cache": self.us_row_geom_cache,
            "us_row_geom_resolve": self.us_row_geom_resolve,
            "us_row_overlay": self.us_row_overlay,
            "us_frame_overlay_prepare": self.us_frame_overlay_prepare,
            "surface_rows_iterated": self.surface_rows_iterated,
            "surface_rows_with_rect": self.surface_rows_with_rect,
            "us_windowed_surface_paint_callback": self.us_windowed_surface_paint_callback,
            "us_windowed_surface_frame_lookup": self.us_windowed_surface_frame_lookup,
            "us_windowed_surface_hook": self.us_windowed_surface_hook,
            "us_windowed_surface_row_loop": self.us_windowed_surface_row_loop,
            "us_windowed_surface_row_rect": self.us_windowed_surface_row_rect,
            "us_windowed_surface_row_paint": self.us_windowed_surface_row_paint,
            "us_windowed_surface_non_row": self.us_windowed_surface_non_row,
            "us_windowed_surface_row_callback_gap": self.us_windowed_surface_row_callback_gap,
            "us_torture_autoscroll": self.us_torture_autoscroll,
            "us_torture_overlay": self.us_torture_overlay,
        })
    }
}

impl BundleStatsCodeEditorPaintPerfSummary {
    fn observe(&mut self, p: &BundleStatsCodeEditorPaintPerf) {
        self.frames = self.frames.saturating_add(1);
        self.sum.add_frame(p);
        self.max.max_frame(p);
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "frames": self.frames,
            "sum": self.sum.to_json(),
            "max": self.max.to_json(),
            "p50": self.p50.to_json(),
            "p95": self.p95.to_json(),
        })
    }
}

impl BundleStatsCodeEditorPaintPerf {
    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "frame_seq": self.frame_seq,
            "visible_start": self.visible_start,
            "visible_end": self.visible_end,
            "visible_rows": self.visible_rows,
            "cache_base_entries": self.cache_base_entries,
            "cache_frame_min_entries": self.cache_frame_min_entries,
            "cache_effective_entries": self.cache_effective_entries,
            "rows_painted": self.rows_painted,
            "rows_drew_rich": self.rows_drew_rich,
            "rows_scene_replayed": self.rows_scene_replayed,
            "rows_scene_prepaint_planned": self.rows_scene_prepaint_planned,
            "rows_scene_prepaint_plan_used": self.rows_scene_prepaint_plan_used,
            "rows_scene_stored": self.rows_scene_stored,
            "rows_scene_stored_at_visible_start": self.rows_scene_stored_at_visible_start,
            "rows_scene_stored_at_visible_end": self.rows_scene_stored_at_visible_end,
            "row_scene_ops_stored": self.row_scene_ops_stored,
            "rows_scene_prepaint_edge_stored": self.rows_scene_prepaint_edge_stored,
            "row_scene_prepaint_edge_ops_stored": self.row_scene_prepaint_edge_ops_stored,
            "rows_scene_prepaint_candidates": self.rows_scene_prepaint_candidates,
            "rows_scene_prepaint_skip_no_cache": self.rows_scene_prepaint_skip_no_cache,
            "rows_scene_prepaint_skip_unsupported_key": self.rows_scene_prepaint_skip_unsupported_key,
            "rows_scene_prepaint_skip_preedit": self.rows_scene_prepaint_skip_preedit,
            "rows_scene_prepaint_skip_syntax_empty": self.rows_scene_prepaint_skip_syntax_empty,
            "rows_scene_prepaint_skip_key_mismatch": self.rows_scene_prepaint_skip_key_mismatch,
            "rows_scene_fast_miss_no_entry": self.rows_scene_fast_miss_no_entry,
            "rows_scene_fast_miss_key_mismatch": self.rows_scene_fast_miss_key_mismatch,
            "rows_scene_full_miss_no_entry": self.rows_scene_full_miss_no_entry,
            "rows_scene_full_miss_key_mismatch": self.rows_scene_full_miss_key_mismatch,
            "quads_selection": self.quads_selection,
            "quads_caret": self.quads_caret,
            "syntax_rows_stored": self.syntax_rows_stored,
            "us_total": self.us_total,
            "us_row_text": self.us_row_text,
            "us_baseline_measure": self.us_baseline_measure,
            "us_row_content_resolve": self.us_row_content_resolve,
            "us_row_rich_cache_compare": self.us_row_rich_cache_compare,
            "us_row_geom_key": self.us_row_geom_key,
            "us_rich_materialize": self.us_rich_materialize,
            "us_text_draw": self.us_text_draw,
            "us_row_scene_key": self.us_row_scene_key,
            "us_row_scene_fast_probe": self.us_row_scene_fast_probe,
            "us_row_scene_full_probe": self.us_row_scene_full_probe,
            "us_row_scene_fast_key_compare": self.us_row_scene_fast_key_compare,
            "us_row_scene_full_key_compare": self.us_row_scene_full_key_compare,
            "us_row_scene_replay_touch": self.us_row_scene_replay_touch,
            "us_row_scene_replay_ops": self.us_row_scene_replay_ops,
            "us_row_scene_prepaint_plan": self.us_row_scene_prepaint_plan,
            "us_row_scene_prepaint_probe": self.us_row_scene_prepaint_probe,
            "us_row_scene_prepaint_key_compare": self.us_row_scene_prepaint_key_compare,
            "us_row_scene_capture_ops": self.us_row_scene_capture_ops,
            "us_row_scene_store": self.us_row_scene_store,
            "us_row_scene_prepaint_edge_store": self.us_row_scene_prepaint_edge_store,
            "us_row_scene_fast_path": self.us_row_scene_fast_path,
            "us_row_scene_full_path": self.us_row_scene_full_path,
            "us_syntax_spans": self.us_syntax_spans,
            "us_syntax_slice": self.us_syntax_slice,
            "us_syntax_highlight": self.us_syntax_highlight,
            "us_syntax_distribute": self.us_syntax_distribute,
            "us_syntax_store": self.us_syntax_store,
            "us_selection_rects": self.us_selection_rects,
            "us_caret_x": self.us_caret_x,
            "us_caret_stops": self.us_caret_stops,
            "us_caret_rect": self.us_caret_rect,
            "us_row_geom_cache": self.us_row_geom_cache,
            "us_row_geom_resolve": self.us_row_geom_resolve,
            "us_row_overlay": self.us_row_overlay,
            "us_frame_overlay_prepare": self.us_frame_overlay_prepare,
            "surface_rows_iterated": self.surface_rows_iterated,
            "surface_rows_with_rect": self.surface_rows_with_rect,
            "us_windowed_surface_paint_callback": self.us_windowed_surface_paint_callback,
            "us_windowed_surface_frame_lookup": self.us_windowed_surface_frame_lookup,
            "us_windowed_surface_hook": self.us_windowed_surface_hook,
            "us_windowed_surface_row_loop": self.us_windowed_surface_row_loop,
            "us_windowed_surface_row_rect": self.us_windowed_surface_row_rect,
            "us_windowed_surface_row_paint": self.us_windowed_surface_row_paint,
            "us_windowed_surface_non_row": self.us_windowed_surface_non_row,
            "us_windowed_surface_row_callback_gap": self.us_windowed_surface_row_callback_gap,
            "us_torture_autoscroll": self.us_torture_autoscroll,
            "us_torture_overlay": self.us_torture_overlay,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsScrollLayoutProfile {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) test_id: Option<String>,
    pub(super) axis: Option<String>,
    pub(super) pass: Option<String>,
    pub(super) probe_unbounded: bool,
    pub(super) children: u32,
    pub(super) available_w: Option<f32>,
    pub(super) available_h: Option<f32>,
    pub(super) desired_w: Option<f32>,
    pub(super) desired_h: Option<f32>,
    pub(super) content_w: Option<f32>,
    pub(super) content_h: Option<f32>,
    pub(super) post_layout_extents_mode: bool,
    pub(super) interactive_resize: bool,
    pub(super) direct_children_layout_invalidated: bool,
    pub(super) descendant_subtree_layout_dirty: bool,
    pub(super) force_barrier_child_root_relayout: bool,
    pub(super) phase_profiles: Vec<BundleStatsScrollLayoutPhaseProfile>,
    pub(super) measure_children_us: u64,
    pub(super) solve_barrier_us: u64,
    pub(super) layout_children_us: u64,
    pub(super) layout_children_first_pass_us: u64,
    pub(super) layout_child_first_pass_nodes_visited: u32,
    pub(super) layout_child_first_pass_nodes_performed: u32,
    pub(super) layout_child_first_pass_max_us: u64,
    pub(super) layout_child_first_pass_kind_profiles: Vec<BundleStatsScrollLayoutKindProfile>,
    pub(super) corrected_content_relayout: bool,
    pub(super) layout_children_corrected_content_us: u64,
    pub(super) layout_child_corrected_content_nodes_visited: u32,
    pub(super) layout_child_corrected_content_nodes_performed: u32,
    pub(super) layout_child_corrected_content_max_us: u64,
    pub(super) layout_child_corrected_content_kind_profiles:
        Vec<BundleStatsScrollLayoutKindProfile>,
    pub(super) layout_child_nodes_visited: u32,
    pub(super) layout_child_nodes_performed: u32,
    pub(super) layout_child_kind_profiles: Vec<BundleStatsScrollLayoutKindProfile>,
    pub(super) layout_child_max_us: u64,
    pub(super) layout_child_max_node: Option<u64>,
    pub(super) layout_child_max_invalidated: bool,
    pub(super) layout_child_max_subtree_dirty: bool,
    pub(super) layout_child_max_subtree_dirty_count: u32,
    pub(super) layout_child_max_nodes_visited: u32,
    pub(super) layout_child_max_nodes_performed: u32,
    pub(super) layout_child_max_bounds_changed: Option<bool>,
    pub(super) layout_child_max_bounds_size_changed: Option<bool>,
    pub(super) layout_child_max_input_matches_before: Option<bool>,
    pub(super) layout_child_max_input_size_matches_before: Option<bool>,
    pub(super) total_us: u64,
    pub(super) element_path: Option<String>,
    pub(super) role: Option<String>,
    pub(super) semantics_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsScrollLayoutKindProfile {
    pub(super) kind: Option<String>,
    pub(super) nodes: u32,
    pub(super) self_us: u64,
    pub(super) total_us: u64,
    pub(super) max_self_us: u64,
    pub(super) max_total_us: u64,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsScrollLayoutPhaseProfile {
    pub(super) phase: Option<String>,
    pub(super) us: u64,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutDirtyDescendant {
    pub(super) node: u64,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) element_path: Option<String>,
    pub(super) subtree_layout_dirty_count: u32,
    pub(super) source_root_node: Option<u64>,
    pub(super) source: Option<String>,
    pub(super) detail: Option<String>,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutRequestBuildRoot {
    pub(super) root_node: u64,
    pub(super) root_kind: Option<String>,
    pub(super) root_element: Option<u64>,
    pub(super) root_element_kind: Option<String>,
    pub(super) root_element_path: Option<String>,
    pub(super) elapsed_us: u64,
    pub(super) mode: Option<String>,
    pub(super) had_layout_engine_node: bool,
    pub(super) layout_invalidated: bool,
    pub(super) subtree_layout_dirty: bool,
    pub(super) subtree_layout_dirty_count: u32,
    pub(super) descendant_layout_dirty_count: u32,
    pub(super) needs_layout: bool,
    pub(super) is_translation_only: bool,
    pub(super) nodes_marked_seen: u32,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
    pub(super) dirty_descendants: Vec<BundleStatsLayoutDirtyDescendant>,
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
pub(super) struct BundleStatsCommandAvailabilityHotspot {
    pub(super) command: String,
    pub(super) route: String,
    pub(super) start_node: u64,
    pub(super) resolved_node: Option<u64>,
    pub(super) outcome: String,
    pub(super) elapsed_us: u64,
    pub(super) start_element: Option<u64>,
    pub(super) start_element_kind: Option<String>,
    pub(super) start_element_path: Option<String>,
    pub(super) resolved_element: Option<u64>,
    pub(super) resolved_element_kind: Option<String>,
    pub(super) resolved_element_path: Option<String>,
}

impl BundleStatsCommandAvailabilityHotspot {
    fn to_json(&self) -> Value {
        serde_json::json!({
            "command": self.command,
            "route": self.route,
            "start_node": self.start_node,
            "resolved_node": self.resolved_node,
            "outcome": self.outcome,
            "elapsed_us": self.elapsed_us,
            "start_element": self.start_element,
            "start_element_kind": self.start_element_kind,
            "start_element_path": self.start_element_path,
            "resolved_element": self.resolved_element,
            "resolved_element_kind": self.resolved_element_kind,
            "resolved_element_path": self.resolved_element_path,
        })
    }
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

impl BundleStatsPaintWidgetHotspot {
    fn is_canvas(&self) -> bool {
        self.element_kind.as_deref() == Some("Canvas")
    }

    fn to_json(&self) -> Value {
        let mut h_obj = Map::new();
        h_obj.insert("node".to_string(), Value::from(self.node));
        h_obj.insert(
            "element".to_string(),
            self.element.map(Value::from).unwrap_or(Value::Null),
        );
        h_obj.insert(
            "element_kind".to_string(),
            self.element_kind
                .clone()
                .map(Value::from)
                .unwrap_or(Value::Null),
        );
        h_obj.insert(
            "widget_type".to_string(),
            self.widget_type
                .clone()
                .map(Value::from)
                .unwrap_or(Value::Null),
        );
        h_obj.insert("paint_time_us".to_string(), Value::from(self.paint_time_us));
        h_obj.insert(
            "inclusive_time_us".to_string(),
            Value::from(self.inclusive_time_us),
        );
        h_obj.insert(
            "inclusive_scene_ops_delta".to_string(),
            Value::from(self.inclusive_scene_ops_delta),
        );
        h_obj.insert(
            "exclusive_scene_ops_delta".to_string(),
            Value::from(self.exclusive_scene_ops_delta),
        );
        h_obj.insert(
            "role".to_string(),
            self.role.clone().map(Value::from).unwrap_or(Value::Null),
        );
        h_obj.insert(
            "test_id".to_string(),
            self.test_id.clone().map(Value::from).unwrap_or(Value::Null),
        );
        Value::Object(h_obj)
    }
}

#[derive(Debug, Default, Clone)]
struct BundleStatsPaintWidgetHotspotSummary {
    sampled_top_n_per_frame: usize,
    frames_with_hotspots: u32,
    canvas: BundleStatsPaintWidgetHotspotClassSummary,
    non_canvas: BundleStatsPaintWidgetHotspotClassSummary,
}

#[derive(Debug, Default, Clone)]
struct BundleStatsPaintWidgetHotspotClassSummary {
    frames: u32,
    exclusive_us: Vec<u64>,
    inclusive_us: Vec<u64>,
    exclusive_scene_ops: Vec<u64>,
    inclusive_scene_ops: Vec<u64>,
    sampled_sum_exclusive_us: Vec<u64>,
    sampled_sum_inclusive_us: Vec<u64>,
    sampled_sum_exclusive_scene_ops: Vec<u64>,
    sampled_sum_inclusive_scene_ops: Vec<u64>,
    top: Option<BundleStatsPaintWidgetHotspot>,
}

impl BundleStatsPaintWidgetHotspotClassSummary {
    fn observe(
        &mut self,
        hotspot: &BundleStatsPaintWidgetHotspot,
        sampled_sum_exclusive_us: u64,
        sampled_sum_inclusive_us: u64,
        sampled_sum_exclusive_scene_ops: u64,
        sampled_sum_inclusive_scene_ops: u64,
    ) {
        self.frames = self.frames.saturating_add(1);
        self.exclusive_us.push(hotspot.paint_time_us);
        self.inclusive_us.push(hotspot.inclusive_time_us);
        self.exclusive_scene_ops
            .push(hotspot.exclusive_scene_ops_delta as u64);
        self.inclusive_scene_ops
            .push(hotspot.inclusive_scene_ops_delta as u64);
        self.sampled_sum_exclusive_us
            .push(sampled_sum_exclusive_us);
        self.sampled_sum_inclusive_us
            .push(sampled_sum_inclusive_us);
        self.sampled_sum_exclusive_scene_ops
            .push(sampled_sum_exclusive_scene_ops);
        self.sampled_sum_inclusive_scene_ops
            .push(sampled_sum_inclusive_scene_ops);

        let replace_top = self.top.as_ref().is_none_or(|top| {
            hotspot
                .paint_time_us
                .cmp(&top.paint_time_us)
                .then_with(|| hotspot.inclusive_time_us.cmp(&top.inclusive_time_us))
                .is_gt()
        });
        if replace_top {
            self.top = Some(hotspot.clone());
        }
    }

    fn p50_exclusive_us(&self) -> u64 {
        hotspot_percentile(&self.exclusive_us, 0.50)
    }

    fn p95_exclusive_us(&self) -> u64 {
        hotspot_percentile(&self.exclusive_us, 0.95)
    }

    fn max_exclusive_us(&self) -> u64 {
        self.exclusive_us.iter().copied().max().unwrap_or(0)
    }

    fn to_json(&self) -> Value {
        serde_json::json!({
            "frames": self.frames,
            "exclusive_us": crate::summarize_times_us(&self.exclusive_us),
            "inclusive_us": crate::summarize_times_us(&self.inclusive_us),
            "exclusive_scene_ops_delta": hotspot_summary_json(&self.exclusive_scene_ops),
            "inclusive_scene_ops_delta": hotspot_summary_json(&self.inclusive_scene_ops),
            "sampled_sum_exclusive_us": crate::summarize_times_us(&self.sampled_sum_exclusive_us),
            "sampled_sum_inclusive_us": crate::summarize_times_us(&self.sampled_sum_inclusive_us),
            "sampled_sum_exclusive_scene_ops_delta": hotspot_summary_json(&self.sampled_sum_exclusive_scene_ops),
            "sampled_sum_inclusive_scene_ops_delta": hotspot_summary_json(&self.sampled_sum_inclusive_scene_ops),
            "top": self.top.as_ref().map(BundleStatsPaintWidgetHotspot::to_json),
        })
    }
}

impl BundleStatsPaintWidgetHotspotSummary {
    fn observe_frame(&mut self, hotspots: &[BundleStatsPaintWidgetHotspot], sampled_top_n: usize) {
        self.sampled_top_n_per_frame = self.sampled_top_n_per_frame.max(sampled_top_n);
        if hotspots.is_empty() {
            return;
        }

        self.frames_with_hotspots = self.frames_with_hotspots.saturating_add(1);

        let mut top_canvas: Option<&BundleStatsPaintWidgetHotspot> = None;
        let mut top_non_canvas: Option<&BundleStatsPaintWidgetHotspot> = None;
        let mut canvas_sum_exclusive_us = 0u64;
        let mut canvas_sum_inclusive_us = 0u64;
        let mut canvas_sum_exclusive_scene_ops = 0u64;
        let mut canvas_sum_inclusive_scene_ops = 0u64;
        let mut non_canvas_sum_exclusive_us = 0u64;
        let mut non_canvas_sum_inclusive_us = 0u64;
        let mut non_canvas_sum_exclusive_scene_ops = 0u64;
        let mut non_canvas_sum_inclusive_scene_ops = 0u64;

        for hotspot in hotspots {
            let is_canvas = hotspot.is_canvas();
            if is_canvas {
                canvas_sum_exclusive_us =
                    canvas_sum_exclusive_us.saturating_add(hotspot.paint_time_us);
                canvas_sum_inclusive_us =
                    canvas_sum_inclusive_us.saturating_add(hotspot.inclusive_time_us);
                canvas_sum_exclusive_scene_ops = canvas_sum_exclusive_scene_ops
                    .saturating_add(hotspot.exclusive_scene_ops_delta as u64);
                canvas_sum_inclusive_scene_ops = canvas_sum_inclusive_scene_ops
                    .saturating_add(hotspot.inclusive_scene_ops_delta as u64);
            } else {
                non_canvas_sum_exclusive_us =
                    non_canvas_sum_exclusive_us.saturating_add(hotspot.paint_time_us);
                non_canvas_sum_inclusive_us =
                    non_canvas_sum_inclusive_us.saturating_add(hotspot.inclusive_time_us);
                non_canvas_sum_exclusive_scene_ops = non_canvas_sum_exclusive_scene_ops
                    .saturating_add(hotspot.exclusive_scene_ops_delta as u64);
                non_canvas_sum_inclusive_scene_ops = non_canvas_sum_inclusive_scene_ops
                    .saturating_add(hotspot.inclusive_scene_ops_delta as u64);
            }

            let top = if is_canvas { &mut top_canvas } else { &mut top_non_canvas };
            let replace = top.as_ref().is_none_or(|current| {
                hotspot
                    .paint_time_us
                    .cmp(&current.paint_time_us)
                    .then_with(|| hotspot.inclusive_time_us.cmp(&current.inclusive_time_us))
                    .is_gt()
            });
            if replace {
                *top = Some(hotspot);
            }
        }

        if let Some(hotspot) = top_canvas {
            self.canvas.observe(
                hotspot,
                canvas_sum_exclusive_us,
                canvas_sum_inclusive_us,
                canvas_sum_exclusive_scene_ops,
                canvas_sum_inclusive_scene_ops,
            );
        }
        if let Some(hotspot) = top_non_canvas {
            self.non_canvas.observe(
                hotspot,
                non_canvas_sum_exclusive_us,
                non_canvas_sum_inclusive_us,
                non_canvas_sum_exclusive_scene_ops,
                non_canvas_sum_inclusive_scene_ops,
            );
        }
    }

    fn has_samples(&self) -> bool {
        self.frames_with_hotspots > 0
    }

    fn canvas_minus_code_editor_p95_us_total(
        &self,
        code_editor: &BundleStatsCodeEditorPaintPerfSummary,
    ) -> Option<i64> {
        (self.canvas.frames > 0 && code_editor.frames > 0).then(|| {
            self.canvas.p95_exclusive_us() as i64 - code_editor.p95.us_total as i64
        })
    }

    fn canvas_minus_windowed_surface_callback_p95(
        &self,
        code_editor: &BundleStatsCodeEditorPaintPerfSummary,
    ) -> Option<i64> {
        (self.canvas.frames > 0 && code_editor.frames > 0).then(|| {
            self.canvas.p95_exclusive_us() as i64
                - code_editor.p95.us_windowed_surface_paint_callback as i64
        })
    }

    fn windowed_surface_callback_minus_code_editor_p95_us_total(
        &self,
        code_editor: &BundleStatsCodeEditorPaintPerfSummary,
    ) -> Option<i64> {
        (self.canvas.frames > 0 && code_editor.frames > 0).then(|| {
            code_editor.p95.us_windowed_surface_paint_callback as i64
                - code_editor.p95.us_total as i64
        })
    }

    fn windowed_surface_row_paint_minus_code_editor_p95_us_total(
        &self,
        code_editor: &BundleStatsCodeEditorPaintPerfSummary,
    ) -> Option<i64> {
        (self.canvas.frames > 0 && code_editor.frames > 0).then(|| {
            code_editor.p95.us_windowed_surface_row_paint as i64
                - code_editor.p95.us_total as i64
        })
    }

    fn windowed_surface_callback_minus_row_paint_p95(
        &self,
        code_editor: &BundleStatsCodeEditorPaintPerfSummary,
    ) -> Option<i64> {
        (self.canvas.frames > 0 && code_editor.frames > 0).then(|| {
            code_editor.p95.us_windowed_surface_paint_callback as i64
                - code_editor.p95.us_windowed_surface_row_paint as i64
        })
    }

    fn gap_per_row_ns(gap_us: i64, rows: u64) -> Option<i64> {
        if rows == 0 {
            return None;
        }

        let rows = i64::try_from(rows).unwrap_or(i64::MAX);
        Some(gap_us.saturating_mul(1_000) / rows)
    }

    fn windowed_surface_callback_minus_row_paint_p95_per_row_ns(
        &self,
        code_editor: &BundleStatsCodeEditorPaintPerfSummary,
    ) -> Option<i64> {
        let gap = self.windowed_surface_callback_minus_row_paint_p95(code_editor)?;
        Self::gap_per_row_ns(gap, code_editor.p95.surface_rows_with_rect)
    }

    fn windowed_surface_row_callback_gap_p95_per_row_ns(
        code_editor: &BundleStatsCodeEditorPaintPerfSummary,
    ) -> Option<i64> {
        if code_editor.frames == 0 {
            return None;
        }

        Self::gap_per_row_ns(
            code_editor.p95.us_windowed_surface_row_callback_gap as i64,
            code_editor.p95.surface_rows_with_rect,
        )
    }

    fn code_editor_windowed_surface_p95_json(
        code_editor: &BundleStatsCodeEditorPaintPerfSummary,
    ) -> Value {
        if code_editor.frames == 0 {
            return Value::Null;
        }

        serde_json::json!({
            "paint_callback": code_editor.p95.us_windowed_surface_paint_callback,
            "frame_lookup": code_editor.p95.us_windowed_surface_frame_lookup,
            "hook": code_editor.p95.us_windowed_surface_hook,
            "row_loop": code_editor.p95.us_windowed_surface_row_loop,
            "row_rect": code_editor.p95.us_windowed_surface_row_rect,
            "row_paint": code_editor.p95.us_windowed_surface_row_paint,
            "non_row": code_editor.p95.us_windowed_surface_non_row,
            "row_callback_gap": code_editor.p95.us_windowed_surface_row_callback_gap,
            "rows_with_rect": code_editor.p95.surface_rows_with_rect,
        })
    }

    fn to_json(&self, code_editor: &BundleStatsCodeEditorPaintPerfSummary) -> Value {
        serde_json::json!({
            "sampled_top_n_per_frame": self.sampled_top_n_per_frame as u64,
            "frames_with_hotspots": self.frames_with_hotspots,
            "canvas": self.canvas.to_json(),
            "non_canvas": self.non_canvas.to_json(),
            "gap_to_code_editor_p95": {
                "canvas_exclusive_minus_us_total": self.canvas_minus_code_editor_p95_us_total(code_editor),
                "canvas_exclusive_minus_windowed_surface_paint_callback": self.canvas_minus_windowed_surface_callback_p95(code_editor),
                "windowed_surface_paint_callback_minus_us_total": self.windowed_surface_callback_minus_code_editor_p95_us_total(code_editor),
                "windowed_surface_row_paint_minus_us_total": self.windowed_surface_row_paint_minus_code_editor_p95_us_total(code_editor),
                "windowed_surface_paint_callback_minus_row_paint": self.windowed_surface_callback_minus_row_paint_p95(code_editor),
                "windowed_surface_paint_callback_minus_row_paint_per_row_ns": self.windowed_surface_callback_minus_row_paint_p95_per_row_ns(code_editor),
                "windowed_surface_row_callback_gap_per_row_ns": Self::windowed_surface_row_callback_gap_p95_per_row_ns(code_editor),
            },
            "code_editor_windowed_surface_p95": Self::code_editor_windowed_surface_p95_json(code_editor),
        })
    }
}

fn hotspot_percentile(values: &[u64], percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    crate::percentile_nearest_rank_sorted(&sorted, percentile)
}

fn hotspot_summary_json(values: &[u64]) -> Value {
    if values.is_empty() {
        return serde_json::json!({
            "min": 0,
            "p50": 0,
            "p95": 0,
            "max": 0,
        });
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    serde_json::json!({
        "min": sorted.first().copied().unwrap_or(0),
        "p50": crate::percentile_nearest_rank_sorted(&sorted, 0.50),
        "p95": crate::percentile_nearest_rank_sorted(&sorted, 0.95),
        "max": sorted.last().copied().unwrap_or(0),
    })
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
    pub(super) layout_dependency: Option<String>,
    pub(super) contained_relayout_in_frame: bool,
    pub(super) paint_replayed_ops: u32,
    pub(super) reuse_reason: Option<String>,
    pub(super) root_in_semantics: Option<bool>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
    pub(super) boundary_kind: Option<String>,
    pub(super) boundary_layout_dependency: Option<String>,
    pub(super) boundary_build_outcome: Option<String>,
    pub(super) boundary_reuse_reason: Option<String>,
    pub(super) boundary_layout_outcome: Option<String>,
    pub(super) boundary_prepaint_owner: Option<String>,
    pub(super) boundary_paint_outcome: Option<String>,
}

fn push_cache_root_boundary_summary(s: &mut String, c: &BundleStatsCacheRoot) {
    let has_boundary = c.boundary_kind.is_some()
        || c.boundary_layout_dependency.is_some()
        || c.boundary_build_outcome.is_some()
        || c.boundary_reuse_reason.is_some()
        || c.boundary_layout_outcome.is_some()
        || c.boundary_prepaint_owner.is_some()
        || c.boundary_paint_outcome.is_some();
    if !has_boundary {
        return;
    }

    s.push_str(" boundary(");
    s.push_str(c.boundary_kind.as_deref().unwrap_or("?"));
    if let Some(value) = c.boundary_layout_dependency.as_deref() {
        s.push_str(&format!(" dep={value}"));
    }
    if let Some(value) = c.boundary_build_outcome.as_deref() {
        s.push_str(&format!(" build={value}"));
    }
    if let Some(value) = c.boundary_reuse_reason.as_deref() {
        s.push_str(&format!(" reuse_reason={value}"));
    }
    if let Some(value) = c.boundary_layout_outcome.as_deref() {
        s.push_str(&format!(" layout={value}"));
    }
    if let Some(value) = c.boundary_prepaint_owner.as_deref() {
        s.push_str(&format!(" prepaint={value}"));
    }
    if let Some(value) = c.boundary_paint_outcome.as_deref() {
        s.push_str(&format!(" paint={value}"));
    }
    s.push(')');
}

fn insert_cache_root_boundary_json(c_obj: &mut Map<String, Value>, c: &BundleStatsCacheRoot) {
    let has_boundary = c.boundary_kind.is_some()
        || c.boundary_layout_dependency.is_some()
        || c.boundary_build_outcome.is_some()
        || c.boundary_reuse_reason.is_some()
        || c.boundary_layout_outcome.is_some()
        || c.boundary_prepaint_owner.is_some()
        || c.boundary_paint_outcome.is_some();
    if !has_boundary {
        return;
    }

    let mut boundary = Map::new();
    boundary.insert(
        "kind".to_string(),
        c.boundary_kind.clone().map(Value::from).unwrap_or(Value::Null),
    );
    boundary.insert(
        "layout_dependency".to_string(),
        c.boundary_layout_dependency
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    boundary.insert(
        "build_outcome".to_string(),
        c.boundary_build_outcome
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    boundary.insert(
        "reuse_reason".to_string(),
        c.boundary_reuse_reason
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    boundary.insert(
        "layout_outcome".to_string(),
        c.boundary_layout_outcome
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    boundary.insert(
        "prepaint_owner".to_string(),
        c.boundary_prepaint_owner
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    boundary.insert(
        "paint_outcome".to_string(),
        c.boundary_paint_outcome
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    c_obj.insert("boundary".to_string(), Value::Object(boundary));
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineSolve {
    pub(super) root_node: u64,
    pub(super) root_element: Option<u64>,
    pub(super) root_element_kind: Option<String>,
    pub(super) root_element_path: Option<String>,
    pub(super) solve_time_us: u64,
    pub(super) solve_profile: Option<BundleStatsLayoutEngineSolveProfile>,
    pub(super) clean_geometry_solve_skip_rejection:
        Option<BundleStatsCleanGeometrySolveSkipRejection>,
    pub(super) measure_calls: u64,
    pub(super) measure_cache_hits: u64,
    pub(super) measure_time_us: u64,
    pub(super) top_measures: Vec<BundleStatsLayoutEngineMeasureHotspot>,
    pub(super) root_role: Option<String>,
    pub(super) root_test_id: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsCleanGeometrySolveSkipRejection {
    pub(super) reason: String,
    pub(super) detail: Option<String>,
    pub(super) node: Option<u64>,
    pub(super) element: Option<u64>,
    pub(super) element_kind: Option<String>,
    pub(super) element_path: Option<String>,
    pub(super) role: Option<String>,
    pub(super) test_id: Option<String>,
}

pub(super) fn clean_geometry_solve_skip_rejection_to_json(
    rejection: &BundleStatsCleanGeometrySolveSkipRejection,
) -> Value {
    let mut obj = Map::new();
    obj.insert("reason".to_string(), Value::from(rejection.reason.clone()));
    obj.insert(
        "detail".to_string(),
        rejection
            .detail
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    obj.insert(
        "node".to_string(),
        rejection.node.map(Value::from).unwrap_or(Value::Null),
    );
    obj.insert(
        "element".to_string(),
        rejection.element.map(Value::from).unwrap_or(Value::Null),
    );
    obj.insert(
        "element_kind".to_string(),
        rejection
            .element_kind
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    obj.insert(
        "element_path".to_string(),
        rejection
            .element_path
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    obj.insert(
        "role".to_string(),
        rejection
            .role
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    obj.insert(
        "test_id".to_string(),
        rejection
            .test_id
            .clone()
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    Value::Object(obj)
}

#[derive(Debug, Default, Clone)]
pub(super) struct BundleStatsLayoutEngineSolveProfile {
    pub(super) reason: String,
    pub(super) available_w_kind: String,
    pub(super) available_h_kind: String,
    pub(super) available_w: Option<f64>,
    pub(super) available_h: Option<f64>,
    pub(super) previous_available_w_kind: Option<String>,
    pub(super) previous_available_h_kind: Option<String>,
    pub(super) previous_available_w: Option<f64>,
    pub(super) previous_available_h: Option<f64>,
    pub(super) available_w_delta: Option<f64>,
    pub(super) available_h_delta: Option<f64>,
    pub(super) scale_factor: f64,
    pub(super) previous_scale_factor: Option<f64>,
    pub(super) scale_factor_delta: Option<f64>,
    pub(super) previous_frame_delta: Option<u64>,
    pub(super) batch_roots: u64,
    pub(super) subtree_nodes: u64,
    pub(super) flex_wrap_patch_time_us: u64,
    pub(super) flex_wrap_patch_visited_nodes: u64,
    pub(super) flex_wrap_patch_wrap_nodes: u64,
    pub(super) flex_wrap_patch_candidate_children: u64,
    pub(super) flex_wrap_patch_probes: u64,
    pub(super) flex_wrap_patch_mutations: u64,
    pub(super) flex_wrap_patch_skipped_no_wrap_descendant: bool,
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

    fn dispatch_accounted_time_us(row: &BundleStatsSnapshotRow) -> u64 {
        row.hit_test_time_us
            .saturating_add(row.dispatch_hover_update_time_us)
            .saturating_add(row.dispatch_input_state_update_time_us)
            .saturating_add(row.dispatch_context_build_time_us)
            .saturating_add(row.dispatch_prelude_time_us)
            .saturating_add(row.dispatch_pointer_arbitration_time_us)
            .saturating_add(row.dispatch_pointer_target_routing_time_us)
            .saturating_add(row.dispatch_post_widget_control_flow_time_us)
            .saturating_add(row.dispatch_scroll_handle_invalidation_time_us)
            .saturating_add(row.dispatch_active_layers_time_us)
            .saturating_add(row.dispatch_input_context_time_us)
            .saturating_add(row.dispatch_event_chain_build_time_us)
            .saturating_add(row.dispatch_widget_capture_time_us)
            .saturating_add(row.dispatch_widget_bubble_time_us)
            .saturating_add(row.dispatch_cursor_query_time_us)
            .saturating_add(row.dispatch_pointer_move_layer_observers_time_us)
            .saturating_add(row.dispatch_synth_hover_observer_time_us)
            .saturating_add(row.dispatch_cursor_effect_time_us)
            .saturating_add(row.dispatch_post_dispatch_snapshot_time_us)
    }

    fn dispatch_unattributed_time_us(row: &BundleStatsSnapshotRow) -> u64 {
        row.dispatch_time_us
            .saturating_sub(Self::dispatch_accounted_time_us(row))
    }

    fn dispatch_runtime_wrapper_time_us(row: &BundleStatsSnapshotRow) -> u64 {
        if row.dispatch_inner_body_time_us == 0 {
            return 0;
        }
        row.dispatch_time_us
            .saturating_sub(row.dispatch_inner_body_time_us)
    }

    fn dispatch_inner_body_unattributed_time_us(row: &BundleStatsSnapshotRow) -> u64 {
        let body_time = if row.dispatch_inner_body_time_us == 0 {
            row.dispatch_time_us
        } else {
            row.dispatch_inner_body_time_us
        };
        body_time.saturating_sub(Self::dispatch_accounted_time_us(row))
    }

    fn print_dispatch_breakdown_row(row: &BundleStatsSnapshotRow, label: &str) {
        if row.dispatch_time_us == 0 {
            return;
        }

        let accounted = Self::dispatch_accounted_time_us(row);
        let unattributed = row.dispatch_time_us.saturating_sub(accounted);
        let runtime_wrapper = Self::dispatch_runtime_wrapper_time_us(row);
        let body_unattributed = Self::dispatch_inner_body_unattributed_time_us(row);
        println!(
            "    {label}.us(total/inner_body/accounted/unattributed/body_unattributed/runtime_wrapper/hit_test/hover_update/input_state/context_build/prelude/pointer_arbitration/pointer_target_routing/post_widget_control_flow/scroll_inv/active_layers/input_ctx/chain/capture/bubble/cursor_query/pointer_observers/synth_hover/cursor_effect/post_snapshot)={}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{} events(pointer/timer/other)={}/{}/{} event_time(pointer/timer/other)={}/{}/{}",
            row.dispatch_time_us,
            row.dispatch_inner_body_time_us,
            accounted,
            unattributed,
            body_unattributed,
            runtime_wrapper,
            row.hit_test_time_us,
            row.dispatch_hover_update_time_us,
            row.dispatch_input_state_update_time_us,
            row.dispatch_context_build_time_us,
            row.dispatch_prelude_time_us,
            row.dispatch_pointer_arbitration_time_us,
            row.dispatch_pointer_target_routing_time_us,
            row.dispatch_post_widget_control_flow_time_us,
            row.dispatch_scroll_handle_invalidation_time_us,
            row.dispatch_active_layers_time_us,
            row.dispatch_input_context_time_us,
            row.dispatch_event_chain_build_time_us,
            row.dispatch_widget_capture_time_us,
            row.dispatch_widget_bubble_time_us,
            row.dispatch_cursor_query_time_us,
            row.dispatch_pointer_move_layer_observers_time_us,
            row.dispatch_synth_hover_observer_time_us,
            row.dispatch_cursor_effect_time_us,
            row.dispatch_post_dispatch_snapshot_time_us,
            row.dispatch_pointer_events,
            row.dispatch_timer_events,
            row.dispatch_other_events,
            row.dispatch_pointer_event_time_us,
            row.dispatch_timer_event_time_us,
            row.dispatch_other_event_time_us
        );
    }

    fn print_code_editor_paint_perf_summary(&self) {
        let p = &self.code_editor_paint_perf;
        if p.frames == 0 {
            return;
        }

        println!(
            "code_editor.paint_perf frames={} sum.rows(painted/replayed/prepaint_planned/prepaint_used/stored/row_ops/prepaint_edge_stored/prepaint_edge_ops/rich/syntax_stored)={}/{}/{}/{}/{}/{}/{}/{}/{}/{} sum.quads(selection/caret)={}/{} max.rows(painted/replayed/prepaint_planned/prepaint_used/stored/row_ops/prepaint_edge_stored/prepaint_edge_ops)={}/{}/{}/{}/{}/{}/{}/{}",
            p.frames,
            p.sum.rows_painted,
            p.sum.rows_scene_replayed,
            p.sum.rows_scene_prepaint_planned,
            p.sum.rows_scene_prepaint_plan_used,
            p.sum.rows_scene_stored,
            p.sum.row_scene_ops_stored,
            p.sum.rows_scene_prepaint_edge_stored,
            p.sum.row_scene_prepaint_edge_ops_stored,
            p.sum.rows_drew_rich,
            p.sum.syntax_rows_stored,
            p.sum.quads_selection,
            p.sum.quads_caret,
            p.max.rows_painted,
            p.max.rows_scene_replayed,
            p.max.rows_scene_prepaint_planned,
            p.max.rows_scene_prepaint_plan_used,
            p.max.rows_scene_stored,
            p.max.row_scene_ops_stored,
            p.max.rows_scene_prepaint_edge_stored,
            p.max.row_scene_prepaint_edge_ops_stored,
        );
        println!(
            "code_editor.paint_perf sum.us(content/row_text/geom_key/scene_key/rich_cmp/fast_key_cmp/full_key_cmp/syntax_spans/text/rich/fast_path/full_path)={}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}",
            p.sum.us_row_content_resolve,
            p.sum.us_row_text,
            p.sum.us_row_geom_key,
            p.sum.us_row_scene_key,
            p.sum.us_row_rich_cache_compare,
            p.sum.us_row_scene_fast_key_compare,
            p.sum.us_row_scene_full_key_compare,
            p.sum.us_syntax_spans,
            p.sum.us_text_draw,
            p.sum.us_rich_materialize,
            p.sum.us_row_scene_fast_path,
            p.sum.us_row_scene_full_path,
        );
        println!(
            "code_editor.paint_perf sum.us(total/prepaint_plan/prepaint_probe/prepaint_key_compare/replay_touch/replay_ops/capture_ops/store/prepaint_edge_store/fast_probe/full_probe/geom_cache/geom_resolve/overlay/frame_overlay)={}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}",
            p.sum.us_total,
            p.sum.us_row_scene_prepaint_plan,
            p.sum.us_row_scene_prepaint_probe,
            p.sum.us_row_scene_prepaint_key_compare,
            p.sum.us_row_scene_replay_touch,
            p.sum.us_row_scene_replay_ops,
            p.sum.us_row_scene_capture_ops,
            p.sum.us_row_scene_store,
            p.sum.us_row_scene_prepaint_edge_store,
            p.sum.us_row_scene_fast_probe,
            p.sum.us_row_scene_full_probe,
            p.sum.us_row_geom_cache,
            p.sum.us_row_geom_resolve,
            p.sum.us_row_overlay,
            p.sum.us_frame_overlay_prepare,
        );
        println!(
            "code_editor.paint_perf sum.surface(rows_iterated/rows_with_rect)={}/{} sum.us(surface_total/frame_lookup/hook/row_loop/row_rect/row_paint/non_row/row_callback_gap)={}/{}/{}/{}/{}/{}/{}/{}",
            p.sum.surface_rows_iterated,
            p.sum.surface_rows_with_rect,
            p.sum.us_windowed_surface_paint_callback,
            p.sum.us_windowed_surface_frame_lookup,
            p.sum.us_windowed_surface_hook,
            p.sum.us_windowed_surface_row_loop,
            p.sum.us_windowed_surface_row_rect,
            p.sum.us_windowed_surface_row_paint,
            p.sum.us_windowed_surface_non_row,
            p.sum.us_windowed_surface_row_callback_gap,
        );
        println!(
            "code_editor.paint_perf sum.us(torture_autoscroll/torture_overlay)={}/{}",
            p.sum.us_torture_autoscroll,
            p.sum.us_torture_overlay,
        );
        println!(
            "code_editor.paint_perf sum.rows(scene_store_start/end,prepaint_candidates/no_cache/unsupported/preedit/syntax_empty/key_mismatch,fast_miss_no_entry/key_mismatch,full_miss_no_entry/key_mismatch)={}/{}, {}/{}/{}/{}/{}/{}, {}/{}, {}/{}",
            p.sum.rows_scene_stored_at_visible_start,
            p.sum.rows_scene_stored_at_visible_end,
            p.sum.rows_scene_prepaint_candidates,
            p.sum.rows_scene_prepaint_skip_no_cache,
            p.sum.rows_scene_prepaint_skip_unsupported_key,
            p.sum.rows_scene_prepaint_skip_preedit,
            p.sum.rows_scene_prepaint_skip_syntax_empty,
            p.sum.rows_scene_prepaint_skip_key_mismatch,
            p.sum.rows_scene_fast_miss_no_entry,
            p.sum.rows_scene_fast_miss_key_mismatch,
            p.sum.rows_scene_full_miss_no_entry,
            p.sum.rows_scene_full_miss_key_mismatch,
        );
        println!(
            "code_editor.paint_perf p50/p95.us(total/prepaint_plan/prepaint_probe/prepaint_key_compare/content/row_text/geom_key/scene_key/rich_cmp/fast_key_cmp/text/fast_path/surface_total/surface_non_row/surface_row_callback_gap/torture_autoscroll/torture_overlay)={}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}, {}/{}",
            p.p50.us_total,
            p.p95.us_total,
            p.p50.us_row_scene_prepaint_plan,
            p.p95.us_row_scene_prepaint_plan,
            p.p50.us_row_scene_prepaint_probe,
            p.p95.us_row_scene_prepaint_probe,
            p.p50.us_row_scene_prepaint_key_compare,
            p.p95.us_row_scene_prepaint_key_compare,
            p.p50.us_row_content_resolve,
            p.p95.us_row_content_resolve,
            p.p50.us_row_text,
            p.p95.us_row_text,
            p.p50.us_row_geom_key,
            p.p95.us_row_geom_key,
            p.p50.us_row_scene_key,
            p.p95.us_row_scene_key,
            p.p50.us_row_rich_cache_compare,
            p.p95.us_row_rich_cache_compare,
            p.p50.us_row_scene_fast_key_compare,
            p.p95.us_row_scene_fast_key_compare,
            p.p50.us_text_draw,
            p.p95.us_text_draw,
            p.p50.us_row_scene_fast_path,
            p.p95.us_row_scene_fast_path,
            p.p50.us_windowed_surface_paint_callback,
            p.p95.us_windowed_surface_paint_callback,
            p.p50.us_windowed_surface_non_row,
            p.p95.us_windowed_surface_non_row,
            p.p50.us_windowed_surface_row_callback_gap,
            p.p95.us_windowed_surface_row_callback_gap,
            p.p50.us_torture_autoscroll,
            p.p95.us_torture_autoscroll,
            p.p50.us_torture_overlay,
            p.p95.us_torture_overlay,
        );
    }

    fn print_paint_widget_hotspot_summary(&self) {
        let p = &self.paint_widget_hotspot_summary;
        if !p.has_samples() {
            return;
        }

        let canvas_gap_total = p
            .canvas_minus_code_editor_p95_us_total(&self.code_editor_paint_perf)
            .map_or_else(|| "n/a".to_string(), |v| v.to_string());
        let canvas_gap_surface = p
            .canvas_minus_windowed_surface_callback_p95(&self.code_editor_paint_perf)
            .map_or_else(|| "n/a".to_string(), |v| v.to_string());
        let surface_gap_total = p
            .windowed_surface_callback_minus_code_editor_p95_us_total(
                &self.code_editor_paint_perf,
            )
            .map_or_else(|| "n/a".to_string(), |v| v.to_string());
        let surface_row_gap_total = p
            .windowed_surface_row_paint_minus_code_editor_p95_us_total(
                &self.code_editor_paint_perf,
            )
            .map_or_else(|| "n/a".to_string(), |v| v.to_string());
        let surface_callback_gap_row = p
            .windowed_surface_callback_minus_row_paint_p95(&self.code_editor_paint_perf)
            .map_or_else(|| "n/a".to_string(), |v| v.to_string());
        let surface_callback_gap_row_per_row = p
            .windowed_surface_callback_minus_row_paint_p95_per_row_ns(
                &self.code_editor_paint_perf,
            )
            .map_or_else(|| "n/a".to_string(), |v| v.to_string());
        let surface_row_callback_gap_per_row =
            BundleStatsPaintWidgetHotspotSummary::windowed_surface_row_callback_gap_p95_per_row_ns(
                &self.code_editor_paint_perf,
            )
            .map_or_else(|| "n/a".to_string(), |v| v.to_string());

        println!(
            "paint_widget.hotspots sample_top_n={} frames={} canvas_frames={} non_canvas_frames={} canvas.top_exclusive_us(p50/p95/max)={}/{}/{} canvas.sampled_sum_exclusive_us(p50/p95/max)={}/{}/{} non_canvas.top_exclusive_us(p50/p95/max)={}/{}/{} non_canvas.sampled_sum_exclusive_us(p50/p95/max)={}/{}/{} canvas.gap_p95_us(code_editor_total/surface_callback)={}/{}",
            p.sampled_top_n_per_frame,
            p.frames_with_hotspots,
            p.canvas.frames,
            p.non_canvas.frames,
            p.canvas.p50_exclusive_us(),
            p.canvas.p95_exclusive_us(),
            p.canvas.max_exclusive_us(),
            hotspot_percentile(&p.canvas.sampled_sum_exclusive_us, 0.50),
            hotspot_percentile(&p.canvas.sampled_sum_exclusive_us, 0.95),
            p.canvas
                .sampled_sum_exclusive_us
                .iter()
                .copied()
                .max()
                .unwrap_or(0),
            p.non_canvas.p50_exclusive_us(),
            p.non_canvas.p95_exclusive_us(),
            p.non_canvas.max_exclusive_us(),
            hotspot_percentile(&p.non_canvas.sampled_sum_exclusive_us, 0.50),
            hotspot_percentile(&p.non_canvas.sampled_sum_exclusive_us, 0.95),
            p.non_canvas
                .sampled_sum_exclusive_us
                .iter()
                .copied()
                .max()
                .unwrap_or(0),
            canvas_gap_total,
            canvas_gap_surface,
        );
        if self.code_editor_paint_perf.frames > 0 {
            let code_editor = &self.code_editor_paint_perf.p95;
            println!(
                "paint_widget.hotspots code_editor.surface_p95_us(callback/row_paint/non_row/row_callback_gap/hook)={}/{}/{}/{}/{} surface.gap_p95_us(callback_minus_total/row_paint_minus_total/callback_minus_row_paint)={}/{}/{} surface.gap_p95_per_row_ns(callback_minus_row_paint/row_callback_gap)={}/{}",
                code_editor.us_windowed_surface_paint_callback,
                code_editor.us_windowed_surface_row_paint,
                code_editor.us_windowed_surface_non_row,
                code_editor.us_windowed_surface_row_callback_gap,
                code_editor.us_windowed_surface_hook,
                surface_gap_total,
                surface_row_gap_total,
                surface_callback_gap_row,
                surface_callback_gap_row_per_row,
                surface_row_callback_gap_per_row,
            );
        }
    }

    fn print_code_editor_paint_perf_row(row: &BundleStatsSnapshotRow) {
        let Some(p) = row.code_editor_paint_perf.as_ref() else {
            return;
        };

        println!(
            "    code_editor.paint_perf frame_seq={} visible(start/end/rows)={}/{}/{} cache(base/min/effective)={}/{}/{} rows(painted/replayed/prepaint_planned/prepaint_used/stored/row_ops/prepaint_edge_stored/prepaint_edge_ops/rich/syntax_stored)={}/{}/{}/{}/{}/{}/{}/{}/{}/{} quads(selection/caret)={}/{}",
            p.frame_seq,
            p.visible_start,
            p.visible_end,
            p.visible_rows,
            p.cache_base_entries,
            p.cache_frame_min_entries,
            p.cache_effective_entries,
            p.rows_painted,
            p.rows_scene_replayed,
            p.rows_scene_prepaint_planned,
            p.rows_scene_prepaint_plan_used,
            p.rows_scene_stored,
            p.row_scene_ops_stored,
            p.rows_scene_prepaint_edge_stored,
            p.row_scene_prepaint_edge_ops_stored,
            p.rows_drew_rich,
            p.syntax_rows_stored,
            p.quads_selection,
            p.quads_caret,
        );
        println!(
            "    code_editor.paint_perf.us(total/prepaint_plan/prepaint_probe/prepaint_key_compare/content/row_text/text/rich/geom_key/scene_key/rich_cmp/fast_key_cmp/full_key_cmp/replay_touch/replay_ops/capture_ops/store/prepaint_edge_store/fast_probe/full_probe/fast_path/full_path/syntax_spans/geom_cache/geom_resolve/overlay/frame_overlay/torture_autoscroll/torture_overlay)={}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}/{}",
            p.us_total,
            p.us_row_scene_prepaint_plan,
            p.us_row_scene_prepaint_probe,
            p.us_row_scene_prepaint_key_compare,
            p.us_row_content_resolve,
            p.us_row_text,
            p.us_text_draw,
            p.us_rich_materialize,
            p.us_row_geom_key,
            p.us_row_scene_key,
            p.us_row_rich_cache_compare,
            p.us_row_scene_fast_key_compare,
            p.us_row_scene_full_key_compare,
            p.us_row_scene_replay_touch,
            p.us_row_scene_replay_ops,
            p.us_row_scene_capture_ops,
            p.us_row_scene_store,
            p.us_row_scene_prepaint_edge_store,
            p.us_row_scene_fast_probe,
            p.us_row_scene_full_probe,
            p.us_row_scene_fast_path,
            p.us_row_scene_full_path,
            p.us_syntax_spans,
            p.us_row_geom_cache,
            p.us_row_geom_resolve,
            p.us_row_overlay,
            p.us_frame_overlay_prepare,
            p.us_torture_autoscroll,
            p.us_torture_overlay,
        );
        println!(
            "    code_editor.paint_perf.surface rows(iterated/with_rect)={}/{} us(total/frame_lookup/hook/row_loop/row_rect/row_paint/non_row/row_callback_gap)={}/{}/{}/{}/{}/{}/{}/{}",
            p.surface_rows_iterated,
            p.surface_rows_with_rect,
            p.us_windowed_surface_paint_callback,
            p.us_windowed_surface_frame_lookup,
            p.us_windowed_surface_hook,
            p.us_windowed_surface_row_loop,
            p.us_windowed_surface_row_rect,
            p.us_windowed_surface_row_paint,
            p.us_windowed_surface_non_row,
            p.us_windowed_surface_row_callback_gap,
        );
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
                "dispatch attribution p50/p95/max (us): accounted={}/{}/{} unattributed={}/{}/{} body_unattributed={}/{}/{} runtime_wrapper={}/{}/{}",
                self.p50_dispatch_accounted_time_us,
                self.p95_dispatch_accounted_time_us,
                self.max_dispatch_accounted_time_us,
                self.p50_dispatch_unattributed_time_us,
                self.p95_dispatch_unattributed_time_us,
                self.max_dispatch_unattributed_time_us,
                self.p50_dispatch_inner_body_unattributed_time_us,
                self.p95_dispatch_inner_body_unattributed_time_us,
                self.max_dispatch_inner_body_unattributed_time_us,
                self.p50_dispatch_runtime_wrapper_time_us,
                self.p95_dispatch_runtime_wrapper_time_us,
                self.max_dispatch_runtime_wrapper_time_us
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
            if self.p95_renderer_prepare_text_collect_pin_keys_us > 0
                || self.max_renderer_prepare_text_collect_pin_keys_us > 0
                || self.p95_renderer_prepare_text_bucket_delta_us > 0
                || self.max_renderer_prepare_text_bucket_delta_us > 0
                || self.p95_renderer_prepare_text_prewarm_us > 0
                || self.max_renderer_prepare_text_prewarm_us > 0
                || self.p95_renderer_prepare_text_pin_bucket_update_us > 0
                || self.max_renderer_prepare_text_pin_bucket_update_us > 0
                || self.p95_renderer_prepare_text_flush_uploads_us > 0
                || self.max_renderer_prepare_text_flush_uploads_us > 0
            {
                println!(
                    "renderer text_prepare p95/max (us): collect_pin_keys={}/{} bucket_delta={}/{} prewarm={}/{} pin_update={}/{} flush={}/{}",
                    self.p95_renderer_prepare_text_collect_pin_keys_us,
                    self.max_renderer_prepare_text_collect_pin_keys_us,
                    self.p95_renderer_prepare_text_bucket_delta_us,
                    self.max_renderer_prepare_text_bucket_delta_us,
                    self.p95_renderer_prepare_text_prewarm_us,
                    self.max_renderer_prepare_text_prewarm_us,
                    self.p95_renderer_prepare_text_pin_bucket_update_us,
                    self.max_renderer_prepare_text_pin_bucket_update_us,
                    self.p95_renderer_prepare_text_flush_uploads_us,
                    self.max_renderer_prepare_text_flush_uploads_us,
                );
            }
        }
        self.print_code_editor_paint_perf_summary();
        self.print_paint_widget_hotspot_summary();
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
                || row.renderer_uniform_bytes > 0
                || row.renderer_instance_bytes > 0
                || row.renderer_vertex_bytes > 0
                || row.renderer_encode_scene_stack_us > 0
                || row.renderer_encode_scene_clip_us > 0
                || row.renderer_encode_scene_mask_us > 0
                || row.renderer_encode_scene_effect_us > 0
                || row.renderer_encode_scene_quad_us > 0
                || row.renderer_encode_scene_image_us > 0
                || row.renderer_encode_scene_text_us > 0
                || row.renderer_encode_scene_path_us > 0
                || row.renderer_encode_scene_viewport_us > 0
                || row.renderer_encode_scene_flush_us > 0
                || row.renderer_encode_scene_text_shadow_us > 0
                || row.renderer_encode_scene_text_setup_us > 0
                || row.renderer_encode_scene_text_glyphs_us > 0
                || row.renderer_encode_scene_text_glyph_transform_us > 0
                || row.renderer_encode_scene_text_glyph_emit_us > 0
                || row.renderer_encode_scene_text_group_flush_us > 0
                || row.renderer_encode_scene_text_vertex_grow_events > 0
                || row.renderer_encode_scene_text_transform_fast_path_glyphs > 0
                || row.renderer_encode_scene_text_transform_generic_glyphs > 0
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
                if row.renderer_prepare_text_collect_pin_keys_us > 0
                    || row.renderer_prepare_text_bucket_delta_us > 0
                    || row.renderer_prepare_text_prewarm_us > 0
                    || row.renderer_prepare_text_pin_bucket_update_us > 0
                    || row.renderer_prepare_text_flush_uploads_us > 0
                    || row.renderer_prepare_text_scene_text_blobs > 0
                    || row.renderer_prepare_text_pinned_glyph_keys > 0
                {
                    line.push_str(&format!(
                        " renderer.text_prepare.us(collect/bucket_delta/prewarm/pin_update/flush)={}/{}/{}/{}/{}",
                        row.renderer_prepare_text_collect_pin_keys_us,
                        row.renderer_prepare_text_bucket_delta_us,
                        row.renderer_prepare_text_prewarm_us,
                        row.renderer_prepare_text_pin_bucket_update_us,
                        row.renderer_prepare_text_flush_uploads_us,
                    ));
                    line.push_str(&format!(
                        " renderer.text_prepare.counts(blobs/pinned/prewarm/retained/added/removed)={}/{}/{}/{}/{}/{}",
                        row.renderer_prepare_text_scene_text_blobs,
                        row.renderer_prepare_text_pinned_glyph_keys,
                        row.renderer_prepare_text_prewarm_glyph_keys,
                        row.renderer_prepare_text_retained_glyph_keys,
                        row.renderer_prepare_text_added_glyph_keys,
                        row.renderer_prepare_text_removed_glyph_keys,
                    ));
                }
                if row.renderer_uniform_bytes > 0
                    || row.renderer_instance_bytes > 0
                    || row.renderer_vertex_bytes > 0
                {
                    line.push_str(&format!(
                        " renderer.bytes(uniform/instance/vertex)={}/{}/{}",
                        row.renderer_uniform_bytes,
                        row.renderer_instance_bytes,
                        row.renderer_vertex_bytes,
                    ));
                }
                line.push_str(&format!(
                    " renderer.encode.us(stack/clip/mask/effect/quad/image/text/path/viewport/flush)={}/{}/{}/{}/{}/{}/{}/{}/{}/{}",
                    row.renderer_encode_scene_stack_us,
                    row.renderer_encode_scene_clip_us,
                    row.renderer_encode_scene_mask_us,
                    row.renderer_encode_scene_effect_us,
                    row.renderer_encode_scene_quad_us,
                    row.renderer_encode_scene_image_us,
                    row.renderer_encode_scene_text_us,
                    row.renderer_encode_scene_path_us,
                    row.renderer_encode_scene_viewport_us,
                    row.renderer_encode_scene_flush_us,
                ));
                line.push_str(&format!(
                    " renderer.encode.text(us/shadow/setup/glyphs)={}/{}/{}",
                    row.renderer_encode_scene_text_shadow_us,
                    row.renderer_encode_scene_text_setup_us,
                    row.renderer_encode_scene_text_glyphs_us,
                ));
                line.push_str(&format!(
                    " renderer.encode.text(us/transform/emit/flush)={}/{}/{}",
                    row.renderer_encode_scene_text_glyph_transform_us,
                    row.renderer_encode_scene_text_glyph_emit_us,
                    row.renderer_encode_scene_text_group_flush_us,
                ));
                line.push_str(&format!(
                    " renderer.encode.text(vertex_grow_events)={}",
                    row.renderer_encode_scene_text_vertex_grow_events,
                ));
                line.push_str(&format!(
                    " renderer.encode.text(transform_fast/generic)={}/{}",
                    row.renderer_encode_scene_text_transform_fast_path_glyphs,
                    row.renderer_encode_scene_text_transform_generic_glyphs,
                ));
                if row.renderer_encode_scene_stack_ops > 0
                    || row.renderer_encode_scene_clip_ops > 0
                    || row.renderer_encode_scene_mask_ops > 0
                    || row.renderer_encode_scene_effect_ops > 0
                    || row.renderer_encode_scene_quad_ops > 0
                    || row.renderer_encode_scene_image_ops > 0
                    || row.renderer_encode_scene_text_ops > 0
                    || row.renderer_encode_scene_path_ops > 0
                    || row.renderer_encode_scene_viewport_ops > 0
                    || row.renderer_encode_scene_flushes > 0
                {
                    line.push_str(&format!(
                        " renderer.encode.ops(stack/clip/mask/effect/quad/image/text/path/viewport/flush)={}/{}/{}/{}/{}/{}/{}/{}/{}/{}",
                        row.renderer_encode_scene_stack_ops,
                        row.renderer_encode_scene_clip_ops,
                        row.renderer_encode_scene_mask_ops,
                        row.renderer_encode_scene_effect_ops,
                        row.renderer_encode_scene_quad_ops,
                        row.renderer_encode_scene_image_ops,
                        row.renderer_encode_scene_text_ops,
                        row.renderer_encode_scene_path_ops,
                        row.renderer_encode_scene_viewport_ops,
                        row.renderer_encode_scene_flushes,
                    ));
                }
            }
            println!("{line}");
            Self::print_code_editor_paint_perf_row(row);
            Self::print_dispatch_breakdown_row(row, "dispatch_breakdown");
            if row.dispatch_post_dispatch_snapshot_time_us > 0
                || row.window_runtime_snapshot_focus_repair_time_us > 0
                || row.window_runtime_snapshot_input_context_time_us > 0
                || row.window_runtime_snapshot_command_availability_time_us > 0
                || row.window_runtime_snapshot_shortcut_overlay_time_us > 0
            {
                println!(
                    "    window_runtime_snapshot.us(dispatch_total/focus_repair/input_ctx/command_availability/shortcut_overlay)={}/{}/{}/{}/{}",
                    row.dispatch_post_dispatch_snapshot_time_us,
                    row.window_runtime_snapshot_focus_repair_time_us,
                    row.window_runtime_snapshot_input_context_time_us,
                    row.window_runtime_snapshot_command_availability_time_us,
                    row.window_runtime_snapshot_shortcut_overlay_time_us
                );
            }
            if row.window_runtime_snapshot_widget_command_count > 0
                || row.window_runtime_snapshot_command_registry_collect_time_us > 0
                || row.window_runtime_snapshot_command_availability_eval_time_us > 0
            {
                println!(
                    "    window_runtime_snapshot.command_availability(widget_count/collect_us/eval_us)={}/{}/{}",
                    row.window_runtime_snapshot_widget_command_count,
                    row.window_runtime_snapshot_command_registry_collect_time_us,
                    row.window_runtime_snapshot_command_availability_eval_time_us
                );
            }
            if !row.command_availability_hotspots.is_empty() {
                let items: Vec<String> = row
                    .command_availability_hotspots
                    .iter()
                    .map(|h| {
                        let start = h
                            .start_element_path
                            .as_deref()
                            .or(h.start_element_kind.as_deref())
                            .unwrap_or("unknown");
                        let resolved = h
                            .resolved_element_path
                            .as_deref()
                            .or(h.resolved_element_kind.as_deref())
                            .unwrap_or("none");
                        format!(
                            "{}@{}={}us outcome={} start_node={} resolved_node={} start_el={} resolved_el={} start={} resolved={}",
                            h.command,
                            h.route,
                            h.elapsed_us,
                            h.outcome,
                            h.start_node,
                            h.resolved_node
                                .map(|node| node.to_string())
                                .unwrap_or_else(|| "none".to_string()),
                            h.start_element
                                .map(|element| element.to_string())
                                .unwrap_or_else(|| "none".to_string()),
                            h.resolved_element
                                .map(|element| element.to_string())
                                .unwrap_or_else(|| "none".to_string()),
                            compact_string_middle(start, 40, 40),
                            compact_string_middle(resolved, 40, 40)
                        )
                    })
                    .collect();
                println!(
                    "    window_runtime_snapshot.command_availability.hotspots: {}",
                    items.join(" | ")
                );
            }
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
            println!(
                "dispatch attribution p50/p95/max (us): accounted={}/{}/{} unattributed={}/{}/{} body_unattributed={}/{}/{} runtime_wrapper={}/{}/{}",
                self.p50_dispatch_accounted_time_us,
                self.p95_dispatch_accounted_time_us,
                self.max_dispatch_accounted_time_us,
                self.p50_dispatch_unattributed_time_us,
                self.p95_dispatch_unattributed_time_us,
                self.max_dispatch_unattributed_time_us,
                self.p50_dispatch_inner_body_unattributed_time_us,
                self.p95_dispatch_inner_body_unattributed_time_us,
                self.max_dispatch_inner_body_unattributed_time_us,
                self.p50_dispatch_runtime_wrapper_time_us,
                self.p95_dispatch_runtime_wrapper_time_us,
                self.max_dispatch_runtime_wrapper_time_us
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
            if self.p50_renderer_prepare_text_collect_pin_keys_us > 0
                || self.p95_renderer_prepare_text_collect_pin_keys_us > 0
                || self.p50_renderer_prepare_text_bucket_delta_us > 0
                || self.p95_renderer_prepare_text_bucket_delta_us > 0
                || self.p50_renderer_prepare_text_prewarm_us > 0
                || self.p95_renderer_prepare_text_prewarm_us > 0
                || self.p50_renderer_prepare_text_pin_bucket_update_us > 0
                || self.p95_renderer_prepare_text_pin_bucket_update_us > 0
                || self.p50_renderer_prepare_text_flush_uploads_us > 0
                || self.p95_renderer_prepare_text_flush_uploads_us > 0
            {
                println!(
                    "renderer text_prepare p50/p95 (us): collect_pin_keys={}/{} bucket_delta={}/{} prewarm={}/{} pin_update={}/{} flush={}/{}",
                    self.p50_renderer_prepare_text_collect_pin_keys_us,
                    self.p95_renderer_prepare_text_collect_pin_keys_us,
                    self.p50_renderer_prepare_text_bucket_delta_us,
                    self.p95_renderer_prepare_text_bucket_delta_us,
                    self.p50_renderer_prepare_text_prewarm_us,
                    self.p95_renderer_prepare_text_prewarm_us,
                    self.p50_renderer_prepare_text_pin_bucket_update_us,
                    self.p95_renderer_prepare_text_pin_bucket_update_us,
                    self.p50_renderer_prepare_text_flush_uploads_us,
                    self.p95_renderer_prepare_text_flush_uploads_us,
                );
            }
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
            if self.max_renderer_prepare_text_collect_pin_keys_us > 0
                || self.max_renderer_prepare_text_bucket_delta_us > 0
                || self.max_renderer_prepare_text_prewarm_us > 0
                || self.max_renderer_prepare_text_pin_bucket_update_us > 0
                || self.max_renderer_prepare_text_flush_uploads_us > 0
            {
                println!(
                    "renderer text_prepare max (us): collect_pin_keys={} bucket_delta={} prewarm={} pin_update={} flush={}",
                    self.max_renderer_prepare_text_collect_pin_keys_us,
                    self.max_renderer_prepare_text_bucket_delta_us,
                    self.max_renderer_prepare_text_prewarm_us,
                    self.max_renderer_prepare_text_pin_bucket_update_us,
                    self.max_renderer_prepare_text_flush_uploads_us,
                );
            }
        }
        self.print_code_editor_paint_perf_summary();
        self.print_paint_widget_hotspot_summary();
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
                || row.renderer_encode_scene_stack_us > 0
                || row.renderer_encode_scene_clip_us > 0
                || row.renderer_encode_scene_mask_us > 0
                || row.renderer_encode_scene_effect_us > 0
                || row.renderer_encode_scene_quad_us > 0
                || row.renderer_encode_scene_image_us > 0
                || row.renderer_encode_scene_text_us > 0
                || row.renderer_encode_scene_path_us > 0
                || row.renderer_encode_scene_viewport_us > 0
                || row.renderer_encode_scene_flush_us > 0
                || row.renderer_encode_scene_text_shadow_us > 0
                || row.renderer_encode_scene_text_setup_us > 0
                || row.renderer_encode_scene_text_glyphs_us > 0
                || row.renderer_encode_scene_text_glyph_transform_us > 0
                || row.renderer_encode_scene_text_glyph_emit_us > 0
                || row.renderer_encode_scene_text_group_flush_us > 0
                || row.renderer_encode_scene_text_vertex_grow_events > 0
                || row.renderer_encode_scene_text_transform_fast_path_glyphs > 0
                || row.renderer_encode_scene_text_transform_generic_glyphs > 0
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
                if row.renderer_prepare_text_collect_pin_keys_us > 0
                    || row.renderer_prepare_text_bucket_delta_us > 0
                    || row.renderer_prepare_text_prewarm_us > 0
                    || row.renderer_prepare_text_pin_bucket_update_us > 0
                    || row.renderer_prepare_text_flush_uploads_us > 0
                    || row.renderer_prepare_text_scene_text_blobs > 0
                    || row.renderer_prepare_text_pinned_glyph_keys > 0
                {
                    line.push_str(&format!(
                        " renderer.text_prepare.us(collect/bucket_delta/prewarm/pin_update/flush)={}/{}/{}/{}/{}",
                        row.renderer_prepare_text_collect_pin_keys_us,
                        row.renderer_prepare_text_bucket_delta_us,
                        row.renderer_prepare_text_prewarm_us,
                        row.renderer_prepare_text_pin_bucket_update_us,
                        row.renderer_prepare_text_flush_uploads_us,
                    ));
                    line.push_str(&format!(
                        " renderer.text_prepare.counts(blobs/pinned/prewarm/retained/added/removed)={}/{}/{}/{}/{}/{}",
                        row.renderer_prepare_text_scene_text_blobs,
                        row.renderer_prepare_text_pinned_glyph_keys,
                        row.renderer_prepare_text_prewarm_glyph_keys,
                        row.renderer_prepare_text_retained_glyph_keys,
                        row.renderer_prepare_text_added_glyph_keys,
                        row.renderer_prepare_text_removed_glyph_keys,
                    ));
                }
                line.push_str(&format!(
                    " renderer.encode.us(stack/clip/mask/effect/quad/image/text/path/viewport/flush)={}/{}/{}/{}/{}/{}/{}/{}/{}/{}",
                    row.renderer_encode_scene_stack_us,
                    row.renderer_encode_scene_clip_us,
                    row.renderer_encode_scene_mask_us,
                    row.renderer_encode_scene_effect_us,
                    row.renderer_encode_scene_quad_us,
                    row.renderer_encode_scene_image_us,
                    row.renderer_encode_scene_text_us,
                    row.renderer_encode_scene_path_us,
                    row.renderer_encode_scene_viewport_us,
                    row.renderer_encode_scene_flush_us,
                ));
                line.push_str(&format!(
                    " renderer.encode.text(us/shadow/setup/glyphs)={}/{}/{}",
                    row.renderer_encode_scene_text_shadow_us,
                    row.renderer_encode_scene_text_setup_us,
                    row.renderer_encode_scene_text_glyphs_us,
                ));
                line.push_str(&format!(
                    " renderer.encode.text(us/transform/emit/flush)={}/{}/{}",
                    row.renderer_encode_scene_text_glyph_transform_us,
                    row.renderer_encode_scene_text_glyph_emit_us,
                    row.renderer_encode_scene_text_group_flush_us,
                ));
                line.push_str(&format!(
                    " renderer.encode.text(vertex_grow_events)={}",
                    row.renderer_encode_scene_text_vertex_grow_events,
                ));
                line.push_str(&format!(
                    " renderer.encode.text(transform_fast/generic)={}/{}",
                    row.renderer_encode_scene_text_transform_fast_path_glyphs,
                    row.renderer_encode_scene_text_transform_generic_glyphs,
                ));
                if row.renderer_encode_scene_stack_ops > 0
                    || row.renderer_encode_scene_clip_ops > 0
                    || row.renderer_encode_scene_mask_ops > 0
                    || row.renderer_encode_scene_effect_ops > 0
                    || row.renderer_encode_scene_quad_ops > 0
                    || row.renderer_encode_scene_image_ops > 0
                    || row.renderer_encode_scene_text_ops > 0
                    || row.renderer_encode_scene_path_ops > 0
                    || row.renderer_encode_scene_viewport_ops > 0
                    || row.renderer_encode_scene_flushes > 0
                {
                    line.push_str(&format!(
                        " renderer.encode.ops(stack/clip/mask/effect/quad/image/text/path/viewport/flush)={}/{}/{}/{}/{}/{}/{}/{}/{}/{}",
                        row.renderer_encode_scene_stack_ops,
                        row.renderer_encode_scene_clip_ops,
                        row.renderer_encode_scene_mask_ops,
                        row.renderer_encode_scene_effect_ops,
                        row.renderer_encode_scene_quad_ops,
                        row.renderer_encode_scene_image_ops,
                        row.renderer_encode_scene_text_ops,
                        row.renderer_encode_scene_path_ops,
                        row.renderer_encode_scene_viewport_ops,
                        row.renderer_encode_scene_flushes,
                    ));
                }
            }
            println!("{line}");
            Self::print_code_editor_paint_perf_row(row);
            Self::print_dispatch_breakdown_row(row, "dispatch_breakdown");
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
            if row.dispatch_post_dispatch_snapshot_time_us > 0
                || row.window_runtime_snapshot_focus_repair_time_us > 0
                || row.window_runtime_snapshot_input_context_time_us > 0
                || row.window_runtime_snapshot_command_availability_time_us > 0
                || row.window_runtime_snapshot_shortcut_overlay_time_us > 0
            {
                println!(
                    "    dispatch_snapshot.us(total/focus_repair/input_ctx/command_availability/shortcut_overlay)={}/{}/{}/{}/{}",
                    row.dispatch_post_dispatch_snapshot_time_us,
                    row.window_runtime_snapshot_focus_repair_time_us,
                    row.window_runtime_snapshot_input_context_time_us,
                    row.window_runtime_snapshot_command_availability_time_us,
                    row.window_runtime_snapshot_shortcut_overlay_time_us
                );
            }
            if row.window_runtime_snapshot_widget_command_count > 0
                || row.window_runtime_snapshot_command_registry_collect_time_us > 0
                || row.window_runtime_snapshot_command_availability_eval_time_us > 0
            {
                println!(
                    "    dispatch_snapshot.command_availability(widget_count/collect_us/eval_us)={}/{}/{}",
                    row.window_runtime_snapshot_widget_command_count,
                    row.window_runtime_snapshot_command_registry_collect_time_us,
                    row.window_runtime_snapshot_command_availability_eval_time_us
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
                || row.paint_host_widget_observed_deps_calls > 0
            {
                println!(
                    "    paint_host_widget.us(models/globals/instance)={}/{}/{} items={}/{} calls(instance/deps/empty/model_non_empty/global_non_empty)={}/{}/{}/{}/{}",
                    row.paint_host_widget_observed_models_time_us,
                    row.paint_host_widget_observed_globals_time_us,
                    row.paint_host_widget_instance_lookup_time_us,
                    row.paint_host_widget_observed_models_items,
                    row.paint_host_widget_observed_globals_items,
                    row.paint_host_widget_instance_lookup_calls,
                    row.paint_host_widget_observed_deps_calls,
                    row.paint_host_widget_observed_deps_empty_calls,
                    row.paint_host_widget_observed_models_non_empty_calls,
                    row.paint_host_widget_observed_globals_non_empty_calls,
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
                        if let Some(value) = c.layout_dependency.as_deref()
                            && !value.is_empty()
                        {
                            s.push_str(&format!(" layout_dependency={value}"));
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
                        push_cache_root_boundary_summary(&mut s, c);
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
                        if let Some(value) = c.layout_dependency.as_deref()
                            && !value.is_empty()
                        {
                            s.push_str(&format!(" layout_dependency={value}"));
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
                        push_cache_root_boundary_summary(&mut s, c);
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
            if !row.layout_request_build_roots.is_empty() {
                let items: Vec<String> = row
                    .layout_request_build_roots
                    .iter()
                    .take(3)
                    .map(|r| {
                        let mut out = format!("us={} root={}", r.elapsed_us, r.root_node);
                        if let Some(kind) = r.root_kind.as_deref()
                            && !kind.is_empty()
                        {
                            out.push_str(&format!(" kind={kind}"));
                        }
                        if let Some(mode) = r.mode.as_deref()
                            && !mode.is_empty()
                        {
                            out.push_str(&format!(" mode={mode}"));
                        }
                        out.push_str(&format!(
                            " engine={} invalidated={} subtree_dirty={} dirty_count={} descendant_dirty={} needs_layout={} translation_only={} marked_seen={}",
                            r.had_layout_engine_node,
                            r.layout_invalidated,
                            r.subtree_layout_dirty,
                            r.subtree_layout_dirty_count,
                            r.descendant_layout_dirty_count,
                            r.needs_layout,
                            r.is_translation_only,
                            r.nodes_marked_seen,
                        ));
                        if let Some(test_id) = r.root_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(role) = r.root_role.as_deref()
                            && !role.is_empty()
                        {
                            out.push_str(&format!(" role={role}"));
                        }
                        if let Some(element_kind) = r.root_element_kind.as_deref()
                            && !element_kind.is_empty()
                        {
                            out.push_str(&format!(" element_kind={element_kind}"));
                        }
                        if let Some(element) = r.root_element {
                            out.push_str(&format!(" element={element}"));
                        }
                        if let Some(path) = r.root_element_path.as_deref()
                            && !path.is_empty()
                        {
                            let path = compact_debug_path(path);
                            out.push_str(&format!(" path={path}"));
                        }
                        if !r.dirty_descendants.is_empty() {
                            let dirty = r
                                .dirty_descendants
                                .iter()
                                .take(2)
                                .map(|d| {
                                    let mut item = format!(
                                        "node={} dirty_count={}",
                                        d.node, d.subtree_layout_dirty_count
                                    );
                                    if let Some(source) = d.source.as_deref()
                                        && !source.is_empty()
                                    {
                                        item.push_str(&format!(" source={source}"));
                                    }
                                    if let Some(detail) = d.detail.as_deref()
                                        && !detail.is_empty()
                                    {
                                        item.push_str(&format!(" detail={detail}"));
                                    }
                                    if let Some(test_id) = d.test_id.as_deref()
                                        && !test_id.is_empty()
                                    {
                                        item.push_str(&format!(" test_id={test_id}"));
                                    }
                                    if let Some(kind) = d.element_kind.as_deref()
                                        && !kind.is_empty()
                                    {
                                        item.push_str(&format!(" element_kind={kind}"));
                                    }
                                    item
                                })
                                .collect::<Vec<_>>()
                                .join(",");
                            out.push_str(&format!(" dirty_desc=[{dirty}]"));
                        }
                        out
                    })
                    .collect();
                println!("    layout_request_build_roots: {}", items.join(" | "));
            }
            if !row.scroll_layout_profiles.is_empty() {
                let items: Vec<String> = row
                    .scroll_layout_profiles
                    .iter()
                    .take(3)
                    .map(|p| {
                        let mut out = format!(
                            "total_us={} layout_children_us={} child_max_us={} node={}",
                            p.total_us, p.layout_children_us, p.layout_child_max_us, p.node
                        );
                        if let Some(pass) = p.pass.as_deref()
                            && !pass.is_empty()
                        {
                            out.push_str(&format!(" pass={pass}"));
                        }
                        if let Some(axis) = p.axis.as_deref()
                            && !axis.is_empty()
                        {
                            out.push_str(&format!(" axis={axis}"));
                        }
                        out.push_str(&format!(
                            " resize={} direct_invalidated={} descendant_dirty={} child_dirty={} bounds_changed={:?} size_changed={:?} input_matches_before={:?} first_pass_us={} corrected_us={} corrected_relayout={}",
                            p.interactive_resize,
                            p.direct_children_layout_invalidated,
                            p.descendant_subtree_layout_dirty,
                            p.layout_child_max_subtree_dirty,
                            p.layout_child_max_bounds_changed,
                            p.layout_child_max_bounds_size_changed,
                            p.layout_child_max_input_matches_before,
                            p.layout_children_first_pass_us,
                            p.layout_children_corrected_content_us,
                            p.corrected_content_relayout,
                        ));
                        out.push_str(&format!(
                            " first_pass_nodes={}/{} corrected_nodes={}/{}",
                            p.layout_child_first_pass_nodes_visited,
                            p.layout_child_first_pass_nodes_performed,
                            p.layout_child_corrected_content_nodes_visited,
                            p.layout_child_corrected_content_nodes_performed,
                        ));
                        if !p.layout_child_kind_profiles.is_empty() {
                            let kinds = p
                                .layout_child_kind_profiles
                                .iter()
                                .take(4)
                                .map(|k| {
                                    format!(
                                        "{}:{}us/{}n",
                                        k.kind.as_deref().unwrap_or("<unknown>"),
                                        k.self_us,
                                        k.nodes
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(",");
                            out.push_str(&format!(" child_kind_self=[{kinds}]"));
                        }
                        if !p.phase_profiles.is_empty() {
                            let phases = p
                                .phase_profiles
                                .iter()
                                .take(6)
                                .map(|phase| {
                                    format!(
                                        "{}:{}us",
                                        phase.phase.as_deref().unwrap_or("<unknown>"),
                                        phase.us
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(",");
                            out.push_str(&format!(" phases=[{phases}]"));
                        }
                        if let Some(test_id) = p.test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        } else if let Some(test_id) = p.semantics_test_id.as_deref()
                            && !test_id.is_empty()
                        {
                            out.push_str(&format!(" test_id={test_id}"));
                        }
                        if let Some(path) = p.element_path.as_deref()
                            && !path.is_empty()
                        {
                            let path = compact_debug_path(path);
                            out.push_str(&format!(" path={path}"));
                        }
                        out
                    })
                    .collect();
                println!("    scroll_layout_profiles: {}", items.join(" | "));
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
                        if let Some(profile) = s.solve_profile.as_ref() {
                            if !profile.reason.is_empty() {
                                out.push_str(&format!(" reason={}", profile.reason));
                            }
                            out.push_str(&format!(" subtree_nodes={}", profile.subtree_nodes));
                            if profile.batch_roots > 1 {
                                out.push_str(&format!(" batch_roots={}", profile.batch_roots));
                            }
                            if profile.flex_wrap_patch_time_us > 0
                                || profile.flex_wrap_patch_probes > 0
                                || profile.flex_wrap_patch_mutations > 0
                            {
                                out.push_str(&format!(
                                    " flex_patch.us={} flex_patch.nodes={} flex_patch.wrap={} flex_patch.candidates={} flex_patch.probes={} flex_patch.mutations={}",
                                    profile.flex_wrap_patch_time_us,
                                    profile.flex_wrap_patch_visited_nodes,
                                    profile.flex_wrap_patch_wrap_nodes,
                                    profile.flex_wrap_patch_candidate_children,
                                    profile.flex_wrap_patch_probes,
                                    profile.flex_wrap_patch_mutations
                                ));
                            }
                            if let Some(w) = profile.available_w {
                                out.push_str(&format!(" avail.w={w:.1}"));
                            } else if !profile.available_w_kind.is_empty() {
                                out.push_str(&format!(" avail.w={}", profile.available_w_kind));
                            }
                            if let Some(h) = profile.available_h {
                                out.push_str(&format!(" avail.h={h:.1}"));
                            } else if !profile.available_h_kind.is_empty() {
                                out.push_str(&format!(" avail.h={}", profile.available_h_kind));
                            }
                            if let Some(dw) = profile.available_w_delta {
                                out.push_str(&format!(" delta.w={dw:.1}"));
                            } else if let Some(kind) = profile.previous_available_w_kind.as_deref()
                                && !kind.is_empty()
                            {
                                out.push_str(&format!(" prev.w={kind}"));
                            }
                            if let Some(dh) = profile.available_h_delta {
                                out.push_str(&format!(" delta.h={dh:.1}"));
                            } else if let Some(kind) = profile.previous_available_h_kind.as_deref()
                                && !kind.is_empty()
                            {
                                out.push_str(&format!(" prev.h={kind}"));
                            }
                            if let Some(frame_delta) = profile.previous_frame_delta {
                                out.push_str(&format!(" frame_delta={frame_delta}"));
                            }
                        }
                        if let Some(rejection) = s.clean_geometry_solve_skip_rejection.as_ref() {
                            if !rejection.reason.is_empty() {
                                out.push_str(&format!(" clean.reject={}", rejection.reason));
                            }
                            if let Some(detail) = rejection.detail.as_deref()
                                && !detail.is_empty()
                            {
                                out.push_str(&format!(" clean.detail={detail}"));
                            }
                            if let Some(node) = rejection.node {
                                out.push_str(&format!(" clean.node={node}"));
                            }
                            if let Some(kind) = rejection.element_kind.as_deref()
                                && !kind.is_empty()
                            {
                                out.push_str(&format!(" clean.kind={kind}"));
                            }
                            if let Some(test_id) = rejection.test_id.as_deref()
                                && !test_id.is_empty()
                            {
                                out.push_str(&format!(" clean.test_id={test_id}"));
                            }
                            if let Some(role) = rejection.role.as_deref()
                                && !role.is_empty()
                            {
                                out.push_str(&format!(" clean.role={role}"));
                            }
                            if let Some(path) = rejection.element_path.as_deref()
                                && !path.is_empty()
                            {
                                let path = compact_debug_path(path);
                                out.push_str(&format!(" clean.path={path}"));
                            }
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

        fn scroll_layout_kind_profile_to_json(
            p: &BundleStatsScrollLayoutKindProfile,
        ) -> Value {
            let mut obj = Map::new();
            obj.insert(
                "kind".to_string(),
                p.kind.clone().map(Value::from).unwrap_or(Value::Null),
            );
            obj.insert("nodes".to_string(), Value::from(p.nodes));
            obj.insert("self_us".to_string(), Value::from(p.self_us));
            obj.insert("total_us".to_string(), Value::from(p.total_us));
            obj.insert("max_self_us".to_string(), Value::from(p.max_self_us));
            obj.insert("max_total_us".to_string(), Value::from(p.max_total_us));
            Value::Object(obj)
        }

        fn scroll_layout_phase_profile_to_json(
            p: &BundleStatsScrollLayoutPhaseProfile,
        ) -> Value {
            let mut obj = Map::new();
            obj.insert(
                "phase".to_string(),
                p.phase.clone().map(Value::from).unwrap_or(Value::Null),
            );
            obj.insert("us".to_string(), Value::from(p.us));
            Value::Object(obj)
        }

        let mut root = Map::new();
        root.insert(
            "schema_version".to_string(),
            Value::from(crate::perf_schema::PERF_STATS_SCHEMA_VERSION),
        );
        root.insert(
            "kind".to_string(),
            Value::from(crate::perf_schema::PERF_STATS_KIND),
        );
        root.insert(
            "schema_policy".to_string(),
            crate::perf_schema::schema_policy_json(),
        );
        root.insert(
            "registered_perf_keys".to_string(),
            crate::perf_keys::registered_frame_stats_keys_json(),
        );
        root.insert(
            "source_bundle_schema_version".to_string(),
            Value::from(self.source_bundle_schema_version),
        );
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
        root.insert(
            "code_editor_paint_perf".to_string(),
            self.code_editor_paint_perf.to_json(),
        );
        root.insert(
            "paint_widget_hotspot_summary".to_string(),
            self.paint_widget_hotspot_summary
                .to_json(&self.code_editor_paint_perf),
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
            "paint_record_visual_bounds_time_us".to_string(),
            Value::from(self.max_paint_record_visual_bounds_time_us),
        );
        max.insert(
            "paint_record_visual_bounds_calls".to_string(),
            Value::from(self.max_paint_record_visual_bounds_calls),
        );
        max.insert(
            "paint_cache_key_time_us".to_string(),
            Value::from(self.max_paint_cache_key_time_us),
        );
        max.insert(
            "paint_cache_hit_check_time_us".to_string(),
            Value::from(self.max_paint_cache_hit_check_time_us),
        );
        max.insert(
            "paint_observation_record_time_us".to_string(),
            Value::from(self.max_paint_observation_record_time_us),
        );
        max.insert(
            "paint_host_widget_observed_models_time_us".to_string(),
            Value::from(self.max_paint_host_widget_observed_models_time_us),
        );
        max.insert(
            "paint_host_widget_observed_models_items".to_string(),
            Value::from(self.max_paint_host_widget_observed_models_items),
        );
        max.insert(
            "paint_host_widget_observed_globals_time_us".to_string(),
            Value::from(self.max_paint_host_widget_observed_globals_time_us),
        );
        max.insert(
            "paint_host_widget_observed_globals_items".to_string(),
            Value::from(self.max_paint_host_widget_observed_globals_items),
        );
        max.insert(
            "paint_host_widget_observed_deps_calls".to_string(),
            Value::from(self.max_paint_host_widget_observed_deps_calls),
        );
        max.insert(
            "paint_host_widget_observed_deps_empty_calls".to_string(),
            Value::from(self.max_paint_host_widget_observed_deps_empty_calls),
        );
        max.insert(
            "paint_host_widget_observed_models_non_empty_calls".to_string(),
            Value::from(self.max_paint_host_widget_observed_models_non_empty_calls),
        );
        max.insert(
            "paint_host_widget_observed_globals_non_empty_calls".to_string(),
            Value::from(self.max_paint_host_widget_observed_globals_non_empty_calls),
        );
        max.insert(
            "paint_host_widget_instance_lookup_time_us".to_string(),
            Value::from(self.max_paint_host_widget_instance_lookup_time_us),
        );
        max.insert(
            "paint_host_widget_instance_lookup_calls".to_string(),
            Value::from(self.max_paint_host_widget_instance_lookup_calls),
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
            "dispatch_accounted_time_us".to_string(),
            Value::from(self.max_dispatch_accounted_time_us),
        );
        max.insert(
            "dispatch_unattributed_time_us".to_string(),
            Value::from(self.max_dispatch_unattributed_time_us),
        );
        max.insert(
            "dispatch_inner_body_unattributed_time_us".to_string(),
            Value::from(self.max_dispatch_inner_body_unattributed_time_us),
        );
        max.insert(
            "dispatch_runtime_wrapper_time_us".to_string(),
            Value::from(self.max_dispatch_runtime_wrapper_time_us),
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
            "renderer_prepare_text_collect_pin_keys_us".to_string(),
            Value::from(self.max_renderer_prepare_text_collect_pin_keys_us),
        );
        max.insert(
            "renderer_prepare_text_bucket_delta_us".to_string(),
            Value::from(self.max_renderer_prepare_text_bucket_delta_us),
        );
        max.insert(
            "renderer_prepare_text_prewarm_us".to_string(),
            Value::from(self.max_renderer_prepare_text_prewarm_us),
        );
        max.insert(
            "renderer_prepare_text_pin_bucket_update_us".to_string(),
            Value::from(self.max_renderer_prepare_text_pin_bucket_update_us),
        );
        max.insert(
            "renderer_prepare_text_flush_uploads_us".to_string(),
            Value::from(self.max_renderer_prepare_text_flush_uploads_us),
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
            "paint_record_visual_bounds_time_us".to_string(),
            Value::from(self.p50_paint_record_visual_bounds_time_us),
        );
        p50.insert(
            "paint_record_visual_bounds_calls".to_string(),
            Value::from(self.p50_paint_record_visual_bounds_calls),
        );
        p50.insert(
            "paint_cache_key_time_us".to_string(),
            Value::from(self.p50_paint_cache_key_time_us),
        );
        p50.insert(
            "paint_cache_hit_check_time_us".to_string(),
            Value::from(self.p50_paint_cache_hit_check_time_us),
        );
        p50.insert(
            "paint_observation_record_time_us".to_string(),
            Value::from(self.p50_paint_observation_record_time_us),
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
            "dispatch_accounted_time_us".to_string(),
            Value::from(self.p50_dispatch_accounted_time_us),
        );
        p50.insert(
            "dispatch_unattributed_time_us".to_string(),
            Value::from(self.p50_dispatch_unattributed_time_us),
        );
        p50.insert(
            "dispatch_inner_body_unattributed_time_us".to_string(),
            Value::from(self.p50_dispatch_inner_body_unattributed_time_us),
        );
        p50.insert(
            "dispatch_runtime_wrapper_time_us".to_string(),
            Value::from(self.p50_dispatch_runtime_wrapper_time_us),
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
            "paint_host_widget_observed_models_time_us".to_string(),
            Value::from(self.p50_paint_host_widget_observed_models_time_us),
        );
        p50.insert(
            "paint_host_widget_observed_models_items".to_string(),
            Value::from(self.p50_paint_host_widget_observed_models_items),
        );
        p50.insert(
            "paint_host_widget_observed_globals_time_us".to_string(),
            Value::from(self.p50_paint_host_widget_observed_globals_time_us),
        );
        p50.insert(
            "paint_host_widget_observed_globals_items".to_string(),
            Value::from(self.p50_paint_host_widget_observed_globals_items),
        );
        p50.insert(
            "paint_host_widget_observed_deps_calls".to_string(),
            Value::from(self.p50_paint_host_widget_observed_deps_calls),
        );
        p50.insert(
            "paint_host_widget_observed_deps_empty_calls".to_string(),
            Value::from(self.p50_paint_host_widget_observed_deps_empty_calls),
        );
        p50.insert(
            "paint_host_widget_observed_models_non_empty_calls".to_string(),
            Value::from(self.p50_paint_host_widget_observed_models_non_empty_calls),
        );
        p50.insert(
            "paint_host_widget_observed_globals_non_empty_calls".to_string(),
            Value::from(self.p50_paint_host_widget_observed_globals_non_empty_calls),
        );
        p50.insert(
            "paint_host_widget_instance_lookup_time_us".to_string(),
            Value::from(self.p50_paint_host_widget_instance_lookup_time_us),
        );
        p50.insert(
            "paint_host_widget_instance_lookup_calls".to_string(),
            Value::from(self.p50_paint_host_widget_instance_lookup_calls),
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
        p50.insert(
            "renderer_prepare_text_collect_pin_keys_us".to_string(),
            Value::from(self.p50_renderer_prepare_text_collect_pin_keys_us),
        );
        p50.insert(
            "renderer_prepare_text_bucket_delta_us".to_string(),
            Value::from(self.p50_renderer_prepare_text_bucket_delta_us),
        );
        p50.insert(
            "renderer_prepare_text_prewarm_us".to_string(),
            Value::from(self.p50_renderer_prepare_text_prewarm_us),
        );
        p50.insert(
            "renderer_prepare_text_pin_bucket_update_us".to_string(),
            Value::from(self.p50_renderer_prepare_text_pin_bucket_update_us),
        );
        p50.insert(
            "renderer_prepare_text_flush_uploads_us".to_string(),
            Value::from(self.p50_renderer_prepare_text_flush_uploads_us),
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
            "paint_record_visual_bounds_time_us".to_string(),
            Value::from(self.p95_paint_record_visual_bounds_time_us),
        );
        p95.insert(
            "paint_record_visual_bounds_calls".to_string(),
            Value::from(self.p95_paint_record_visual_bounds_calls),
        );
        p95.insert(
            "paint_cache_key_time_us".to_string(),
            Value::from(self.p95_paint_cache_key_time_us),
        );
        p95.insert(
            "paint_cache_hit_check_time_us".to_string(),
            Value::from(self.p95_paint_cache_hit_check_time_us),
        );
        p95.insert(
            "paint_observation_record_time_us".to_string(),
            Value::from(self.p95_paint_observation_record_time_us),
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
            "dispatch_accounted_time_us".to_string(),
            Value::from(self.p95_dispatch_accounted_time_us),
        );
        p95.insert(
            "dispatch_unattributed_time_us".to_string(),
            Value::from(self.p95_dispatch_unattributed_time_us),
        );
        p95.insert(
            "dispatch_inner_body_unattributed_time_us".to_string(),
            Value::from(self.p95_dispatch_inner_body_unattributed_time_us),
        );
        p95.insert(
            "dispatch_runtime_wrapper_time_us".to_string(),
            Value::from(self.p95_dispatch_runtime_wrapper_time_us),
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
            "paint_host_widget_observed_models_time_us".to_string(),
            Value::from(self.p95_paint_host_widget_observed_models_time_us),
        );
        p95.insert(
            "paint_host_widget_observed_models_items".to_string(),
            Value::from(self.p95_paint_host_widget_observed_models_items),
        );
        p95.insert(
            "paint_host_widget_observed_globals_time_us".to_string(),
            Value::from(self.p95_paint_host_widget_observed_globals_time_us),
        );
        p95.insert(
            "paint_host_widget_observed_globals_items".to_string(),
            Value::from(self.p95_paint_host_widget_observed_globals_items),
        );
        p95.insert(
            "paint_host_widget_observed_deps_calls".to_string(),
            Value::from(self.p95_paint_host_widget_observed_deps_calls),
        );
        p95.insert(
            "paint_host_widget_observed_deps_empty_calls".to_string(),
            Value::from(self.p95_paint_host_widget_observed_deps_empty_calls),
        );
        p95.insert(
            "paint_host_widget_observed_models_non_empty_calls".to_string(),
            Value::from(self.p95_paint_host_widget_observed_models_non_empty_calls),
        );
        p95.insert(
            "paint_host_widget_observed_globals_non_empty_calls".to_string(),
            Value::from(self.p95_paint_host_widget_observed_globals_non_empty_calls),
        );
        p95.insert(
            "paint_host_widget_instance_lookup_time_us".to_string(),
            Value::from(self.p95_paint_host_widget_instance_lookup_time_us),
        );
        p95.insert(
            "paint_host_widget_instance_lookup_calls".to_string(),
            Value::from(self.p95_paint_host_widget_instance_lookup_calls),
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
        p95.insert(
            "renderer_prepare_text_collect_pin_keys_us".to_string(),
            Value::from(self.p95_renderer_prepare_text_collect_pin_keys_us),
        );
        p95.insert(
            "renderer_prepare_text_bucket_delta_us".to_string(),
            Value::from(self.p95_renderer_prepare_text_bucket_delta_us),
        );
        p95.insert(
            "renderer_prepare_text_prewarm_us".to_string(),
            Value::from(self.p95_renderer_prepare_text_prewarm_us),
        );
        p95.insert(
            "renderer_prepare_text_pin_bucket_update_us".to_string(),
            Value::from(self.p95_renderer_prepare_text_pin_bucket_update_us),
        );
        p95.insert(
            "renderer_prepare_text_flush_uploads_us".to_string(),
            Value::from(self.p95_renderer_prepare_text_flush_uploads_us),
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
                    "renderer_prepare_text_collect_pin_keys_us".to_string(),
                    Value::from(row.renderer_prepare_text_collect_pin_keys_us),
                );
                obj.insert(
                    "renderer_prepare_text_bucket_delta_us".to_string(),
                    Value::from(row.renderer_prepare_text_bucket_delta_us),
                );
                obj.insert(
                    "renderer_prepare_text_prewarm_us".to_string(),
                    Value::from(row.renderer_prepare_text_prewarm_us),
                );
                obj.insert(
                    "renderer_prepare_text_pin_bucket_update_us".to_string(),
                    Value::from(row.renderer_prepare_text_pin_bucket_update_us),
                );
                obj.insert(
                    "renderer_prepare_text_flush_uploads_us".to_string(),
                    Value::from(row.renderer_prepare_text_flush_uploads_us),
                );
                obj.insert(
                    "renderer_prepare_text_scene_text_blobs".to_string(),
                    Value::from(row.renderer_prepare_text_scene_text_blobs),
                );
                obj.insert(
                    "renderer_prepare_text_pinned_glyph_keys".to_string(),
                    Value::from(row.renderer_prepare_text_pinned_glyph_keys),
                );
                obj.insert(
                    "renderer_prepare_text_prewarm_glyph_keys".to_string(),
                    Value::from(row.renderer_prepare_text_prewarm_glyph_keys),
                );
                obj.insert(
                    "renderer_prepare_text_retained_glyph_keys".to_string(),
                    Value::from(row.renderer_prepare_text_retained_glyph_keys),
                );
                obj.insert(
                    "renderer_prepare_text_added_glyph_keys".to_string(),
                    Value::from(row.renderer_prepare_text_added_glyph_keys),
                );
                obj.insert(
                    "renderer_prepare_text_removed_glyph_keys".to_string(),
                    Value::from(row.renderer_prepare_text_removed_glyph_keys),
                );
                obj.insert(
                    "renderer_uniform_bytes".to_string(),
                    Value::from(row.renderer_uniform_bytes),
                );
                obj.insert(
                    "renderer_instance_bytes".to_string(),
                    Value::from(row.renderer_instance_bytes),
                );
                obj.insert(
                    "renderer_vertex_bytes".to_string(),
                    Value::from(row.renderer_vertex_bytes),
                );
                obj.insert(
                    "renderer_encode_scene_stack_us".to_string(),
                    Value::from(row.renderer_encode_scene_stack_us),
                );
                obj.insert(
                    "renderer_encode_scene_clip_us".to_string(),
                    Value::from(row.renderer_encode_scene_clip_us),
                );
                obj.insert(
                    "renderer_encode_scene_mask_us".to_string(),
                    Value::from(row.renderer_encode_scene_mask_us),
                );
                obj.insert(
                    "renderer_encode_scene_effect_us".to_string(),
                    Value::from(row.renderer_encode_scene_effect_us),
                );
                obj.insert(
                    "renderer_encode_scene_quad_us".to_string(),
                    Value::from(row.renderer_encode_scene_quad_us),
                );
                obj.insert(
                    "renderer_encode_scene_image_us".to_string(),
                    Value::from(row.renderer_encode_scene_image_us),
                );
                obj.insert(
                    "renderer_encode_scene_text_us".to_string(),
                    Value::from(row.renderer_encode_scene_text_us),
                );
                obj.insert(
                    "renderer_encode_scene_path_us".to_string(),
                    Value::from(row.renderer_encode_scene_path_us),
                );
                obj.insert(
                    "renderer_encode_scene_viewport_us".to_string(),
                    Value::from(row.renderer_encode_scene_viewport_us),
                );
                obj.insert(
                    "renderer_encode_scene_flush_us".to_string(),
                    Value::from(row.renderer_encode_scene_flush_us),
                );
                obj.insert(
                    "renderer_encode_scene_text_shadow_us".to_string(),
                    Value::from(row.renderer_encode_scene_text_shadow_us),
                );
                obj.insert(
                    "renderer_encode_scene_text_setup_us".to_string(),
                    Value::from(row.renderer_encode_scene_text_setup_us),
                );
                obj.insert(
                    "renderer_encode_scene_text_glyphs_us".to_string(),
                    Value::from(row.renderer_encode_scene_text_glyphs_us),
                );
                obj.insert(
                    "renderer_encode_scene_text_glyph_transform_us".to_string(),
                    Value::from(row.renderer_encode_scene_text_glyph_transform_us),
                );
                obj.insert(
                    "renderer_encode_scene_text_glyph_emit_us".to_string(),
                    Value::from(row.renderer_encode_scene_text_glyph_emit_us),
                );
                obj.insert(
                    "renderer_encode_scene_text_group_flush_us".to_string(),
                    Value::from(row.renderer_encode_scene_text_group_flush_us),
                );
                obj.insert(
                    "renderer_encode_scene_text_vertex_grow_events".to_string(),
                    Value::from(row.renderer_encode_scene_text_vertex_grow_events),
                );
                obj.insert(
                    "renderer_encode_scene_text_transform_fast_path_glyphs".to_string(),
                    Value::from(row.renderer_encode_scene_text_transform_fast_path_glyphs),
                );
                obj.insert(
                    "renderer_encode_scene_text_transform_generic_glyphs".to_string(),
                    Value::from(row.renderer_encode_scene_text_transform_generic_glyphs),
                );
                obj.insert(
                    "renderer_encode_scene_stack_ops".to_string(),
                    Value::from(row.renderer_encode_scene_stack_ops),
                );
                obj.insert(
                    "renderer_encode_scene_clip_ops".to_string(),
                    Value::from(row.renderer_encode_scene_clip_ops),
                );
                obj.insert(
                    "renderer_encode_scene_mask_ops".to_string(),
                    Value::from(row.renderer_encode_scene_mask_ops),
                );
                obj.insert(
                    "renderer_encode_scene_effect_ops".to_string(),
                    Value::from(row.renderer_encode_scene_effect_ops),
                );
                obj.insert(
                    "renderer_encode_scene_quad_ops".to_string(),
                    Value::from(row.renderer_encode_scene_quad_ops),
                );
                obj.insert(
                    "renderer_encode_scene_image_ops".to_string(),
                    Value::from(row.renderer_encode_scene_image_ops),
                );
                obj.insert(
                    "renderer_encode_scene_text_ops".to_string(),
                    Value::from(row.renderer_encode_scene_text_ops),
                );
                obj.insert(
                    "renderer_encode_scene_path_ops".to_string(),
                    Value::from(row.renderer_encode_scene_path_ops),
                );
                obj.insert(
                    "renderer_encode_scene_viewport_ops".to_string(),
                    Value::from(row.renderer_encode_scene_viewport_ops),
                );
                obj.insert(
                    "renderer_encode_scene_flushes".to_string(),
                    Value::from(row.renderer_encode_scene_flushes),
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
                    "dispatch_inner_body_time_us".to_string(),
                    Value::from(row.dispatch_inner_body_time_us),
                );
                let dispatch_accounted_time_us = Self::dispatch_accounted_time_us(row);
                obj.insert(
                    "dispatch_accounted_time_us".to_string(),
                    Value::from(dispatch_accounted_time_us),
                );
                obj.insert(
                    "dispatch_unattributed_time_us".to_string(),
                    Value::from(row.dispatch_time_us.saturating_sub(dispatch_accounted_time_us)),
                );
                obj.insert(
                    "dispatch_inner_body_unattributed_time_us".to_string(),
                    Value::from(Self::dispatch_inner_body_unattributed_time_us(row)),
                );
                obj.insert(
                    "dispatch_runtime_wrapper_time_us".to_string(),
                    Value::from(Self::dispatch_runtime_wrapper_time_us(row)),
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
                    "dispatch_input_state_update_time_us".to_string(),
                    Value::from(row.dispatch_input_state_update_time_us),
                );
                obj.insert(
                    "dispatch_context_build_time_us".to_string(),
                    Value::from(row.dispatch_context_build_time_us),
                );
                obj.insert(
                    "dispatch_prelude_time_us".to_string(),
                    Value::from(row.dispatch_prelude_time_us),
                );
                obj.insert(
                    "dispatch_pointer_arbitration_time_us".to_string(),
                    Value::from(row.dispatch_pointer_arbitration_time_us),
                );
                obj.insert(
                    "dispatch_pointer_target_routing_time_us".to_string(),
                    Value::from(row.dispatch_pointer_target_routing_time_us),
                );
                obj.insert(
                    "dispatch_post_widget_control_flow_time_us".to_string(),
                    Value::from(row.dispatch_post_widget_control_flow_time_us),
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
                    "window_runtime_snapshot_focus_repair_time_us".to_string(),
                    Value::from(row.window_runtime_snapshot_focus_repair_time_us),
                );
                obj.insert(
                    "window_runtime_snapshot_input_context_time_us".to_string(),
                    Value::from(row.window_runtime_snapshot_input_context_time_us),
                );
                obj.insert(
                    "window_runtime_snapshot_command_availability_time_us".to_string(),
                    Value::from(row.window_runtime_snapshot_command_availability_time_us),
                );
                obj.insert(
                    "window_runtime_snapshot_widget_command_count".to_string(),
                    Value::from(row.window_runtime_snapshot_widget_command_count),
                );
                obj.insert(
                    "window_runtime_snapshot_command_registry_collect_time_us".to_string(),
                    Value::from(row.window_runtime_snapshot_command_registry_collect_time_us),
                );
                obj.insert(
                    "window_runtime_snapshot_command_availability_eval_time_us".to_string(),
                    Value::from(row.window_runtime_snapshot_command_availability_eval_time_us),
                );
                obj.insert(
                    "window_runtime_snapshot_shortcut_overlay_time_us".to_string(),
                    Value::from(row.window_runtime_snapshot_shortcut_overlay_time_us),
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
                    "layout_engine_child_rect_queries".to_string(),
                    Value::from(row.layout_engine_child_rect_queries),
                );
                obj.insert(
                    "layout_engine_child_rect_time_us".to_string(),
                    Value::from(row.layout_engine_child_rect_time_us),
                );
                obj.insert(
                    "layout_engine_widget_fallback_solves".to_string(),
                    Value::from(row.layout_engine_widget_fallback_solves),
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
                    "paint_host_widget_observed_deps_calls".to_string(),
                    Value::from(row.paint_host_widget_observed_deps_calls),
                );
                obj.insert(
                    "paint_host_widget_observed_deps_empty_calls".to_string(),
                    Value::from(row.paint_host_widget_observed_deps_empty_calls),
                );
                obj.insert(
                    "paint_host_widget_observed_models_non_empty_calls".to_string(),
                    Value::from(row.paint_host_widget_observed_models_non_empty_calls),
                );
                obj.insert(
                    "paint_host_widget_observed_globals_non_empty_calls".to_string(),
                    Value::from(row.paint_host_widget_observed_globals_non_empty_calls),
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
                    "code_editor_paint_perf".to_string(),
                    row.code_editor_paint_perf
                        .as_ref()
                        .map(BundleStatsCodeEditorPaintPerf::to_json)
                        .unwrap_or(Value::Null),
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
                            "layout_dependency".to_string(),
                            c.layout_dependency
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
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
                        insert_cache_root_boundary_json(&mut c_obj, c);
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
                            "layout_dependency".to_string(),
                            c.layout_dependency
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
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
                        insert_cache_root_boundary_json(&mut c_obj, c);
                        Value::Object(c_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "top_contained_relayout_cache_roots".to_string(),
                    Value::Array(top_contained_relayout_cache_roots),
                );

                let layout_request_build_roots = row
                    .layout_request_build_roots
                    .iter()
                    .map(|r| {
                        let mut r_obj = Map::new();
                        r_obj.insert("root_node".to_string(), Value::from(r.root_node));
                        r_obj.insert(
                            "root_kind".to_string(),
                            r.root_kind.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        r_obj.insert(
                            "root_element".to_string(),
                            r.root_element.map(Value::from).unwrap_or(Value::Null),
                        );
                        r_obj.insert(
                            "root_element_kind".to_string(),
                            r.root_element_kind
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        r_obj.insert(
                            "root_element_path".to_string(),
                            r.root_element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        r_obj.insert("elapsed_us".to_string(), Value::from(r.elapsed_us));
                        r_obj.insert(
                            "mode".to_string(),
                            r.mode.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        r_obj.insert(
                            "had_layout_engine_node".to_string(),
                            Value::from(r.had_layout_engine_node),
                        );
                        r_obj.insert(
                            "layout_invalidated".to_string(),
                            Value::from(r.layout_invalidated),
                        );
                        r_obj.insert(
                            "subtree_layout_dirty".to_string(),
                            Value::from(r.subtree_layout_dirty),
                        );
                        r_obj.insert(
                            "subtree_layout_dirty_count".to_string(),
                            Value::from(r.subtree_layout_dirty_count),
                        );
                        r_obj.insert(
                            "descendant_layout_dirty_count".to_string(),
                            Value::from(r.descendant_layout_dirty_count),
                        );
                        r_obj.insert("needs_layout".to_string(), Value::from(r.needs_layout));
                        r_obj.insert(
                            "is_translation_only".to_string(),
                            Value::from(r.is_translation_only),
                        );
                        r_obj.insert(
                            "nodes_marked_seen".to_string(),
                            Value::from(r.nodes_marked_seen),
                        );
                        r_obj.insert(
                            "root_role".to_string(),
                            r.root_role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        r_obj.insert(
                            "root_test_id".to_string(),
                            r.root_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        let dirty_descendants = r
                            .dirty_descendants
                            .iter()
                            .map(|d| {
                                let mut d_obj = Map::new();
                                d_obj.insert("node".to_string(), Value::from(d.node));
                                d_obj.insert(
                                    "element".to_string(),
                                    d.element.map(Value::from).unwrap_or(Value::Null),
                                );
                                d_obj.insert(
                                    "element_kind".to_string(),
                                    d.element_kind
                                        .clone()
                                        .map(Value::from)
                                        .unwrap_or(Value::Null),
                                );
                                d_obj.insert(
                                    "element_path".to_string(),
                                    d.element_path
                                        .clone()
                                        .map(Value::from)
                                        .unwrap_or(Value::Null),
                                );
                                d_obj.insert(
                                    "subtree_layout_dirty_count".to_string(),
                                    Value::from(d.subtree_layout_dirty_count),
                                );
                                d_obj.insert(
                                    "source_root_node".to_string(),
                                    d.source_root_node.map(Value::from).unwrap_or(Value::Null),
                                );
                                d_obj.insert(
                                    "source".to_string(),
                                    d.source.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                d_obj.insert(
                                    "detail".to_string(),
                                    d.detail.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                d_obj.insert(
                                    "role".to_string(),
                                    d.role.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                d_obj.insert(
                                    "test_id".to_string(),
                                    d.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                                );
                                Value::Object(d_obj)
                            })
                            .collect::<Vec<_>>();
                        r_obj.insert(
                            "dirty_descendants".to_string(),
                            Value::Array(dirty_descendants),
                        );
                        Value::Object(r_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "layout_request_build_roots".to_string(),
                    Value::Array(layout_request_build_roots),
                );

                let scroll_layout_profiles = row
                    .scroll_layout_profiles
                    .iter()
                    .map(|p| {
                        let mut p_obj = Map::new();
                        p_obj.insert("node".to_string(), Value::from(p.node));
                        p_obj.insert(
                            "element".to_string(),
                            p.element.map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "test_id".to_string(),
                            p.test_id.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "semantics_test_id".to_string(),
                            p.semantics_test_id
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "role".to_string(),
                            p.role.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "axis".to_string(),
                            p.axis.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "pass".to_string(),
                            p.pass.clone().map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert("probe_unbounded".to_string(), Value::from(p.probe_unbounded));
                        p_obj.insert("children".to_string(), Value::from(p.children));
                        p_obj.insert(
                            "available_w".to_string(),
                            p.available_w.map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "available_h".to_string(),
                            p.available_h.map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "desired_w".to_string(),
                            p.desired_w.map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "desired_h".to_string(),
                            p.desired_h.map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "content_w".to_string(),
                            p.content_w.map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "content_h".to_string(),
                            p.content_h.map(Value::from).unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "post_layout_extents_mode".to_string(),
                            Value::from(p.post_layout_extents_mode),
                        );
                        p_obj.insert(
                            "interactive_resize".to_string(),
                            Value::from(p.interactive_resize),
                        );
                        p_obj.insert(
                            "direct_children_layout_invalidated".to_string(),
                            Value::from(p.direct_children_layout_invalidated),
                        );
                        p_obj.insert(
                            "descendant_subtree_layout_dirty".to_string(),
                            Value::from(p.descendant_subtree_layout_dirty),
                        );
                        p_obj.insert(
                            "force_barrier_child_root_relayout".to_string(),
                            Value::from(p.force_barrier_child_root_relayout),
                        );
                        p_obj.insert(
                            "phase_profiles".to_string(),
                            Value::Array(
                                p.phase_profiles
                                    .iter()
                                    .map(scroll_layout_phase_profile_to_json)
                                    .collect(),
                            ),
                        );
                        p_obj.insert(
                            "measure_children_us".to_string(),
                            Value::from(p.measure_children_us),
                        );
                        p_obj.insert("solve_barrier_us".to_string(), Value::from(p.solve_barrier_us));
                        p_obj.insert(
                            "layout_children_us".to_string(),
                            Value::from(p.layout_children_us),
                        );
                        p_obj.insert(
                            "layout_children_first_pass_us".to_string(),
                            Value::from(p.layout_children_first_pass_us),
                        );
                        p_obj.insert(
                            "layout_child_first_pass_nodes_visited".to_string(),
                            Value::from(p.layout_child_first_pass_nodes_visited),
                        );
                        p_obj.insert(
                            "layout_child_first_pass_nodes_performed".to_string(),
                            Value::from(p.layout_child_first_pass_nodes_performed),
                        );
                        p_obj.insert(
                            "layout_child_first_pass_max_us".to_string(),
                            Value::from(p.layout_child_first_pass_max_us),
                        );
                        p_obj.insert(
                            "layout_child_first_pass_kind_profiles".to_string(),
                            Value::Array(
                                p.layout_child_first_pass_kind_profiles
                                    .iter()
                                    .map(scroll_layout_kind_profile_to_json)
                                    .collect(),
                            ),
                        );
                        p_obj.insert(
                            "corrected_content_relayout".to_string(),
                            Value::from(p.corrected_content_relayout),
                        );
                        p_obj.insert(
                            "layout_children_corrected_content_us".to_string(),
                            Value::from(p.layout_children_corrected_content_us),
                        );
                        p_obj.insert(
                            "layout_child_corrected_content_nodes_visited".to_string(),
                            Value::from(p.layout_child_corrected_content_nodes_visited),
                        );
                        p_obj.insert(
                            "layout_child_corrected_content_nodes_performed".to_string(),
                            Value::from(p.layout_child_corrected_content_nodes_performed),
                        );
                        p_obj.insert(
                            "layout_child_corrected_content_max_us".to_string(),
                            Value::from(p.layout_child_corrected_content_max_us),
                        );
                        p_obj.insert(
                            "layout_child_corrected_content_kind_profiles".to_string(),
                            Value::Array(
                                p.layout_child_corrected_content_kind_profiles
                                    .iter()
                                    .map(scroll_layout_kind_profile_to_json)
                                    .collect(),
                            ),
                        );
                        p_obj.insert(
                            "layout_child_nodes_visited".to_string(),
                            Value::from(p.layout_child_nodes_visited),
                        );
                        p_obj.insert(
                            "layout_child_nodes_performed".to_string(),
                            Value::from(p.layout_child_nodes_performed),
                        );
                        p_obj.insert(
                            "layout_child_kind_profiles".to_string(),
                            Value::Array(
                                p.layout_child_kind_profiles
                                    .iter()
                                    .map(scroll_layout_kind_profile_to_json)
                                    .collect(),
                            ),
                        );
                        p_obj.insert(
                            "layout_child_max_us".to_string(),
                            Value::from(p.layout_child_max_us),
                        );
                        p_obj.insert(
                            "layout_child_max_node".to_string(),
                            p.layout_child_max_node
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "layout_child_max_invalidated".to_string(),
                            Value::from(p.layout_child_max_invalidated),
                        );
                        p_obj.insert(
                            "layout_child_max_subtree_dirty".to_string(),
                            Value::from(p.layout_child_max_subtree_dirty),
                        );
                        p_obj.insert(
                            "layout_child_max_subtree_dirty_count".to_string(),
                            Value::from(p.layout_child_max_subtree_dirty_count),
                        );
                        p_obj.insert(
                            "layout_child_max_nodes_visited".to_string(),
                            Value::from(p.layout_child_max_nodes_visited),
                        );
                        p_obj.insert(
                            "layout_child_max_nodes_performed".to_string(),
                            Value::from(p.layout_child_max_nodes_performed),
                        );
                        p_obj.insert(
                            "layout_child_max_bounds_changed".to_string(),
                            p.layout_child_max_bounds_changed
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "layout_child_max_bounds_size_changed".to_string(),
                            p.layout_child_max_bounds_size_changed
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "layout_child_max_input_matches_before".to_string(),
                            p.layout_child_max_input_matches_before
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        p_obj.insert(
                            "layout_child_max_input_size_matches_before".to_string(),
                            p.layout_child_max_input_size_matches_before
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        p_obj.insert("total_us".to_string(), Value::from(p.total_us));
                        p_obj.insert(
                            "element_path".to_string(),
                            p.element_path
                                .clone()
                                .map(Value::from)
                                .unwrap_or(Value::Null),
                        );
                        Value::Object(p_obj)
                    })
                    .collect::<Vec<_>>();
                obj.insert(
                    "scroll_layout_profiles".to_string(),
                    Value::Array(scroll_layout_profiles),
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
                        s_obj.insert(
                            "solve_profile".to_string(),
                            s.solve_profile
                                .as_ref()
                                .map(|p| {
                                    let mut p_obj = Map::new();
                                    p_obj.insert(
                                        "reason".to_string(),
                                        Value::from(p.reason.clone()),
                                    );
                                    p_obj.insert(
                                        "available_w_kind".to_string(),
                                        Value::from(p.available_w_kind.clone()),
                                    );
                                    p_obj.insert(
                                        "available_h_kind".to_string(),
                                        Value::from(p.available_h_kind.clone()),
                                    );
                                    p_obj.insert(
                                        "available_w".to_string(),
                                        p.available_w.map(Value::from).unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "available_h".to_string(),
                                        p.available_h.map(Value::from).unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "previous_available_w_kind".to_string(),
                                        p.previous_available_w_kind
                                            .clone()
                                            .map(Value::from)
                                            .unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "previous_available_h_kind".to_string(),
                                        p.previous_available_h_kind
                                            .clone()
                                            .map(Value::from)
                                            .unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "previous_available_w".to_string(),
                                        p.previous_available_w
                                            .map(Value::from)
                                            .unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "previous_available_h".to_string(),
                                        p.previous_available_h
                                            .map(Value::from)
                                            .unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "available_w_delta".to_string(),
                                        p.available_w_delta
                                            .map(Value::from)
                                            .unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "available_h_delta".to_string(),
                                        p.available_h_delta
                                            .map(Value::from)
                                            .unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "scale_factor".to_string(),
                                        Value::from(p.scale_factor),
                                    );
                                    p_obj.insert(
                                        "previous_scale_factor".to_string(),
                                        p.previous_scale_factor
                                            .map(Value::from)
                                            .unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "scale_factor_delta".to_string(),
                                        p.scale_factor_delta
                                            .map(Value::from)
                                            .unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "previous_frame_delta".to_string(),
                                        p.previous_frame_delta
                                            .map(Value::from)
                                            .unwrap_or(Value::Null),
                                    );
                                    p_obj.insert(
                                        "batch_roots".to_string(),
                                        Value::from(p.batch_roots),
                                    );
                                    p_obj.insert(
                                        "subtree_nodes".to_string(),
                                        Value::from(p.subtree_nodes),
                                    );
                                    p_obj.insert(
                                        "flex_wrap_patch_time_us".to_string(),
                                        Value::from(p.flex_wrap_patch_time_us),
                                    );
                                    p_obj.insert(
                                        "flex_wrap_patch_visited_nodes".to_string(),
                                        Value::from(p.flex_wrap_patch_visited_nodes),
                                    );
                                    p_obj.insert(
                                        "flex_wrap_patch_wrap_nodes".to_string(),
                                        Value::from(p.flex_wrap_patch_wrap_nodes),
                                    );
                                    p_obj.insert(
                                        "flex_wrap_patch_candidate_children".to_string(),
                                        Value::from(p.flex_wrap_patch_candidate_children),
                                    );
                                    p_obj.insert(
                                        "flex_wrap_patch_probes".to_string(),
                                        Value::from(p.flex_wrap_patch_probes),
                                    );
                                    p_obj.insert(
                                        "flex_wrap_patch_mutations".to_string(),
                                        Value::from(p.flex_wrap_patch_mutations),
                                    );
                                    p_obj.insert(
                                        "flex_wrap_patch_skipped_no_wrap_descendant".to_string(),
                                        Value::from(
                                            p.flex_wrap_patch_skipped_no_wrap_descendant,
                                        ),
                                    );
                                    Value::Object(p_obj)
                                })
                                .unwrap_or(Value::Null),
                        );
                        s_obj.insert(
                            "clean_geometry_solve_skip_rejection".to_string(),
                            s.clean_geometry_solve_skip_rejection
                                .as_ref()
                                .map(clean_geometry_solve_skip_rejection_to_json)
                                .unwrap_or(Value::Null),
                        );
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
                    .map(BundleStatsPaintWidgetHotspot::to_json)
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

                let command_availability_hotspots = row
                    .command_availability_hotspots
                    .iter()
                    .map(BundleStatsCommandAvailabilityHotspot::to_json)
                    .collect::<Vec<_>>();
                obj.insert(
                    "command_availability_hotspots".to_string(),
                    Value::Array(command_availability_hotspots),
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
