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
        let Some(n) = self.nodes.get_mut(node) else {
            return;
        };
        let layout_before = n.invalidation.layout;
        n.invalidation.clear();
        n.paint_invalidated_by_hit_test_only = false;
        record_layout_invalidation_transition(
            &mut self.layout_invalidations_count,
            layout_before,
            n.invalidation.layout,
        );
    }

    #[cfg(test)]
    pub(crate) fn test_set_layout_invalidation(&mut self, node: NodeId, value: bool) {
        let Some(n) = self.nodes.get_mut(node) else {
            return;
        };
        let layout_before = n.invalidation.layout;
        n.invalidation.layout = value;
        if value {
            n.invalidation.paint = true;
        }
        let should_mark_contained_cache_root_dirty = value
            && self.view_cache_active()
            && n.view_cache.enabled
            && n.view_cache.contained_layout;
        record_layout_invalidation_transition(
            &mut self.layout_invalidations_count,
            layout_before,
            n.invalidation.layout,
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

    pub fn set_root(&mut self, root: NodeId) {
        let _ = self.set_base_root(root);
    }

    pub fn add_child(&mut self, parent: NodeId, child: NodeId) {
        if let Some(node) = self.nodes.get_mut(child) {
            node.parent = Some(parent);
        }
        if let Some(node) = self.nodes.get_mut(parent) {
            node.children.push(child);
            node.invalidation.hit_test = true;
            if !node.invalidation.layout {
                self.layout_invalidations_count = self.layout_invalidations_count.saturating_add(1);
            }
            node.invalidation.layout = true;
            node.invalidation.paint = true;
        }
        self.mark_invalidation_local(parent, Invalidation::HitTest);
    }

    #[track_caller]
    pub fn set_children(&mut self, parent: NodeId, children: Vec<NodeId>) {
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
            for &child in &children {
                if let Some(n) = self.nodes.get_mut(child) {
                    n.parent = Some(parent);
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
}
