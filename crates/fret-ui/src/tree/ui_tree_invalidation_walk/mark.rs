use super::super::*;

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn mark_invalidation(&mut self, node: NodeId, inv: Invalidation) {
        self.mark_invalidation_with_source(node, inv, UiDebugInvalidationSource::Other);
    }

    pub(in crate::tree) fn invalidation_marks_view_dirty(
        source: UiDebugInvalidationSource,
        inv: Invalidation,
        detail: UiDebugInvalidationDetail,
    ) -> bool {
        matches!(
            source,
            UiDebugInvalidationSource::Notify
                | UiDebugInvalidationSource::ModelChange
                | UiDebugInvalidationSource::GlobalChange
        ) || matches!(detail, UiDebugInvalidationDetail::HoverRegionEdge)
            || (inv != Invalidation::Paint
                && matches!(
                    detail,
                    UiDebugInvalidationDetail::ScrollHandleLayout
                        | UiDebugInvalidationDetail::ScrollHandleWindowUpdate
                        | UiDebugInvalidationDetail::ScrollHandleScrollToItemWindowUpdate
                        | UiDebugInvalidationDetail::ScrollHandleViewportResizeWindowUpdate
                        | UiDebugInvalidationDetail::ScrollHandleItemsRevisionWindowUpdate
                        | UiDebugInvalidationDetail::ScrollHandlePrefetchWindowUpdate
                ))
    }

    fn record_invalidation_walk_call(&mut self, source: UiDebugInvalidationSource) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.invalidation_walk_calls =
            self.debug_stats.invalidation_walk_calls.saturating_add(1);
        match source {
            UiDebugInvalidationSource::ModelChange => {
                self.debug_stats.invalidation_walk_calls_model_change = self
                    .debug_stats
                    .invalidation_walk_calls_model_change
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::GlobalChange => {
                self.debug_stats.invalidation_walk_calls_global_change = self
                    .debug_stats
                    .invalidation_walk_calls_global_change
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Notify => {
                self.debug_stats.invalidation_walk_calls_other = self
                    .debug_stats
                    .invalidation_walk_calls_other
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Hover => {
                self.debug_stats.invalidation_walk_calls_hover = self
                    .debug_stats
                    .invalidation_walk_calls_hover
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Focus => {
                self.debug_stats.invalidation_walk_calls_focus = self
                    .debug_stats
                    .invalidation_walk_calls_focus
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Other => {
                self.debug_stats.invalidation_walk_calls_other = self
                    .debug_stats
                    .invalidation_walk_calls_other
                    .saturating_add(1);
            }
        }
    }

    fn record_invalidation_walk_node(&mut self, source: UiDebugInvalidationSource) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.invalidation_walk_nodes =
            self.debug_stats.invalidation_walk_nodes.saturating_add(1);
        match source {
            UiDebugInvalidationSource::ModelChange => {
                self.debug_stats.invalidation_walk_nodes_model_change = self
                    .debug_stats
                    .invalidation_walk_nodes_model_change
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::GlobalChange => {
                self.debug_stats.invalidation_walk_nodes_global_change = self
                    .debug_stats
                    .invalidation_walk_nodes_global_change
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Notify => {
                self.debug_stats.invalidation_walk_nodes_other = self
                    .debug_stats
                    .invalidation_walk_nodes_other
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Hover => {
                self.debug_stats.invalidation_walk_nodes_hover = self
                    .debug_stats
                    .invalidation_walk_nodes_hover
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Focus => {
                self.debug_stats.invalidation_walk_nodes_focus = self
                    .debug_stats
                    .invalidation_walk_nodes_focus
                    .saturating_add(1);
            }
            UiDebugInvalidationSource::Other => {
                self.debug_stats.invalidation_walk_nodes_other = self
                    .debug_stats
                    .invalidation_walk_nodes_other
                    .saturating_add(1);
            }
        }
    }

    pub(in crate::tree) fn mark_invalidation_with_source(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
    ) {
        let detail = UiDebugInvalidationDetail::from_source(source);
        self.mark_invalidation_with_detail(node, inv, source, detail);
    }

    fn mark_invalidation_with_detail(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        let stop_at_view_cache = self.view_cache_active();
        let agg_enabled = self.subtree_layout_dirty_aggregation_enabled();
        self.record_invalidation_walk_call(source);
        let mut current = Some(node);
        let mut hit_cache_root: Option<NodeId> = None;
        let root_element = self.nodes.get(node).and_then(|n| n.element);
        let mut walked_nodes: u32 = 0;
        let mut invalidation_active = true;
        let mut pending_layout_dirty_delta: i32 = 0;
        let mut agg_walk_len: u32 = 0;
        while let Some(id) = current {
            if invalidation_active && self.nodes.contains_key(id) {
                self.record_invalidation_walk_node(source);
                walked_nodes = walked_nodes.saturating_add(1);
            }
            let mut did_stop = false;
            let mut mark_dirty = false;
            let mut counter_update: Option<(InvalidationFlags, InvalidationFlags)> = None;
            let mut self_delta: i32 = 0;
            let mut rebuild_subtree_layout_dirty: bool = false;
            let next_parent = if let Some(n) = self.nodes.get_mut(id) {
                let next_parent = n.parent;
                if invalidation_active {
                    let prev = n.invalidation;
                    let layout_before = n.invalidation.layout;
                    Self::mark_node_invalidation_state(n, inv);
                    let layout_after = n.invalidation.layout;
                    record_layout_invalidation_transition(
                        &mut self.layout_invalidations_count,
                        layout_before,
                        layout_after,
                    );
                    counter_update = Some((prev, n.invalidation));
                    self_delta = match (layout_before, layout_after) {
                        (false, true) => 1,
                        (true, false) => -1,
                        _ => 0,
                    };

                    let can_truncate_at_cache_root = inv == Invalidation::Paint
                        || (n.view_cache.contained_layout
                            && n.view_cache.layout_definite
                            && n.bounds.size != Size::default())
                        // For auto-sized cache roots, allow descendant invalidations to truncate at
                        // the first cache boundary we hit. A separate repair step
                        // (`propagate_auto_sized_view_cache_root_invalidations`) will propagate a
                        // single invalidation from the cache root to its ancestors so the root can be
                        // placed before running contained relayouts.
                        //
                        // Importantly, do *not* truncate when the invalidation originates at the
                        // cache root itself (e.g. the repair step), so it can still reach ancestors.
                        || (n.view_cache.contained_layout
                            && !n.view_cache.layout_definite
                            && id != node);
                    if stop_at_view_cache && n.view_cache.enabled && can_truncate_at_cache_root {
                        if self.debug_enabled {
                            self.debug_stats.view_cache_invalidation_truncations = self
                                .debug_stats
                                .view_cache_invalidation_truncations
                                .saturating_add(1);
                        }
                        hit_cache_root = Some(id);
                        did_stop = true;
                        if Self::invalidation_marks_view_dirty(source, inv, detail) {
                            n.view_cache_needs_rerender = true;
                            mark_dirty = true;
                        }
                    }
                }

                if agg_enabled {
                    let apply_delta = pending_layout_dirty_delta.saturating_add(self_delta);
                    if apply_delta != 0 {
                        let underflow =
                            super::super::ui_tree_subtree_layout_dirty::apply_i32_delta_to_u32(
                                &mut n.subtree_layout_dirty_count,
                                apply_delta,
                            );
                        if underflow {
                            rebuild_subtree_layout_dirty = true;
                            tracing::error!(
                                node = ?id,
                                element = ?n.element,
                                stored = n.subtree_layout_dirty_count,
                                delta = apply_delta,
                                "subtree layout dirty count underflow during invalidation walk"
                            );
                        }
                    }
                }
                next_parent
            } else {
                break;
            };

            if let Some((prev, next)) = counter_update {
                self.update_invalidation_counters(prev, next);
            }

            if rebuild_subtree_layout_dirty {
                self.repair_subtree_layout_dirty_counts_from(id);
            }

            if agg_enabled {
                agg_walk_len = agg_walk_len.saturating_add(1);
                pending_layout_dirty_delta = if rebuild_subtree_layout_dirty {
                    0
                } else {
                    pending_layout_dirty_delta.saturating_add(self_delta)
                };
            }

            if did_stop {
                if mark_dirty {
                    self.mark_cache_root_dirty(id, source, detail);
                }
                invalidation_active = false;
            }
            if !invalidation_active && (!agg_enabled || pending_layout_dirty_delta == 0) {
                break;
            }
            current = next_parent;
        }

        if agg_enabled && self.debug_enabled && agg_walk_len > 0 && pending_layout_dirty_delta != 0
        {
            self.debug_stats.layout_subtree_dirty_agg_updates = self
                .debug_stats
                .layout_subtree_dirty_agg_updates
                .saturating_add(1);
            self.debug_stats.layout_subtree_dirty_agg_nodes_touched = self
                .debug_stats
                .layout_subtree_dirty_agg_nodes_touched
                .saturating_add(agg_walk_len);
            self.debug_stats.layout_subtree_dirty_agg_max_parent_walk = self
                .debug_stats
                .layout_subtree_dirty_agg_max_parent_walk
                .max(agg_walk_len);
        }

        if self.debug_enabled {
            self.debug_invalidation_walks.push(UiDebugInvalidationWalk {
                root: node,
                root_element,
                inv,
                source,
                detail,
                walked_nodes,
                truncated_at: hit_cache_root,
            });
        }

        // Nested cache-root correctness: if a descendant cache root is invalidated, any ancestor
        // cache roots must also be invalidated for the same categories so they cannot replay stale
        // recorded ranges that include the old descendant output.
        if stop_at_view_cache && let Some(cache_root) = hit_cache_root {
            let mut parent = self.nodes.get(cache_root).and_then(|n| n.parent);
            while let Some(id) = parent {
                let next_parent = self.nodes.get(id).and_then(|n| n.parent);
                let mut mark_dirty = false;
                let mut counter_update: Option<(InvalidationFlags, InvalidationFlags)> = None;
                let mut layout_transition: Option<(NodeId, bool, bool)> = None;
                if let Some(n) = self.nodes.get_mut(id)
                    && n.view_cache.enabled
                {
                    let prev = n.invalidation;
                    let layout_before = n.invalidation.layout;
                    Self::mark_node_invalidation_state(n, inv);
                    let layout_after = n.invalidation.layout;
                    record_layout_invalidation_transition(
                        &mut self.layout_invalidations_count,
                        layout_before,
                        layout_after,
                    );
                    layout_transition = Some((id, layout_before, layout_after));
                    counter_update = Some((prev, n.invalidation));
                    if Self::invalidation_marks_view_dirty(source, inv, detail) {
                        n.view_cache_needs_rerender = true;
                        mark_dirty = true;
                    }
                }
                if let Some((id, before, after)) = layout_transition {
                    self.note_layout_invalidation_transition_for_subtree_aggregation(
                        id, before, after,
                    );
                }
                if let Some((prev, next)) = counter_update {
                    self.update_invalidation_counters(prev, next);
                }
                if mark_dirty {
                    self.mark_cache_root_dirty(id, source, detail);
                }
                parent = next_parent;
            }
        }
    }

    fn invalidation_mask(inv: Invalidation) -> u8 {
        const PAINT: u8 = 1 << 0;
        const LAYOUT: u8 = 1 << 1;
        const HIT_TEST: u8 = 1 << 2;
        match inv {
            Invalidation::Paint => PAINT,
            Invalidation::Layout => PAINT | LAYOUT,
            Invalidation::HitTest => PAINT | LAYOUT | HIT_TEST,
            Invalidation::HitTestOnly => PAINT | HIT_TEST,
        }
    }

    pub(in crate::tree) fn mark_invalidation_dedup_with_source<V: InvalidationVisited>(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        visited: &mut V,
        source: UiDebugInvalidationSource,
    ) {
        let detail = UiDebugInvalidationDetail::from_source(source);
        self.mark_invalidation_dedup_with_detail(node, inv, visited, source, detail);
    }

    pub(in crate::tree) fn mark_invalidation_dedup_with_detail<V: InvalidationVisited>(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        visited: &mut V,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        let stop_at_view_cache = self.view_cache_active();
        let agg_enabled = self.subtree_layout_dirty_aggregation_enabled();
        let needed = Self::invalidation_mask(inv);
        if source != UiDebugInvalidationSource::Notify && (visited.mask(node) & needed) == needed {
            return;
        }
        self.record_invalidation_walk_call(source);

        let mut current = Some(node);
        let mut hit_cache_root: Option<NodeId> = None;
        let root_element = self.nodes.get(node).and_then(|n| n.element);
        let mut walked_nodes: u32 = 0;
        let mut invalidation_active = true;
        let mut pending_layout_dirty_delta: i32 = 0;
        let mut agg_walk_len: u32 = 0;
        while let Some(id) = current {
            let already = visited.mask(id);
            if invalidation_active
                && source != UiDebugInvalidationSource::Notify
                && (already & needed) == needed
                && !(stop_at_view_cache && Self::invalidation_marks_view_dirty(source, inv, detail))
            {
                invalidation_active = false;
                if !agg_enabled || pending_layout_dirty_delta == 0 {
                    break;
                }
            }

            if invalidation_active && self.nodes.contains_key(id) {
                self.record_invalidation_walk_node(source);
                walked_nodes = walked_nodes.saturating_add(1);
            }
            let mut did_stop = false;
            let mut mark_dirty = false;
            let mut self_delta: i32 = 0;
            let mut rebuild_subtree_layout_dirty: bool = false;
            let next_parent = if let Some(n) = self.nodes.get_mut(id) {
                let next_parent = n.parent;
                let mut counter_update: Option<(InvalidationFlags, InvalidationFlags)> = None;
                if invalidation_active
                    && (source == UiDebugInvalidationSource::Notify || (already & needed) != needed)
                {
                    let prev = n.invalidation;
                    let layout_before = n.invalidation.layout;
                    Self::mark_node_invalidation_state(n, inv);
                    record_layout_invalidation_transition(
                        &mut self.layout_invalidations_count,
                        layout_before,
                        n.invalidation.layout,
                    );
                    visited.set_mask(id, already | needed);
                    counter_update = Some((prev, n.invalidation));
                    self_delta = match (layout_before, n.invalidation.layout) {
                        (false, true) => 1,
                        (true, false) => -1,
                        _ => 0,
                    };
                }

                if invalidation_active {
                    let can_truncate_at_cache_root = inv == Invalidation::Paint
                        || (n.view_cache.contained_layout
                            && n.view_cache.layout_definite
                            && n.bounds.size != Size::default())
                        || (n.view_cache.contained_layout
                            && !n.view_cache.layout_definite
                            && id != node);
                    if stop_at_view_cache && n.view_cache.enabled && can_truncate_at_cache_root {
                        if self.debug_enabled {
                            self.debug_stats.view_cache_invalidation_truncations = self
                                .debug_stats
                                .view_cache_invalidation_truncations
                                .saturating_add(1);
                        }
                        if Self::invalidation_marks_view_dirty(source, inv, detail) {
                            n.view_cache_needs_rerender = true;
                            mark_dirty = true;
                        }
                        hit_cache_root = Some(id);
                        did_stop = true;
                    }
                }

                if agg_enabled {
                    let apply_delta = pending_layout_dirty_delta.saturating_add(self_delta);
                    if apply_delta != 0 {
                        let underflow =
                            super::super::ui_tree_subtree_layout_dirty::apply_i32_delta_to_u32(
                                &mut n.subtree_layout_dirty_count,
                                apply_delta,
                            );
                        if underflow {
                            rebuild_subtree_layout_dirty = true;
                            tracing::error!(
                                node = ?id,
                                element = ?n.element,
                                stored = n.subtree_layout_dirty_count,
                                delta = apply_delta,
                                "subtree layout dirty count underflow during invalidation walk"
                            );
                        }
                    }
                }

                if let Some((prev, next)) = counter_update {
                    self.update_invalidation_counters(prev, next);
                }
                next_parent
            } else {
                break;
            };

            if rebuild_subtree_layout_dirty {
                self.repair_subtree_layout_dirty_counts_from(id);
            }

            if did_stop {
                if mark_dirty {
                    self.mark_cache_root_dirty(id, source, detail);
                }
                invalidation_active = false;
            }
            if agg_enabled {
                agg_walk_len = agg_walk_len.saturating_add(1);
                pending_layout_dirty_delta = if rebuild_subtree_layout_dirty {
                    0
                } else {
                    pending_layout_dirty_delta.saturating_add(self_delta)
                };
            }
            if !invalidation_active && (!agg_enabled || pending_layout_dirty_delta == 0) {
                break;
            }
            current = next_parent;
        }

        if agg_enabled && self.debug_enabled && agg_walk_len > 0 && pending_layout_dirty_delta != 0
        {
            self.debug_stats.layout_subtree_dirty_agg_updates = self
                .debug_stats
                .layout_subtree_dirty_agg_updates
                .saturating_add(1);
            self.debug_stats.layout_subtree_dirty_agg_nodes_touched = self
                .debug_stats
                .layout_subtree_dirty_agg_nodes_touched
                .saturating_add(agg_walk_len);
            self.debug_stats.layout_subtree_dirty_agg_max_parent_walk = self
                .debug_stats
                .layout_subtree_dirty_agg_max_parent_walk
                .max(agg_walk_len);
        }

        if self.debug_enabled {
            self.debug_invalidation_walks.push(UiDebugInvalidationWalk {
                root: node,
                root_element,
                inv,
                source,
                detail,
                walked_nodes,
                truncated_at: hit_cache_root,
            });
        }

        // Nested cache-root correctness: if a descendant cache root is invalidated, any ancestor
        // cache roots must also be invalidated for the same categories so they cannot replay stale
        // recorded ranges that include the old descendant output.
        if stop_at_view_cache && let Some(cache_root) = hit_cache_root {
            let mut parent = self.nodes.get(cache_root).and_then(|n| n.parent);
            while let Some(id) = parent {
                let next_parent = self.nodes.get(id).and_then(|n| n.parent);
                let already = visited.mask(id);
                if self.nodes.get(id).is_some_and(|n| n.view_cache.enabled) {
                    let mut mark_dirty = false;
                    let mut counter_update: Option<(InvalidationFlags, InvalidationFlags)> = None;
                    let mut layout_transition: Option<(NodeId, bool, bool)> = None;
                    if let Some(n) = self.nodes.get_mut(id) {
                        if Self::invalidation_marks_view_dirty(source, inv, detail) {
                            n.view_cache_needs_rerender = true;
                            mark_dirty = true;
                        }
                        if (already & needed) != needed {
                            let prev = n.invalidation;
                            let layout_before = n.invalidation.layout;
                            Self::mark_node_invalidation_state(n, inv);
                            let layout_after = n.invalidation.layout;
                            record_layout_invalidation_transition(
                                &mut self.layout_invalidations_count,
                                layout_before,
                                layout_after,
                            );
                            layout_transition = Some((id, layout_before, layout_after));
                            counter_update = Some((prev, n.invalidation));
                        }
                    }
                    if let Some((id, before, after)) = layout_transition {
                        self.note_layout_invalidation_transition_for_subtree_aggregation(
                            id, before, after,
                        );
                    }
                    if let Some((prev, next)) = counter_update {
                        self.update_invalidation_counters(prev, next);
                    }
                    if mark_dirty {
                        self.mark_cache_root_dirty(id, source, detail);
                    }
                    visited.set_mask(id, already | needed);
                }
                parent = next_parent;
            }
        }
    }

    pub fn invalidate(&mut self, node: NodeId, inv: Invalidation) {
        self.mark_invalidation(node, inv);
    }

    pub fn invalidate_with_source(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
    ) {
        let detail = UiDebugInvalidationDetail::from_source(source);
        self.mark_invalidation_with_detail(node, inv, source, detail);
    }

    pub fn invalidate_with_detail(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        detail: UiDebugInvalidationDetail,
    ) {
        self.mark_invalidation_with_detail(node, inv, UiDebugInvalidationSource::Other, detail);
    }

    pub fn invalidate_with_source_and_detail(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        self.mark_invalidation_with_detail(node, inv, source, detail);
    }
}
