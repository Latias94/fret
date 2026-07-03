use super::super::*;
use crate::tree::node_storage::{
    TextWrapNoneMeasureCache, TextWrappedMeasureCache, TextWrappedMeasureCaches,
};

impl<H: UiHost> UiTree<H> {
    pub(crate) fn create_node(&mut self, widget: impl Widget<H> + 'static) -> NodeId {
        let node = Node::new(widget);
        let inv = node.invalidation;
        let id = self.nodes.insert(node);
        self.sync_view_boundary_state_for_node(id);
        self.mark_semantics_dirty();
        self.update_invalidation_counters(InvalidationFlags::default(), inv);
        if inv.layout {
            self.layout_invalidations_count = self.layout_invalidations_count.saturating_add(1);
            self.debug_note_layout_dirty_source(
                id,
                id,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::InitialMount,
            );
        }
        id
    }

    pub(crate) fn set_node_text_wrap_none_measure_cache(
        &mut self,
        node: NodeId,
        fingerprint: u64,
        size: Size,
    ) {
        let Some(n) = self.nodes.get_mut(node) else {
            return;
        };
        n.text_wrap_none_measure_cache = Some(TextWrapNoneMeasureCache { fingerprint, size });
    }

    pub(crate) fn clear_node_text_wrap_none_measure_cache(&mut self, node: NodeId) {
        let Some(n) = self.nodes.get_mut(node) else {
            return;
        };
        n.text_wrap_none_measure_cache = None;
    }

    pub(crate) fn set_node_text_wrapped_measure_cache(
        &mut self,
        node: NodeId,
        fingerprint: u64,
        constraints_max_width: Option<Px>,
        measured_size: Size,
        clamped_size: Size,
    ) {
        let Some(n) = self.nodes.get_mut(node) else {
            return;
        };
        let cache = n
            .text_wrapped_measure_cache
            .get_or_insert_with(TextWrappedMeasureCaches::default);
        cache.insert(TextWrappedMeasureCache {
            fingerprint,
            constraints_max_width,
            measured_size,
            clamped_size,
        });
    }

    pub(crate) fn clear_node_text_wrapped_measure_cache(&mut self, node: NodeId) {
        let Some(n) = self.nodes.get_mut(node) else {
            return;
        };
        n.text_wrapped_measure_cache = None;
    }

    #[cfg(test)]
    pub(crate) fn create_node_for_element(
        &mut self,
        element: GlobalElementId,
        widget: impl Widget<H> + 'static,
    ) -> NodeId {
        let node = Node::new_for_element(element, widget);
        let inv = node.invalidation;
        let id = self.nodes.insert(node);
        self.sync_view_boundary_state_for_node(id);
        self.mark_semantics_dirty();
        self.update_invalidation_counters(InvalidationFlags::default(), inv);
        if inv.layout {
            self.layout_invalidations_count = self.layout_invalidations_count.saturating_add(1);
            self.debug_note_layout_dirty_source(
                id,
                id,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::InitialMount,
            );
        }
        id
    }

    #[cfg(test)]
    pub(crate) fn test_clear_node_invalidations(&mut self, node: NodeId) {
        let Some((layout_before, layout_after)) = self.nodes.get_mut(node).map(|n| {
            let layout_before = n.invalidation.layout;
            n.invalidation.clear();
            n.paint_invalidated_by_hit_test_only = false;
            let layout_after = n.invalidation.layout;
            (layout_before, layout_after)
        }) else {
            return;
        };
        record_layout_invalidation_transition(
            &mut self.layout_invalidations_count,
            layout_before,
            layout_after,
        );
        self.note_layout_invalidation_transition_for_subtree_aggregation(
            node,
            layout_before,
            layout_after,
        );
        if layout_before && !layout_after {
            self.debug_clear_layout_dirty_source(node);
        }
    }

    #[cfg(test)]
    pub(crate) fn test_node_invalidations(&self, node: NodeId) -> Option<InvalidationFlags> {
        self.nodes.get(node).map(|n| n.invalidation)
    }

    #[cfg(test)]
    pub(crate) fn test_invalidation_counters(&self) -> (u32, u32, u32) {
        (
            self.layout_invalidations_count,
            self.invalidated_layout_nodes,
            self.invalidated_paint_nodes,
        )
    }

    #[cfg(test)]
    pub(crate) fn test_set_layout_invalidation(&mut self, node: NodeId, value: bool) {
        let view_cache_active = self.view_cache_active();
        let Some((layout_before, layout_after, should_mark_contained_cache_root_dirty)) =
            self.nodes.get_mut(node).map(|n| {
                let layout_before = n.invalidation.layout;
                n.invalidation.layout = value;
                if value {
                    n.invalidation.paint = true;
                }
                let should_mark_contained_cache_root_dirty = value
                    && view_cache_active
                    && n.view_cache.enabled
                    && n.view_cache.layout_contained_when_bounds_known();
                let layout_after = n.invalidation.layout;
                (
                    layout_before,
                    layout_after,
                    should_mark_contained_cache_root_dirty,
                )
            })
        else {
            return;
        };
        record_layout_invalidation_transition(
            &mut self.layout_invalidations_count,
            layout_before,
            layout_after,
        );
        self.note_layout_invalidation_transition_for_subtree_aggregation(
            node,
            layout_before,
            layout_after,
        );
        if !layout_before && layout_after {
            self.debug_note_layout_dirty_source(
                node,
                node,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::LocalInvalidation,
            );
        } else if layout_before && !layout_after {
            self.debug_clear_layout_dirty_source(node);
        }

        if should_mark_contained_cache_root_dirty {
            self.mark_boundary_layout_dirty(
                node,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::LocalInvalidation,
            );
        } else if !value {
            self.clear_boundary_layout_dirty(node);
        }
    }

    #[cfg(test)]
    pub(crate) fn test_set_paint_invalidation(&mut self, node: NodeId, value: bool) {
        let Some((prev, next)) = self.nodes.get_mut(node).map(|n| {
            let prev = n.invalidation;
            n.invalidation.paint = value;
            if !value {
                n.paint_invalidated_by_hit_test_only = false;
            }
            (prev, n.invalidation)
        }) else {
            return;
        };
        self.update_invalidation_counters(prev, next);
    }

    #[cfg(test)]
    pub(crate) fn test_set_node_parent(&mut self, node: NodeId, parent: Option<NodeId>) {
        let Some(n) = self.nodes.get_mut(node) else {
            return;
        };
        n.parent = parent;
    }

    pub(in crate::tree) fn set_node_children_write_policy(
        &mut self,
        node: NodeId,
        policy: ChildrenWritePolicy,
    ) {
        let Some(entry) = self.nodes.get_mut(node) else {
            return;
        };
        entry.children_write_policy = policy;
    }

    pub(in crate::tree) fn detach_reparented_children_from_old_parents(
        &mut self,
        parent: NodeId,
        children: &[NodeId],
    ) {
        let mut removals: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
        for &child in children {
            let Some(old_parent) = self.nodes.get(child).and_then(|node| node.parent) else {
                continue;
            };
            if old_parent == parent {
                continue;
            }
            removals.entry(old_parent).or_default().insert(child);
        }

        for (old_parent, removing) in removals {
            let Some(old_children) = self.nodes.get(old_parent).map(|node| node.children.clone())
            else {
                continue;
            };
            if !old_children.iter().any(|child| removing.contains(child)) {
                continue;
            }
            let filtered: Vec<NodeId> = old_children
                .into_iter()
                .filter(|child| !removing.contains(child))
                .collect();
            let policy = self
                .nodes
                .get(old_parent)
                .map(|node| node.children_write_policy)
                .unwrap_or_default();
            match policy {
                ChildrenWritePolicy::Standard => self.set_children(old_parent, filtered),
                ChildrenWritePolicy::Barrier => self.set_children_barrier(old_parent, filtered),
            }
        }
    }

    pub fn set_root(&mut self, root: NodeId) {
        let _ = self.set_base_root(root);
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        let Some(mut parent_children) = self.nodes.get(parent).map(|node| node.children.clone())
        else {
            return;
        };
        if !self.nodes.contains_key(child) {
            return;
        }

        let old_parent = self.nodes.get(child).and_then(|node| node.parent);
        let occurrences_in_parent = parent_children.iter().filter(|&&id| id == child).count();

        if old_parent == Some(parent) && occurrences_in_parent == 1 {
            return;
        }

        if let Some(old_parent) = old_parent
            && old_parent != parent
            && let Some(old_children) = self.nodes.get(old_parent).map(|node| node.children.clone())
            && old_children.contains(&child)
        {
            let filtered_old_children: Vec<NodeId> =
                old_children.into_iter().filter(|&id| id != child).collect();
            self.set_children(old_parent, filtered_old_children);
        }

        parent_children.retain(|&id| id != child);
        parent_children.push(child);
        self.set_children(parent, parent_children);
    }

    #[track_caller]
    pub fn set_children(&mut self, parent: NodeId, children: Vec<NodeId>) {
        if self.nodes.get(parent).is_none() {
            return;
        }

        self.set_node_children_write_policy(parent, ChildrenWritePolicy::Standard);
        self.detach_reparented_children_from_old_parents(parent, &children);

        let Some(_old_len) = self.nodes.get(parent).map(|n| n.children.len()) else {
            return;
        };

        // Keep the direct retained parent edge consistent even when the child list is unchanged.
        //
        // Normal topology queries use child edges, but `Node.parent` remains retained storage
        // metadata for the touched edge.
        let same_children = self
            .nodes
            .get(parent)
            .is_some_and(|n| n.children.as_slice() == children.as_slice());
        if same_children {
            self.sync_same_children_parent_edges_and_reconnect_layout(parent, &children);
            if self.node_is_reachable_from_layer_forest(parent) {
                for &child in &children {
                    self.index_live_subtree(child);
                    self.sync_live_view_boundary_state_for_subtree(child);
                }
            }
            return;
        }

        #[cfg(feature = "diagnostics")]
        if self.debug_enabled {
            let location = std::panic::Location::caller();
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
                    old_len: _old_len.min(u32::MAX as usize) as u32,
                    new_len: children.len().min(u32::MAX as usize) as u32,
                    old_elements_head,
                    new_elements_head,
                    file: location.file(),
                    line: location.line(),
                    column: location.column(),
                },
            );
        }

        let Some(old_children) = self
            .nodes
            .get_mut(parent)
            .map(|n| std::mem::take(&mut n.children))
        else {
            return;
        };
        let parent_was_live = self.node_is_reachable_from_layer_forest(parent);
        if parent_was_live {
            for &old in &old_children {
                self.unindex_detached_child_subtree(old);
                self.detach_view_boundary_state_for_subtree(old);
            }
        }

        for old in old_children {
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

        let new_children_for_index = children.clone();
        let mut propagate = false;
        let mut counter_update: Option<(InvalidationFlags, InvalidationFlags)> = None;
        let mut layout_transition: Option<(bool, bool)> = None;
        if let Some(n) = self.nodes.get_mut(parent) {
            let prev = n.invalidation;
            n.children = children;
            let layout_before = n.invalidation.layout;
            n.invalidation.mark(Invalidation::HitTest);
            let layout_after = n.invalidation.layout;
            record_layout_invalidation_transition(
                &mut self.layout_invalidations_count,
                layout_before,
                layout_after,
            );
            counter_update = Some((prev, n.invalidation));
            layout_transition = Some((layout_before, layout_after));
            propagate = true;
        }
        if let Some((prev, next)) = counter_update {
            self.update_invalidation_counters(prev, next);
        }
        if let Some((layout_before, layout_after)) = layout_transition
            && !layout_before
            && layout_after
        {
            self.debug_note_layout_dirty_source(
                parent,
                parent,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::StructuralChildrenChanged,
            );
        }

        self.invalidate_dispatch_snapshot_cache();
        self.recompute_node_subtree_layout_dirty_count_and_propagate(parent);

        if parent_was_live {
            for child in new_children_for_index {
                self.index_live_subtree(child);
                self.sync_live_view_boundary_state_for_subtree(child);
            }
        }

        if propagate {
            // Structural changes must invalidate ancestors so the next layout pass walks far
            // enough to place newly mounted subtrees, even when view-cache invalidation
            // truncation is enabled.
            self.invalidate_with_source_and_detail(
                parent,
                Invalidation::HitTest,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::StructuralChildrenChanged,
            );
        }
    }

    pub(in crate::tree) fn sync_same_children_parent_edges_and_reconnect_layout(
        &mut self,
        parent: NodeId,
        children: &[NodeId],
    ) {
        let mut repaired_parent_pointer = false;
        for &child in children {
            if let Some(n) = self.nodes.get_mut(child) {
                repaired_parent_pointer |= n.parent != Some(parent);
                n.parent = Some(parent);
            }
        }
        if repaired_parent_pointer {
            self.bump_command_availability_revision();
        }

        self.recompute_node_subtree_layout_dirty_count_and_propagate(parent);

        if repaired_parent_pointer
            && self.subtree_has_pending_layout_work(parent)
            && self
                .nodes
                .get(parent)
                .is_some_and(|node| !node.invalidation.layout)
        {
            // Same-children writes can reconnect a touched direct parent edge after a descendant
            // became layout-dirty while detached. Mark the parent so the next layout pass descends
            // back into the child-edge subtree.
            self.invalidate_with_source_and_detail(
                parent,
                Invalidation::Layout,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::StructuralParentRepair,
            );
        }
    }

    pub(in crate::tree) fn subtree_has_pending_layout_work(&self, root: NodeId) -> bool {
        self.node_subtree_layout_dirty(root)
    }
}
