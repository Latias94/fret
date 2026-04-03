use super::super::*;

impl<H: UiHost> UiTree<H> {
    /// Set a node's child list without forcing ancestor relayout.
    ///
    /// This is intended for explicit layout barriers (virtualization, scroll, etc.) whose bounds
    /// are stable and do not depend on the size or presence of their children. In these cases,
    /// structural changes should not require re-laying out ancestors, but the subtree still needs
    /// a contained relayout to place newly mounted children.
    ///
    /// The tree will schedule a contained relayout for `parent` during the next layout pass.
    #[track_caller]
    pub(crate) fn set_children_barrier(&mut self, parent: NodeId, children: Vec<NodeId>) {
        if self.nodes.get(parent).is_none() {
            return;
        };

        // Keep parent pointers consistent even when the child list is unchanged.
        let same_children = self
            .nodes
            .get(parent)
            .is_some_and(|n| n.children.as_slice() == children.as_slice());
        if same_children {
            for &child in &children {
                if let Some(n) = self.nodes.get_mut(child) {
                    n.parent = Some(parent);
                }
            }
            // `set_children_barrier` is used by explicit layout barriers (scroll/virtualization)
            // that may be remounted without changing the child list. Ensure the subtree dirty
            // aggregation stays consistent even when the structural list is identical.
            self.recompute_node_subtree_layout_dirty_count_and_propagate(parent);
            if self.subtree_has_pending_layout_work(parent) {
                if self.debug_enabled {
                    self.debug_stats.barrier_relayouts_scheduled = self
                        .debug_stats
                        .barrier_relayouts_scheduled
                        .saturating_add(1);
                }
                self.schedule_barrier_relayout_with_source_and_detail(
                    parent,
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::Unknown,
                );
            }
            return;
        }

        #[cfg(feature = "diagnostics")]
        if self.debug_enabled {
            let location = std::panic::Location::caller();
            let old_len = self
                .nodes
                .get(parent)
                .map(|n| n.children.len())
                .unwrap_or_default();
            let old_elements_head = self
                .nodes
                .get(parent)
                .map(|n| self.debug_sample_child_elements_head(&n.children))
                .unwrap_or([None; 4]);
            let new_elements_head = self.debug_sample_child_elements_head(&children);
            self.debug_set_children_writes.insert(
                parent,
                UiDebugSetChildrenWrite {
                    parent,
                    frame_id: self.debug_stats.frame_id,
                    old_len: old_len.min(u32::MAX as usize) as u32,
                    new_len: children.len().min(u32::MAX as usize) as u32,
                    old_elements_head,
                    new_elements_head,
                    file: location.file(),
                    line: location.line(),
                    column: location.column(),
                },
            );
        }

        if self.debug_enabled {
            self.debug_stats.set_children_barrier_writes = self
                .debug_stats
                .set_children_barrier_writes
                .saturating_add(1);
            self.debug_stats.barrier_relayouts_scheduled = self
                .debug_stats
                .barrier_relayouts_scheduled
                .saturating_add(1);
        }

        let Some(old_children) = self
            .nodes
            .get_mut(parent)
            .map(|n| std::mem::take(&mut n.children))
        else {
            return;
        };

        for old in old_children {
            if children.contains(&old) {
                continue;
            }
            if let Some(n) = self.nodes.get_mut(old)
                && n.parent == Some(parent)
            {
                #[cfg(feature = "diagnostics")]
                if self.debug_enabled {
                    let location = std::panic::Location::caller();
                    self.debug_parent_sever_writes.insert(
                        old,
                        UiDebugParentSeverWrite {
                            child: old,
                            parent,
                            frame_id: self.debug_stats.frame_id,
                            file: location.file(),
                            line: location.line(),
                            column: location.column(),
                        },
                    );
                }
                n.parent = None;
            }
        }

        for &child in &children {
            if let Some(n) = self.nodes.get_mut(child) {
                n.parent = Some(parent);
            }
        }

        let mut counter_update: Option<(InvalidationFlags, InvalidationFlags)> = None;
        if let Some(n) = self.nodes.get_mut(parent) {
            let prev = n.invalidation;
            n.children = children;
            let layout_before = n.invalidation.layout;
            n.invalidation.mark(Invalidation::HitTest);
            record_layout_invalidation_transition(
                &mut self.layout_invalidations_count,
                layout_before,
                n.invalidation.layout,
            );
            counter_update = Some((prev, n.invalidation));
        }
        if let Some((prev, next)) = counter_update {
            self.update_invalidation_counters(prev, next);
        }

        // Structural changes must invalidate paint/hit-testing so routing and rendering see the
        // updated tree, but we intentionally avoid forcing a full ancestor relayout.
        self.mark_invalidation_with_source(
            parent,
            Invalidation::HitTestOnly,
            UiDebugInvalidationSource::Other,
        );

        // Keep subtree layout-dirty aggregation in sync with barrier child changes.
        self.recompute_node_subtree_layout_dirty_count_and_propagate(parent);

        self.pending_barrier_relayouts.push(parent);
    }

    /// Schedule a contained relayout for an explicit layout barrier without forcing ancestor
    /// relayout.
    ///
    /// This is intended for barrier-internal follow-up work (e.g. deferred probes) where the
    /// barrier viewport/bounds are stable and do not depend on the size of its children.
    pub(crate) fn schedule_barrier_relayout_with_source_and_detail(
        &mut self,
        parent: NodeId,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        if self.nodes.get(parent).is_none() {
            return;
        }

        let mut counter_update: Option<(InvalidationFlags, InvalidationFlags)> = None;
        let mut layout_transition: Option<(bool, bool)> = None;
        if let Some(n) = self.nodes.get_mut(parent) {
            let prev = n.invalidation;
            let layout_before = n.invalidation.layout;
            n.invalidation.mark(Invalidation::Layout);
            let layout_after = n.invalidation.layout;
            record_layout_invalidation_transition(
                &mut self.layout_invalidations_count,
                layout_before,
                layout_after,
            );
            counter_update = Some((prev, n.invalidation));
            layout_transition = Some((layout_before, layout_after));
        }
        if let Some((prev, next)) = counter_update {
            self.update_invalidation_counters(prev, next);
        }
        if let Some((layout_before, layout_after)) = layout_transition {
            self.note_layout_invalidation_transition_for_subtree_aggregation(
                parent,
                layout_before,
                layout_after,
            );
        }

        // Ensure routing/painting sees the follow-up frame, but avoid bubbling a full relayout.
        self.invalidate_with_source_and_detail(parent, Invalidation::HitTestOnly, source, detail);
        self.pending_barrier_relayouts.push(parent);
    }

    #[cfg(feature = "diagnostics")]
    pub(super) fn debug_is_reachable_from_layer_roots(&mut self, node: NodeId) -> bool {
        if !self.debug_enabled {
            return false;
        }

        if let Some((frame_id, reachable)) = &self.debug_reachable_from_layer_roots
            && *frame_id == self.debug_stats.frame_id
        {
            return reachable.contains(&node);
        }

        let roots = self.all_layer_roots();
        let mut reachable: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<NodeId> = roots;
        while let Some(id) = stack.pop() {
            if !reachable.insert(id) {
                continue;
            }
            let Some(entry) = self.nodes.get(id) else {
                continue;
            };
            for &child in &entry.children {
                stack.push(child);
            }
        }

        self.debug_reachable_from_layer_roots = Some((self.debug_stats.frame_id, reachable));
        self.debug_reachable_from_layer_roots
            .as_ref()
            .is_some_and(|(_, reachable)| reachable.contains(&node))
    }

    pub(crate) fn debug_record_virtual_list_visible_range_check(
        &mut self,
        requested_refresh: bool,
    ) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.virtual_list_visible_range_checks = self
            .debug_stats
            .virtual_list_visible_range_checks
            .saturating_add(1);
        if requested_refresh {
            self.debug_stats.virtual_list_visible_range_refreshes = self
                .debug_stats
                .virtual_list_visible_range_refreshes
                .saturating_add(1);
        }
    }

    pub(crate) fn take_pending_barrier_relayouts(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.pending_barrier_relayouts)
    }
}
