use super::super::*;

impl<H: UiHost> UiTree<H> {
    #[track_caller]
    pub(crate) fn set_children_in_mount(&mut self, parent: NodeId, children: Vec<NodeId>) {
        if self.nodes.get(parent).is_none() {
            return;
        }

        self.set_node_children_write_policy(parent, ChildrenWritePolicy::Standard);
        self.detach_reparented_children_from_old_parents(parent, &children);

        // Keep the direct retained parent edge consistent even when the child list is unchanged.
        let same_children = self
            .nodes
            .get(parent)
            .is_some_and(|n| n.children.as_slice() == children.as_slice());
        if same_children {
            self.replace_child_parent_index(parent, &children, &children);
            self.sync_same_children_parent_edges_and_reconnect_layout(parent, &children);
            if self.node_is_reachable_from_layer_forest(parent) {
                for &child in &children {
                    self.index_live_subtree(child);
                    self.sync_live_view_boundary_state_for_subtree(child);
                }
            }
            return;
        }

        let parent_is_layer_root = self.root_to_layer.contains_key(&parent);
        let skip_redundant_initial_mount_walk = self.nodes.get(parent).is_some_and(|n| {
            parent_is_layer_root
                && n.children.is_empty()
                && n.invalidation.layout
                && n.invalidation.paint
                && n.invalidation.hit_test
        });

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
        let parent_was_live = self.node_is_reachable_from_layer_forest(parent);

        self.replace_child_parent_index(parent, &old_children, &children);

        for old in old_children {
            if children.contains(&old) {
                continue;
            }
            if parent_was_live {
                self.unindex_detached_child_subtree(old);
                self.detach_view_boundary_state_for_subtree(old);
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
            if skip_redundant_initial_mount_walk {
                self.bump_command_availability_revision();
                self.mark_semantics_dirty();
            } else {
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
    }
}
