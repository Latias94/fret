use super::super::*;

impl<H: UiHost> UiTree<H> {
    #[cfg(debug_assertions)]
    pub(crate) fn debug_note_declarative_render_root_called(&mut self, frame_id: FrameId) {
        self.debug_last_declarative_render_root_frame_id = Some(frame_id);
    }

    #[cfg(debug_assertions)]
    pub(crate) fn debug_forbid_propagate_after_declarative_render_root(&self, frame_id: FrameId) {
        if !crate::runtime_config::ui_runtime_config().debug_forbid_propagate_after_render_root {
            return;
        }

        if self.debug_last_declarative_render_root_frame_id == Some(frame_id) {
            panic!(
                "ui.propagate_*_changes() was called after declarative::render_root() in the same frame (frame_id={:?}). \
                 Call frame_pipeline::propagate_changes() (or UiTree::propagate_*_changes()) before mounting the declarative root.",
                frame_id
            );
        }
    }

    pub(crate) fn begin_debug_frame_if_needed(&mut self, frame_id: FrameId) {
        if !self.debug_enabled {
            return;
        }
        if self.debug_stats.frame_id == frame_id {
            return;
        }

        self.debug_stats.frame_id = frame_id;
        self.debug_stats.frame_arena_capacity_estimate_bytes =
            self.frame_arena.capacity_estimate_bytes();
        self.debug_stats.frame_arena_grow_events = 0;
        self.debug_stats.element_children_vec_pool_reuses = 0;
        self.debug_stats.element_children_vec_pool_misses = 0;
        self.debug_stats.dispatch_time = Duration::default();
        self.debug_stats.dispatch_pointer_events = u32::default();
        self.debug_stats.dispatch_pointer_event_time = Duration::default();
        self.debug_stats.dispatch_timer_events = u32::default();
        self.debug_stats.dispatch_timer_event_time = Duration::default();
        self.debug_stats.dispatch_timer_targeted_events = u32::default();
        self.debug_stats.dispatch_timer_targeted_time = Duration::default();
        self.debug_stats.dispatch_timer_broadcast_events = u32::default();
        self.debug_stats.dispatch_timer_broadcast_time = Duration::default();
        self.debug_stats.dispatch_timer_broadcast_layers_visited = u32::default();
        self.debug_stats
            .dispatch_timer_broadcast_rebuild_visible_layers_time = Duration::default();
        self.debug_stats.dispatch_timer_broadcast_loop_time = Duration::default();
        self.debug_stats.dispatch_timer_slowest_event_time = Duration::default();
        self.debug_stats.dispatch_timer_slowest_token = None;
        self.debug_stats.dispatch_timer_slowest_was_broadcast = bool::default();
        self.debug_stats.dispatch_other_events = u32::default();
        self.debug_stats.dispatch_other_event_time = Duration::default();
        self.debug_stats.hit_test_time = Duration::default();
        self.debug_stats.dispatch_events = 0;
        self.debug_stats.hit_test_queries = 0;
        self.debug_stats.hit_test_bounds_tree_queries = 0;
        self.debug_stats.hit_test_bounds_tree_disabled = 0;
        self.debug_stats.hit_test_bounds_tree_misses = 0;
        self.debug_stats.hit_test_bounds_tree_hits = 0;
        self.debug_stats.hit_test_bounds_tree_candidate_rejected = 0;
        self.debug_stats.hit_test_bounds_tree_nodes_visited = 0;
        self.debug_stats.hit_test_bounds_tree_nodes_pushed = 0;
        self.debug_stats.hit_test_path_cache_hits = 0;
        self.debug_stats.hit_test_path_cache_misses = 0;
        self.debug_stats.hit_test_cached_path_time = Duration::default();
        self.debug_stats.hit_test_bounds_tree_query_time = Duration::default();
        self.debug_stats.hit_test_candidate_self_only_time = Duration::default();
        self.debug_stats.hit_test_fallback_traversal_time = Duration::default();
        self.debug_stats.dispatch_hover_update_time = Duration::default();
        self.debug_stats.dispatch_scroll_handle_invalidation_time = Duration::default();
        self.debug_stats.dispatch_active_layers_time = Duration::default();
        self.debug_stats.dispatch_input_context_time = Duration::default();
        self.debug_stats.dispatch_event_chain_build_time = Duration::default();
        self.debug_stats.dispatch_widget_capture_time = Duration::default();
        self.debug_stats.dispatch_widget_bubble_time = Duration::default();
        self.debug_stats.dispatch_cursor_query_time = Duration::default();
        self.debug_stats.dispatch_pointer_move_layer_observers_time = Duration::default();
        self.debug_stats.dispatch_synth_hover_observer_time = Duration::default();
        self.debug_stats.dispatch_cursor_effect_time = Duration::default();
        self.debug_stats.dispatch_post_dispatch_snapshot_time = Duration::default();
        self.debug_stats.layout_roots_time = Duration::default();
        self.debug_stats.layout_barrier_relayouts_time = Duration::default();
        self.debug_stats.layout_view_cache_time = Duration::default();
        self.debug_stats.layout_semantics_refresh_time = Duration::default();
        self.debug_stats.layout_focus_repair_time = Duration::default();
        self.debug_stats.layout_deferred_cleanup_time = Duration::default();
        self.debug_stats.model_change_invalidation_roots = 0;
        self.debug_stats.model_change_models = 0;
        self.debug_stats.model_change_observation_edges = 0;
        self.debug_stats.model_change_unobserved_models = 0;
        self.debug_stats.global_change_invalidation_roots = 0;
        self.debug_stats.global_change_globals = 0;
        self.debug_stats.global_change_observation_edges = 0;
        self.debug_stats.global_change_unobserved_globals = 0;
        self.debug_stats.invalidation_walk_nodes = 0;
        self.debug_stats.invalidation_walk_calls = 0;
        self.debug_stats.invalidation_walk_nodes_model_change = 0;
        self.debug_stats.invalidation_walk_calls_model_change = 0;
        self.debug_stats.invalidation_walk_nodes_global_change = 0;
        self.debug_stats.invalidation_walk_calls_global_change = 0;
        self.debug_stats.invalidation_walk_nodes_hover = 0;
        self.debug_stats.invalidation_walk_calls_hover = 0;
        self.debug_stats.invalidation_walk_nodes_focus = 0;
        self.debug_stats.invalidation_walk_calls_focus = 0;
        self.debug_stats.invalidation_walk_nodes_other = 0;
        self.debug_stats.invalidation_walk_calls_other = 0;
        self.debug_stats.hover_pressable_target_changes = 0;
        self.debug_stats.hover_hover_region_target_changes = 0;
        self.debug_stats.hover_declarative_instance_changes = 0;
        self.debug_stats.hover_declarative_hit_test_invalidations = 0;
        self.debug_stats.hover_declarative_layout_invalidations = 0;
        self.debug_stats.hover_declarative_paint_invalidations = 0;
        self.debug_stats.view_cache_active = self.view_cache_active();
        self.debug_stats.view_cache_invalidation_truncations = 0;
        self.debug_stats.view_cache_contained_relayouts = 0;
        self.debug_stats.view_cache_roots_total = 0;
        self.debug_stats.view_cache_roots_reused = 0;
        self.debug_stats.view_cache_roots_first_mount = 0;
        self.debug_stats.view_cache_roots_node_recreated = 0;
        self.debug_stats.view_cache_roots_cache_key_mismatch = 0;
        self.debug_stats.view_cache_roots_not_marked_reuse_root = 0;
        self.debug_stats.view_cache_roots_needs_rerender = 0;
        self.debug_stats.view_cache_roots_layout_invalidated = 0;
        self.debug_stats.view_cache_roots_manual = 0;
        self.debug_stats.set_children_barrier_writes = 0;
        self.debug_stats.barrier_relayouts_scheduled = 0;
        self.debug_stats.barrier_relayouts_performed = 0;
        self.debug_stats.virtual_list_visible_range_checks = 0;
        self.debug_stats.virtual_list_visible_range_refreshes = 0;
        self.debug_stats.virtual_list_window_shifts_total = 0;
        self.debug_stats.virtual_list_window_shifts_non_retained = 0;
        self.debug_stats.retained_virtual_list_reconciles = 0;
        self.debug_stats.retained_virtual_list_attached_items = 0;
        self.debug_stats.retained_virtual_list_detached_items = 0;

        self.debug_view_cache_roots.clear();
        self.debug_view_cache_contained_relayout_roots.clear();
        self.debug_paint_cache_replays.clear();
        self.debug_layout_engine_solves.clear();
        self.debug_layout_hotspots.clear();
        self.debug_layout_inclusive_hotspots.clear();
        self.debug_layout_stack.clear();
        self.debug_widget_measure_hotspots.clear();
        self.debug_widget_measure_stack.clear();
        self.debug_measure_children.clear();
        self.debug_invalidation_walks.clear();
        self.debug_model_change_hotspots.clear();
        self.debug_model_change_unobserved.clear();
        self.debug_global_change_hotspots.clear();
        self.debug_global_change_unobserved.clear();
        self.debug_hover_edge_this_frame = false;
        self.debug_hover_declarative_invalidations.clear();
        self.debug_dirty_views.clear();
        #[cfg(feature = "diagnostics")]
        self.debug_notify_requests.clear();
        self.debug_virtual_list_windows.clear();
        self.debug_virtual_list_window_shift_samples.clear();
        self.debug_retained_virtual_list_reconciles.clear();
        self.debug_scroll_handle_changes.clear();
        self.debug_scroll_nodes.clear();
        self.debug_scrollbars.clear();
        self.debug_prepaint_actions.clear();
        #[cfg(feature = "diagnostics")]
        {
            // Keep `debug_set_children_writes` and `debug_parent_sever_writes` across frames so
            // GC sweep records can point back to the structural operation that detached an island.
        }
        #[cfg(feature = "diagnostics")]
        self.debug_layer_visible_writes.clear();
        #[cfg(feature = "diagnostics")]
        self.debug_overlay_policy_decisions.clear();
        #[cfg(feature = "diagnostics")]
        self.debug_remove_subtree_frame_context.clear();
        #[cfg(feature = "diagnostics")]
        self.debug_removed_subtrees.clear();
        self.debug_paint_widget_hotspots.clear();
        self.debug_paint_text_prepare_hotspots.clear();
        self.debug_paint_stack.clear();
        #[cfg(feature = "diagnostics")]
        {
            self.debug_reachable_from_layer_roots = None;
            self.debug_text_constraints_measured.clear();
            self.debug_text_constraints_prepared.clear();
        }
        let mut dirty_roots: Vec<NodeId> = self.dirty_cache_roots.iter().copied().collect();
        dirty_roots.sort_by_key(|id| id.data().as_ffi());
        for root in dirty_roots {
            let element = self.nodes.get(root).and_then(|n| n.element);
            let (source, detail) = self
                .dirty_cache_root_reasons
                .get(&root)
                .copied()
                .unwrap_or((
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::Unknown,
                ));
            self.debug_dirty_views.push(UiDebugDirtyView {
                view: ViewId(root),
                element,
                source,
                detail,
            });
        }
    }
}
