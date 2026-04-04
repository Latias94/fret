use super::super::*;
use crate::tree::node_storage::TextWrapNoneMeasureCache;

impl<H: UiHost> UiTree<H> {
    pub(crate) fn create_node(&mut self, widget: impl Widget<H> + 'static) -> NodeId {
        let node = Node::new(widget);
        let inv = node.invalidation;
        let id = self.nodes.insert(node);
        self.update_invalidation_counters(InvalidationFlags::default(), inv);
        if inv.layout {
            self.layout_invalidations_count = self.layout_invalidations_count.saturating_add(1);
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

    #[cfg(test)]
    pub(crate) fn create_node_for_element(
        &mut self,
        element: GlobalElementId,
        widget: impl Widget<H> + 'static,
    ) -> NodeId {
        let node = Node::new_for_element(element, widget);
        let inv = node.invalidation;
        let id = self.nodes.insert(node);
        self.update_invalidation_counters(InvalidationFlags::default(), inv);
        if inv.layout {
            self.layout_invalidations_count = self.layout_invalidations_count.saturating_add(1);
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
                    && n.view_cache.contained_layout;
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

        if should_mark_contained_cache_root_dirty {
            self.mark_cache_root_dirty(
                node,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::Unknown,
            );
        } else if !value {
            self.dirty_cache_roots.remove(&node);
            self.dirty_cache_root_reasons.remove(&node);
        }
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

        // Keep parent pointers consistent even when the child list is unchanged.
        //
        // This matters for view-cache reuse and GC/repair flows where a node may be temporarily
        // detached and then re-attached without changing the parent's child list. Invalidation
        // propagation relies on `parent` pointers even when semantics/debug traversals use the
        // child lists.
        let same_children = self
            .nodes
            .get(parent)
            .is_some_and(|n| n.children.as_slice() == children.as_slice());
        if same_children {
            self.repair_same_children_parent_pointers_and_reconnect_layout(parent, &children);
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

        let mut propagate = false;
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
            propagate = true;
        }
        if let Some((prev, next)) = counter_update {
            self.update_invalidation_counters(prev, next);
        }

        self.recompute_node_subtree_layout_dirty_count_and_propagate(parent);

        if propagate {
            // Structural changes must invalidate ancestors so the next layout pass walks far
            // enough to place newly mounted subtrees, even when view-cache invalidation
            // truncation is enabled.
            self.mark_invalidation_with_source(
                parent,
                Invalidation::HitTest,
                UiDebugInvalidationSource::Other,
            );
        }
    }

    pub(in crate::tree) fn repair_same_children_parent_pointers_and_reconnect_layout(
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

        self.recompute_node_subtree_layout_dirty_count_and_propagate(parent);

        if repaired_parent_pointer
            && self.subtree_has_pending_layout_work(parent)
            && self
                .nodes
                .get(parent)
                .is_some_and(|node| !node.invalidation.layout)
        {
            // Same-children writes are also used as a parent-pointer repair path during retained /
            // GC flows. If a descendant became layout-dirty while detached, reconnect the parent to
            // the authoritative layout invalidation walk so the next frame descends back into the
            // repaired subtree.
            self.mark_invalidation_with_source(
                parent,
                Invalidation::Layout,
                UiDebugInvalidationSource::Other,
            );
        }
    }

    pub(in crate::tree) fn subtree_has_pending_layout_work(&self, root: NodeId) -> bool {
        if self.subtree_layout_dirty_aggregation_enabled() {
            return self.node_subtree_layout_dirty(root);
        }

        let mut stack: Vec<NodeId> = vec![root];
        while let Some(node) = stack.pop() {
            let Some(entry) = self.nodes.get(node) else {
                continue;
            };
            if entry.invalidation.layout {
                return true;
            }
            stack.extend(entry.children.iter().copied());
        }
        false
    }
}
