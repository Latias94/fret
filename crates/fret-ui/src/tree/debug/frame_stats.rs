use super::super::*;
use fret_core::TimerToken;

#[derive(Debug, Default, Clone, Copy)]
pub struct UiDebugFrameStats {
    pub frame_id: FrameId,
    /// Approximate capacity retained by per-frame scratch (“frame arena”) containers.
    ///
    /// This is a coarse signal intended for diagnostics/triage: it tracks reserved capacity
    /// (not current length) and intentionally underestimates hash table overhead.
    pub frame_arena_capacity_estimate_bytes: u64,
    /// Number of scratch containers that grew their capacity during the current frame.
    ///
    /// This is a proxy for allocator churn in hot paths that should ideally stabilize after
    /// warmup.
    pub frame_arena_grow_events: u32,
    /// Number of child-element vectors reused from the per-window pool during element build.
    ///
    /// This is a proxy for “element tree arena” progress: higher reuse implies less allocator churn
    /// while building the ephemeral declarative element tree.
    pub element_children_vec_pool_reuses: u32,
    /// Number of child-element vectors that had to be newly allocated during element build.
    pub element_children_vec_pool_misses: u32,
    /// Total time spent in event dispatch during the current frame.
    ///
    /// This includes pointer routing, capture/focus arbitration, and widget event hooks. It does
    /// not include layout/prepaint/paint, which are tracked separately.
    pub dispatch_time: Duration,
    /// Number of pointer/drag events dispatched during the current frame.
    pub dispatch_pointer_events: u32,
    /// Total wall time spent dispatching pointer/drag events during the current frame.
    pub dispatch_pointer_event_time: Duration,
    /// Number of timer events dispatched during the current frame.
    pub dispatch_timer_events: u32,
    /// Total wall time spent dispatching timer events during the current frame.
    pub dispatch_timer_event_time: Duration,
    /// Number of timer events that resolved to an explicit element target.
    pub dispatch_timer_targeted_events: u32,
    /// Wall time spent dispatching explicitly targeted timer events.
    pub dispatch_timer_targeted_time: Duration,
    /// Number of timer events that fell back to broadcast delivery (no element target).
    pub dispatch_timer_broadcast_events: u32,
    /// Wall time spent in broadcast timer delivery (including layer scanning + dispatch).
    pub dispatch_timer_broadcast_time: Duration,
    /// Total number of layers visited during broadcast timer delivery.
    pub dispatch_timer_broadcast_layers_visited: u32,
    /// Time spent rebuilding the visible-layers scratch list for broadcast timer delivery.
    pub dispatch_timer_broadcast_rebuild_visible_layers_time: Duration,
    /// Time spent iterating candidate layers and dispatching broadcast timers.
    pub dispatch_timer_broadcast_loop_time: Duration,
    /// Slowest single timer event routing time observed in the current frame.
    pub dispatch_timer_slowest_event_time: Duration,
    /// Token of the slowest timer event observed in the current frame (if any).
    pub dispatch_timer_slowest_token: Option<TimerToken>,
    /// Whether the slowest timer event used the broadcast fallback.
    pub dispatch_timer_slowest_was_broadcast: bool,
    /// Number of non-pointer, non-timer events dispatched during the current frame.
    pub dispatch_other_events: u32,
    /// Total wall time spent dispatching non-pointer, non-timer events during the current frame.
    pub dispatch_other_event_time: Duration,
    /// Time spent inside hit-testing during the current frame (subset of `dispatch_time`).
    pub hit_test_time: Duration,
    /// Number of events dispatched during the current frame.
    pub dispatch_events: u32,
    /// Number of hit-test queries executed during the current frame.
    pub hit_test_queries: u32,
    /// Count of bounds-tree queries attempted during hit testing.
    pub hit_test_bounds_tree_queries: u32,
    /// Bounds-tree queries that were disabled (e.g. env-gated, layer not indexed, or unsupported transforms).
    pub hit_test_bounds_tree_disabled: u32,
    /// Bounds-tree queries that missed (no containing leaf).
    pub hit_test_bounds_tree_misses: u32,
    /// Bounds-tree queries that returned a candidate leaf.
    pub hit_test_bounds_tree_hits: u32,
    /// Bounds-tree candidates rejected by `hit_test_node_self_only`, forcing a fallback traversal.
    pub hit_test_bounds_tree_candidate_rejected: u32,
    /// Total bounds-tree nodes visited across all queries in the current frame.
    pub hit_test_bounds_tree_nodes_visited: u32,
    /// Total bounds-tree nodes pushed to the search stack across all queries in the current frame.
    pub hit_test_bounds_tree_nodes_pushed: u32,
    /// Number of hit-test queries that reused the cached path (no bounds-tree query needed).
    pub hit_test_path_cache_hits: u32,
    /// Number of hit-test queries that fell back to bounds-tree or full traversal (cache miss).
    pub hit_test_path_cache_misses: u32,
    /// Total wall time spent attempting the cached-path hit-test fast path in the current frame.
    pub hit_test_cached_path_time: Duration,
    /// Total wall time spent querying the bounds-tree index in the current frame.
    pub hit_test_bounds_tree_query_time: Duration,
    /// Total wall time spent validating bounds-tree candidates (`hit_test_node_self_only`) in the current frame.
    pub hit_test_candidate_self_only_time: Duration,
    /// Total wall time spent in full traversal fallback hit-testing in the current frame.
    pub hit_test_fallback_traversal_time: Duration,
    /// Total wall time spent updating hover state from pointer hit-testing in the current frame.
    pub dispatch_hover_update_time: Duration,
    /// Total wall time spent applying scroll-handle binding invalidations during event dispatch.
    pub dispatch_scroll_handle_invalidation_time: Duration,
    /// Total wall time spent computing active input layers and enforcing modal barrier scope.
    pub dispatch_active_layers_time: Duration,
    /// Total wall time spent constructing and publishing the window input context snapshot.
    pub dispatch_input_context_time: Duration,
    /// Total wall time spent building the event chain during dispatch.
    pub dispatch_event_chain_build_time: Duration,
    /// Total wall time spent delivering capture-phase widget events during dispatch.
    pub dispatch_widget_capture_time: Duration,
    /// Total wall time spent delivering bubble-phase widget events during dispatch.
    pub dispatch_widget_bubble_time: Duration,
    /// Total wall time spent computing the cursor icon from the pointer hit during dispatch.
    pub dispatch_cursor_query_time: Duration,
    /// Total wall time spent dispatching pointer-move layer observers (both post-dispatch and when
    /// pointer dispatch is suppressed).
    pub dispatch_pointer_move_layer_observers_time: Duration,
    /// Total wall time spent dispatching the synthetic hover-move observer chain when the pointer crosses targets.
    pub dispatch_synth_hover_observer_time: Duration,
    /// Total wall time spent pushing cursor-icon effects during dispatch.
    pub dispatch_cursor_effect_time: Duration,
    /// Total wall time spent publishing post-dispatch window integration snapshots (input context,
    /// command availability) during dispatch.
    pub dispatch_post_dispatch_snapshot_time: Duration,
    pub layout_time: Duration,
    pub layout_collect_roots_time: Duration,
    pub layout_invalidate_scroll_handle_bindings_time: Duration,
    pub layout_expand_view_cache_invalidations_time: Duration,
    pub layout_request_build_roots_time: Duration,
    pub layout_pending_barrier_relayouts_time: Duration,
    pub layout_repair_view_cache_bounds_time: Duration,
    pub layout_contained_view_cache_roots_time: Duration,
    pub layout_collapse_layout_observations_time: Duration,
    /// Total wall time spent recording layout observations (`observed_in_layout` + globals).
    pub layout_observation_record_time: Duration,
    /// Total observed-model edges recorded into `observed_in_layout` during this frame.
    pub layout_observation_record_models_items: u32,
    /// Total observed-global edges recorded into `observed_globals_in_layout` during this frame.
    pub layout_observation_record_globals_items: u32,
    pub layout_prepaint_after_layout_time: Duration,
    pub layout_skipped_engine_frame: bool,
    pub layout_roots_time: Duration,
    pub layout_barrier_relayouts_time: Duration,
    pub layout_view_cache_time: Duration,
    pub layout_semantics_refresh_time: Duration,
    pub layout_focus_repair_time: Duration,
    pub layout_deferred_cleanup_time: Duration,
    pub prepaint_time: Duration,
    pub paint_time: Duration,
    pub paint_record_visual_bounds_time: Duration,
    pub paint_record_visual_bounds_calls: u32,
    /// Total wall time spent computing paint-cache keys and paint-cache enablement checks.
    pub paint_cache_key_time: Duration,
    /// Total wall time spent checking paint-cache hit eligibility (excluding replay itself).
    pub paint_cache_hit_check_time: Duration,
    /// Total wall time spent executing `Widget::paint()` (including push/pop transforms).
    pub paint_widget_time: Duration,
    /// Total wall time spent recording paint observations (`observed_in_paint` + globals).
    pub paint_observation_record_time: Duration,
    /// Total wall time spent iterating element-runtime observed models inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_observed_models_time: Duration,
    /// Total observed-model edges iterated inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_observed_models_items: u32,
    /// Total wall time spent iterating element-runtime observed globals inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_observed_globals_time: Duration,
    /// Total observed-global edges iterated inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_observed_globals_items: u32,
    /// Total wall time spent resolving the element instance (`ElementInstance`) inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_instance_lookup_time: Duration,
    /// Number of `ElementInstance` lookups performed inside `ElementHostWidget::paint_impl`.
    pub paint_host_widget_instance_lookup_calls: u32,
    /// Total wall time spent preparing text blobs (`TextSystem::prepare`) during `Widget::paint`.
    pub paint_text_prepare_time: Duration,
    /// Number of text blob preparations performed during `Widget::paint`.
    pub paint_text_prepare_calls: u32,
    /// Count of text prepares where the cached blob was missing.
    pub paint_text_prepare_reason_blob_missing: u32,
    /// Count of text prepares triggered by a scale factor change.
    pub paint_text_prepare_reason_scale_changed: u32,
    /// Count of text prepares triggered by a plain-text change.
    pub paint_text_prepare_reason_text_changed: u32,
    /// Count of text prepares triggered by an attributed-text change.
    pub paint_text_prepare_reason_rich_changed: u32,
    /// Count of text prepares triggered by a text-style change.
    pub paint_text_prepare_reason_style_changed: u32,
    /// Count of text prepares triggered by a wrap-mode change.
    pub paint_text_prepare_reason_wrap_changed: u32,
    /// Count of text prepares triggered by an overflow-mode change.
    pub paint_text_prepare_reason_overflow_changed: u32,
    /// Count of text prepares triggered by a max-width change.
    pub paint_text_prepare_reason_width_changed: u32,
    /// Count of text prepares triggered by a font-stack-key change.
    pub paint_text_prepare_reason_font_stack_changed: u32,
    pub paint_input_context_time: Duration,
    pub paint_scroll_handle_invalidation_time: Duration,
    pub paint_collect_roots_time: Duration,
    pub paint_publish_text_input_snapshot_time: Duration,
    pub paint_collapse_observations_time: Duration,
    pub layout_nodes_visited: u32,
    pub layout_nodes_performed: u32,
    pub prepaint_nodes_visited: u32,
    pub paint_nodes: u32,
    pub paint_nodes_performed: u32,
    pub paint_cache_hits: u32,
    pub paint_cache_misses: u32,
    pub paint_cache_replayed_ops: u32,
    /// Paint-cache replay attempts that were allowed specifically by the
    /// `FRET_UI_PAINT_CACHE_ALLOW_HIT_TEST_ONLY` gate.
    pub paint_cache_hit_test_only_replay_allowed: u32,
    /// Hit-test-only replay attempts rejected because the previous cache key did not match.
    pub paint_cache_hit_test_only_replay_rejected_key_mismatch: u32,
    pub paint_cache_replay_time: Duration,
    pub paint_cache_bounds_translate_time: Duration,
    pub paint_cache_bounds_translated_nodes: u32,
    pub interaction_cache_hits: u32,
    pub interaction_cache_misses: u32,
    pub interaction_cache_replayed_records: u32,
    pub interaction_records: u32,
    /// Number of layout engine root solves performed during the current frame.
    pub layout_engine_solves: u64,
    /// Total time spent in layout engine solves during the current frame.
    pub layout_engine_solve_time: Duration,
    /// Total number of `layout_engine_child_local_rect` queries performed during the current frame.
    pub layout_engine_child_rect_queries: u64,
    /// Total wall time spent inside layout engine child-rect queries during the current frame.
    pub layout_engine_child_rect_time: Duration,
    /// Number of "widget-local" layout engine solves triggered as a fallback when a widget cannot
    /// consume already-solved engine child rects.
    ///
    /// The goal for v2 is to keep this at `0` for normal UI trees by ensuring explicit layout
    /// barriers (scroll/virtualization/splits/...) register viewport roots or explicitly solve
    /// their child roots.
    pub layout_engine_widget_fallback_solves: u64,
    pub layout_fast_path_taken: bool,
    pub layout_invalidations_count: u32,
    pub layout_subtree_dirty_agg_enabled: bool,
    /// Number of aggregation update operations performed during the current frame.
    pub layout_subtree_dirty_agg_updates: u32,
    /// Total number of nodes whose aggregation counter was updated during the current frame.
    pub layout_subtree_dirty_agg_nodes_touched: u32,
    /// Max parent-walk length observed in a single aggregation update during the current frame.
    pub layout_subtree_dirty_agg_max_parent_walk: u32,
    /// Total nodes processed by subtree rebuilds during the current frame.
    pub layout_subtree_dirty_agg_rebuild_nodes: u32,
    /// Count of validation failures observed during the current frame.
    pub layout_subtree_dirty_agg_validation_failures: u32,
    /// Unique nodes observed as invalidation roots for model changes during the current frame.
    pub model_change_invalidation_roots: u32,
    /// Count of changed models consumed for propagation during the current frame.
    pub model_change_models: u32,
    /// Total (model -> node) observation edges scanned during propagation.
    pub model_change_observation_edges: u32,
    /// Count of changed models with no observation edges.
    pub model_change_unobserved_models: u32,
    /// Unique nodes observed as invalidation roots for global changes during the current frame.
    pub global_change_invalidation_roots: u32,
    /// Count of changed globals consumed for propagation during the current frame.
    pub global_change_globals: u32,
    /// Total (global -> node) observation edges scanned during propagation.
    pub global_change_observation_edges: u32,
    /// Count of changed globals with no observation edges.
    pub global_change_unobserved_globals: u32,
    /// Total nodes visited across invalidation walks during the current frame.
    pub invalidation_walk_nodes: u32,
    /// Total invalidation walks performed during the current frame.
    pub invalidation_walk_calls: u32,
    /// Nodes visited across invalidation walks attributed to model changes.
    pub invalidation_walk_nodes_model_change: u32,
    /// Invalidation walks attributed to model changes.
    pub invalidation_walk_calls_model_change: u32,
    /// Nodes visited across invalidation walks attributed to global changes.
    pub invalidation_walk_nodes_global_change: u32,
    /// Invalidation walks attributed to global changes.
    pub invalidation_walk_calls_global_change: u32,
    /// Nodes visited across invalidation walks attributed to hover state changes.
    pub invalidation_walk_nodes_hover: u32,
    /// Invalidation walks attributed to hover state changes.
    pub invalidation_walk_calls_hover: u32,
    /// Nodes visited across invalidation walks attributed to focus changes.
    pub invalidation_walk_nodes_focus: u32,
    /// Invalidation walks attributed to focus changes.
    pub invalidation_walk_calls_focus: u32,
    /// Nodes visited across invalidation walks attributed to all other sources.
    pub invalidation_walk_nodes_other: u32,
    /// Invalidation walks attributed to all other sources.
    pub invalidation_walk_calls_other: u32,
    /// Count of hover target changes for `Pressable` instances during the current frame.
    pub hover_pressable_target_changes: u32,
    /// Count of hover target changes for `HoverRegion` instances during the current frame.
    pub hover_hover_region_target_changes: u32,
    /// Count of declarative instance changes that happened in a frame that also observed a hover
    /// target change.
    pub hover_declarative_instance_changes: u32,
    /// Count of declarative `HitTest` invalidations attributed to hover during the current frame.
    pub hover_declarative_hit_test_invalidations: u32,
    /// Count of declarative `Layout` invalidations attributed to hover during the current frame.
    pub hover_declarative_layout_invalidations: u32,
    /// Count of declarative `Paint` invalidations attributed to hover during the current frame.
    pub hover_declarative_paint_invalidations: u32,
    /// Whether view-cache mode is active for this frame.
    pub view_cache_active: bool,
    /// How many invalidation walks were truncated by a view-cache boundary.
    pub view_cache_invalidation_truncations: u32,
    /// How many "contained" view-cache roots were re-laid out during the final pass.
    pub view_cache_contained_relayouts: u32,
    /// How many view-cache roots were observed during the current frame.
    pub view_cache_roots_total: u32,
    /// How many view-cache roots were reused during the current frame.
    pub view_cache_roots_reused: u32,
    /// View-cache roots that were not reused because they were mounted for the first time.
    pub view_cache_roots_first_mount: u32,
    /// View-cache roots that were not reused because their backing `NodeId` was recreated.
    pub view_cache_roots_node_recreated: u32,
    /// View-cache roots that were not reused because the declarative cache key did not match.
    pub view_cache_roots_cache_key_mismatch: u32,
    /// View-cache roots that were not reused because they were not marked as reuse roots.
    ///
    /// This is an authoring-level signal: either view-cache was not enabled, or reuse was gated off
    /// by local state (e.g. `view_cache_needs_rerender` / layout invalidation) upstream.
    pub view_cache_roots_not_marked_reuse_root: u32,
    /// View-cache roots that were not reused because `view_cache_needs_rerender` was set.
    pub view_cache_roots_needs_rerender: u32,
    /// View-cache roots that were not reused because they had a layout invalidation.
    pub view_cache_roots_layout_invalidated: u32,
    /// View-cache roots recorded from retained/manual cache roots (non-declarative).
    pub view_cache_roots_manual: u32,
    /// How many times `set_children_barrier` was applied (structural changes without forcing
    /// ancestor relayout).
    pub set_children_barrier_writes: u32,
    /// How many barrier relayout roots were scheduled via `set_children_barrier` in this frame.
    pub barrier_relayouts_scheduled: u32,
    /// How many barrier relayout roots were actually laid out in this frame.
    pub barrier_relayouts_performed: u32,
    /// How many VirtualList visible-range checks were evaluated (used to request rerenders under
    /// view-cache reuse).
    pub virtual_list_visible_range_checks: u32,
    /// How many VirtualList visible-range checks requested a refresh (range delta outside the
    /// currently mounted span).
    pub virtual_list_visible_range_refreshes: u32,
    /// How many VirtualList window shifts were observed during the current frame.
    pub virtual_list_window_shifts_total: u32,
    /// How many VirtualList window shifts required a non-retained cache-root rerender.
    pub virtual_list_window_shifts_non_retained: u32,
    /// How many retained VirtualList hosts were reconciled (attach/detach without rerendering the
    /// parent view-cache root).
    pub retained_virtual_list_reconciles: u32,
    /// Total items attached across retained VirtualList reconciles (new keys mounted).
    pub retained_virtual_list_attached_items: u32,
    /// Total items detached across retained VirtualList reconciles (keys removed from children).
    pub retained_virtual_list_detached_items: u32,
    pub focus: Option<NodeId>,
    pub captured: Option<NodeId>,
}
