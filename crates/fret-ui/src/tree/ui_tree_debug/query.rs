use super::super::*;

impl<H: UiHost> UiTree<H> {
    pub fn debug_cache_root_stats(&self) -> Vec<UiDebugCacheRootStats> {
        if !self.debug_enabled {
            return Vec::new();
        }

        let mut out: Vec<UiDebugCacheRootStats> = self
            .debug_view_cache_roots
            .iter()
            .map(|r| UiDebugCacheRootStats {
                root: r.root,
                element: self.nodes.get(r.root).and_then(|n| n.element),
                reused: r.reused,
                contained_layout: r.contained_layout,
                paint_replayed_ops: self
                    .debug_paint_cache_replays
                    .get(&r.root)
                    .copied()
                    .unwrap_or(0),
                reuse_reason: r.reuse_reason,
            })
            .collect();

        out.sort_by_key(|s| std::cmp::Reverse(s.paint_replayed_ops));
        out
    }

    pub fn debug_view_cache_contained_relayout_roots(&self) -> &[NodeId] {
        if !self.debug_enabled {
            return &[];
        }
        &self.debug_view_cache_contained_relayout_roots
    }

    #[cfg(feature = "diagnostics")]
    pub fn debug_set_children_write_for(&self, parent: NodeId) -> Option<UiDebugSetChildrenWrite> {
        if !self.debug_enabled {
            return None;
        }
        self.debug_set_children_writes.get(&parent).copied()
    }

    #[cfg(feature = "diagnostics")]
    pub fn debug_parent_sever_write_for(&self, child: NodeId) -> Option<UiDebugParentSeverWrite> {
        if !self.debug_enabled {
            return None;
        }
        self.debug_parent_sever_writes.get(&child).copied()
    }

    #[cfg(feature = "diagnostics")]
    pub fn debug_layer_visible_writes(&self) -> &[UiDebugSetLayerVisibleWrite] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_layer_visible_writes.as_slice()
    }

    #[cfg(feature = "diagnostics")]
    pub fn debug_overlay_policy_decisions(&self) -> &[UiDebugOverlayPolicyDecisionWrite] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_overlay_policy_decisions.as_slice()
    }

    pub(crate) fn debug_record_notify_request(
        &mut self,
        frame_id: FrameId,
        caller_node: NodeId,
        location: Option<crate::widget::UiSourceLocation>,
    ) {
        #[cfg(feature = "diagnostics")]
        {
            if !self.debug_enabled {
                return;
            }

            let Some(location) = location else {
                return;
            };

            // Mirror the v1 notify routing: the default target is the nearest view-cache root,
            // falling back to the caller node when no cache boundary exists.
            let target = self
                .nearest_view_cache_root(caller_node)
                .unwrap_or(caller_node);

            if self.debug_notify_requests.len() >= 256 {
                return;
            }

            self.debug_notify_requests.push(UiDebugNotifyRequest {
                frame_id,
                caller_node,
                target_view: ViewId(target),
                file: location.file,
                line: location.line,
                column: location.column,
            });
        }

        #[cfg(not(feature = "diagnostics"))]
        {
            let _ = (frame_id, caller_node, location);
        }
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub fn debug_record_overlay_policy_decision(
        &mut self,
        frame_id: FrameId,
        layer: UiLayerId,
        kind: &'static str,
        present: bool,
        interactive: bool,
        wants_timer_events: bool,
        reason: &'static str,
    ) {
        #[cfg(feature = "diagnostics")]
        {
            if !self.debug_enabled {
                return;
            }
            let caller = std::panic::Location::caller();
            self.debug_overlay_policy_decisions
                .push(UiDebugOverlayPolicyDecisionWrite {
                    layer,
                    frame_id,
                    kind,
                    present,
                    interactive,
                    wants_timer_events,
                    reason,
                    file: caller.file(),
                    line: caller.line(),
                    column: caller.column(),
                });
        }

        #[cfg(not(feature = "diagnostics"))]
        {
            let _ = (
                frame_id,
                layer,
                kind,
                present,
                interactive,
                wants_timer_events,
                reason,
            );
        }
    }

    #[cfg(feature = "diagnostics")]
    pub(crate) fn debug_set_remove_subtree_frame_context(
        &mut self,
        root: NodeId,
        ctx: UiDebugRemoveSubtreeFrameContext,
    ) {
        if !self.debug_enabled {
            return;
        }
        self.debug_remove_subtree_frame_context.insert(root, ctx);
    }

    #[cfg(feature = "diagnostics")]
    pub fn debug_removed_subtrees(&self) -> &[UiDebugRemoveSubtreeRecord] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_removed_subtrees.as_slice()
    }

    pub fn debug_layout_engine_solves(&self) -> &[UiDebugLayoutEngineSolve] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_layout_engine_solves.as_slice()
    }

    pub fn debug_layout_hotspots(&self) -> &[UiDebugLayoutHotspot] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_layout_hotspots.as_slice()
    }

    pub fn debug_layout_inclusive_hotspots(&self) -> &[UiDebugLayoutHotspot] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_layout_inclusive_hotspots.as_slice()
    }

    pub fn debug_widget_measure_hotspots(&self) -> &[UiDebugWidgetMeasureHotspot] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_widget_measure_hotspots.as_slice()
    }

    pub fn debug_paint_widget_hotspots(&self) -> &[UiDebugPaintWidgetHotspot] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_paint_widget_hotspots.as_slice()
    }

    pub fn debug_paint_text_prepare_hotspots(&self) -> &[UiDebugPaintTextPrepareHotspot] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_paint_text_prepare_hotspots.as_slice()
    }

    #[cfg(feature = "diagnostics")]
    pub(in crate::tree) fn debug_sample_child_elements_head(
        &self,
        children: &[NodeId],
    ) -> [Option<GlobalElementId>; 4] {
        let mut out: [Option<GlobalElementId>; 4] = [None; 4];
        for (i, &child) in children.iter().take(out.len()).enumerate() {
            out[i] = self.nodes.get(child).and_then(|n| n.element);
        }
        out
    }

    pub fn set_debug_enabled(&mut self, enabled: bool) {
        self.debug_enabled = enabled;
    }

    pub(crate) fn debug_enabled(&self) -> bool {
        self.debug_enabled
    }

    pub fn debug_stats(&self) -> UiDebugFrameStats {
        self.debug_stats
    }

    pub(crate) fn debug_set_element_children_vec_pool_stats(&mut self, reuses: u32, misses: u32) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.element_children_vec_pool_reuses = reuses;
        self.debug_stats.element_children_vec_pool_misses = misses;
    }

    #[cfg(test)]
    pub(crate) fn debug_measure_child_calls_for_parent(&self, parent: NodeId) -> u64 {
        self.debug_measure_children
            .get(&parent)
            .map(|m| m.values().map(|r| r.calls).sum())
            .unwrap_or(0)
    }

    pub fn debug_hover_declarative_invalidation_hotspots(
        &self,
        max: usize,
    ) -> Vec<UiDebugHoverDeclarativeInvalidationHotspot> {
        if !self.debug_enabled || max == 0 {
            return Vec::new();
        }

        let mut out: Vec<UiDebugHoverDeclarativeInvalidationHotspot> = self
            .debug_hover_declarative_invalidations
            .iter()
            .map(
                |(&node, counts)| UiDebugHoverDeclarativeInvalidationHotspot {
                    node,
                    element: self.nodes.get(node).and_then(|n| n.element),
                    hit_test: counts.hit_test,
                    layout: counts.layout,
                    paint: counts.paint,
                },
            )
            .collect();

        out.sort_by_key(|hs| {
            (
                std::cmp::Reverse(hs.layout),
                std::cmp::Reverse(hs.hit_test),
                std::cmp::Reverse(hs.paint),
            )
        });
        out.truncate(max);
        out
    }

    pub fn debug_invalidation_walks(&self) -> &[UiDebugInvalidationWalk] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_invalidation_walks.as_slice()
    }

    pub fn debug_dirty_views(&self) -> &[UiDebugDirtyView] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_dirty_views.as_slice()
    }

    pub fn debug_notify_requests(&self) -> &[UiDebugNotifyRequest] {
        #[cfg(feature = "diagnostics")]
        {
            if !self.debug_enabled {
                &[]
            } else {
                self.debug_notify_requests.as_slice()
            }
        }

        #[cfg(not(feature = "diagnostics"))]
        {
            &[]
        }
    }

    pub fn debug_virtual_list_windows(&self) -> &[UiDebugVirtualListWindow] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_virtual_list_windows.as_slice()
    }

    pub fn debug_virtual_list_window_shift_samples(
        &self,
    ) -> &[UiDebugVirtualListWindowShiftSample] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_virtual_list_window_shift_samples.as_slice()
    }

    pub fn debug_retained_virtual_list_reconciles(&self) -> &[UiDebugRetainedVirtualListReconcile] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_retained_virtual_list_reconciles.as_slice()
    }

    pub fn debug_scroll_handle_changes(&self) -> &[UiDebugScrollHandleChange] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_scroll_handle_changes.as_slice()
    }

    pub fn debug_scroll_nodes(&self) -> &[UiDebugScrollNodeTelemetry] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_scroll_nodes.as_slice()
    }

    pub fn debug_scrollbars(&self) -> &[UiDebugScrollbarTelemetry] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_scrollbars.as_slice()
    }

    pub fn debug_prepaint_actions(&self) -> &[UiDebugPrepaintAction] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_prepaint_actions.as_slice()
    }

    pub fn debug_model_change_hotspots(&self) -> &[UiDebugModelChangeHotspot] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_model_change_hotspots.as_slice()
    }

    pub fn debug_model_change_unobserved(&self) -> &[UiDebugModelChangeUnobserved] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_model_change_unobserved.as_slice()
    }

    pub fn debug_global_change_hotspots(&self) -> &[UiDebugGlobalChangeHotspot] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_global_change_hotspots.as_slice()
    }

    pub fn debug_global_change_unobserved(&self) -> &[UiDebugGlobalChangeUnobserved] {
        if !self.debug_enabled {
            return &[];
        }
        self.debug_global_change_unobserved.as_slice()
    }

    pub fn debug_node_bounds(&self, node: NodeId) -> Option<Rect> {
        self.nodes.get(node).map(|n| n.bounds)
    }

    pub fn debug_node_element(&self, node: NodeId) -> Option<GlobalElementId> {
        self.nodes.get(node).and_then(|n| n.element)
    }

    pub fn debug_node_clips_hit_test(&self, node: NodeId) -> Option<bool> {
        let n = self.nodes.get(node)?;
        let widget = n.widget.as_ref();
        let prepaint = (!self.inspection_active && !n.invalidation.hit_test)
            .then_some(n.prepaint_hit_test)
            .flatten();
        Some(
            prepaint
                .as_ref()
                .map(|p| p.clips_hit_test)
                .unwrap_or_else(|| widget.map(|w| w.clips_hit_test(n.bounds)).unwrap_or(true)),
        )
    }

    pub fn debug_node_can_scroll_descendant_into_view(&self, node: NodeId) -> Option<bool> {
        let n = self.nodes.get(node)?;
        let widget = n.widget.as_ref();
        let prepaint = (!self.inspection_active && !n.invalidation.hit_test)
            .then_some(n.prepaint_hit_test)
            .flatten();
        Some(
            prepaint
                .as_ref()
                .map(|p| p.can_scroll_descendant_into_view)
                .unwrap_or_else(|| {
                    widget
                        .map(|w| w.can_scroll_descendant_into_view())
                        .unwrap_or(false)
                }),
        )
    }

    pub fn debug_node_render_transform(&self, node: NodeId) -> Option<Transform2D> {
        let n = self.nodes.get(node)?;
        let widget = n.widget.as_ref();
        let prepaint = (!self.inspection_active && !n.invalidation.hit_test)
            .then_some(n.prepaint_hit_test)
            .flatten();
        if let Some(inv) = prepaint.as_ref().and_then(|p| p.render_transform_inv) {
            return inv.inverse();
        }
        widget.and_then(|w| w.render_transform(n.bounds))
    }

    pub fn debug_node_children_render_transform(&self, node: NodeId) -> Option<Transform2D> {
        let n = self.nodes.get(node)?;
        let widget = n.widget.as_ref();
        let prepaint = (!self.inspection_active && !n.invalidation.hit_test)
            .then_some(n.prepaint_hit_test)
            .flatten();
        if let Some(inv) = prepaint
            .as_ref()
            .and_then(|p| p.children_render_transform_inv)
        {
            return inv.inverse();
        }
        widget.and_then(|w| w.children_render_transform(n.bounds))
    }

    pub fn debug_text_constraints_snapshot(&self, node: NodeId) -> UiDebugTextConstraintsSnapshot {
        #[cfg(feature = "diagnostics")]
        {
            return UiDebugTextConstraintsSnapshot {
                measured: self.debug_text_constraints_measured.get(&node).copied(),
                prepared: self.debug_text_constraints_prepared.get(&node).copied(),
            };
        }
        #[cfg(not(feature = "diagnostics"))]
        {
            let _ = node;
        }

        #[allow(unreachable_code)]
        UiDebugTextConstraintsSnapshot::default()
    }

    /// Returns the node bounds after applying the accumulated `render_transform` stack.
    ///
    /// This is intended for debugging and tests that need screen-space geometry for overlay
    /// placement/hit-testing scenarios. Unlike `debug_node_bounds`, this includes render-time
    /// transforms such as `Anchored` placement.
    ///
    /// This is not a stable cross-frame geometry query (see
    /// `fret_ui::elements::visual_bounds_for_element` for that contract).
    pub fn debug_node_visual_bounds(&self, node: NodeId) -> Option<Rect> {
        let bounds = self.nodes.get(node).map(|n| n.bounds)?;
        let path = self.debug_node_path(node);
        let mut before = Transform2D::IDENTITY;
        let mut transform = Transform2D::IDENTITY;
        for (idx, id) in path.iter().copied().enumerate() {
            let node_transform = self
                .debug_node_render_transform(id)
                .unwrap_or(Transform2D::IDENTITY);
            let at_node = before.compose(node_transform);
            if id == node {
                transform = at_node;
                break;
            }
            let child_transform = self
                .debug_node_children_render_transform(id)
                .unwrap_or(Transform2D::IDENTITY);
            before = at_node.compose(child_transform);

            // Defensive: if the node wasn't found in `path`, keep identity.
            if idx == path.len().saturating_sub(1) {
                transform = at_node;
            }
        }

        Some(rect_aabb_transformed(bounds, transform))
    }

    pub fn debug_node_path(&self, node: NodeId) -> Vec<NodeId> {
        let mut out: Vec<NodeId> = Vec::new();
        let mut current = Some(node);
        while let Some(id) = current {
            out.push(id);
            current = self.nodes.get(id).and_then(|n| n.parent);
        }
        out.reverse();
        out
    }

    pub fn debug_layers_in_paint_order(&self) -> Vec<UiDebugLayerInfo> {
        self.layer_order
            .iter()
            .copied()
            .filter_map(|id| {
                let layer = self.layers.get(id)?;
                Some(UiDebugLayerInfo {
                    id,
                    root: layer.root,
                    visible: layer.visible,
                    blocks_underlay_input: layer.blocks_underlay_input,
                    hit_testable: layer.hit_testable,
                    pointer_occlusion: layer.pointer_occlusion,
                    wants_pointer_down_outside_events: layer.wants_pointer_down_outside_events,
                    consume_pointer_down_outside_events: layer.consume_pointer_down_outside_events,
                    pointer_down_outside_branches: layer.pointer_down_outside_branches.clone(),
                    wants_pointer_move_events: layer.wants_pointer_move_events,
                    wants_timer_events: layer.wants_timer_events,
                })
            })
            .collect()
    }

    pub fn debug_hit_test(&self, position: Point) -> UiDebugHitTest {
        let (active_roots, barrier_root) = self.active_input_layers();
        let hit = self.hit_test_layers(&active_roots, position);
        UiDebugHitTest {
            hit,
            active_layer_roots: active_roots,
            barrier_root,
        }
    }

    /// Hit-test using the same cached fast paths used by pointer routing.
    ///
    /// This is intended for diagnostics tooling that needs “what the runtime would route right
    /// now”, including bounds-tree acceleration and view-cache/prepaint interaction data.
    ///
    /// Note: this method mutates internal hit-test caches.
    pub fn debug_hit_test_routing(&mut self, position: Point) -> UiDebugHitTest {
        let (active_roots, barrier_root) = self.active_input_layers();

        // Avoid leaking a stale cached path into diagnostics queries. Pointer routing already
        // manages when the cache is eligible for reuse (e.g. move vs click).
        self.hit_test_path_cache = None;

        let hit = self.hit_test_layers_cached(&active_roots, position);
        UiDebugHitTest {
            hit,
            active_layer_roots: active_roots,
            barrier_root,
        }
    }
}
