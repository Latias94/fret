use super::super::*;

impl<H: UiHost> UiTree<H> {
    #[track_caller]
    pub(crate) fn set_children_in_mount(&mut self, parent: NodeId, children: Vec<NodeId>) {
        if self.nodes.get(parent).is_none() {
            return;
        }

        self.set_node_children_write_policy(parent, ChildrenWritePolicy::Standard);
        self.detach_reparented_children_from_old_parents(parent, &children);

        // Keep parent pointers consistent even when the child list is unchanged.
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
}
