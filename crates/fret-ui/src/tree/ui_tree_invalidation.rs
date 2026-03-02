use super::*;
use std::any::Any;

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn mark_node_invalidation_state(node: &mut Node<H>, inv: Invalidation) {
        match inv {
            Invalidation::HitTestOnly => {
                if !node.invalidation.paint {
                    node.paint_invalidated_by_hit_test_only = true;
                }
            }
            Invalidation::Paint | Invalidation::Layout | Invalidation::HitTest => {
                node.paint_invalidated_by_hit_test_only = false;
            }
        }
        node.invalidation.mark(inv);
    }

    pub(in crate::tree) fn update_invalidation_counters(
        &mut self,
        prev: InvalidationFlags,
        next: InvalidationFlags,
    ) {
        if prev.layout != next.layout {
            if next.layout {
                self.invalidated_layout_nodes = self.invalidated_layout_nodes.saturating_add(1);
            } else {
                self.invalidated_layout_nodes = self.invalidated_layout_nodes.saturating_sub(1);
            }
        }
        if prev.paint != next.paint {
            if next.paint {
                self.invalidated_paint_nodes = self.invalidated_paint_nodes.saturating_add(1);
            } else {
                self.invalidated_paint_nodes = self.invalidated_paint_nodes.saturating_sub(1);
            }
        }
        if prev.hit_test != next.hit_test {
            if next.hit_test {
                self.invalidated_hit_test_nodes = self.invalidated_hit_test_nodes.saturating_add(1);
            } else {
                self.invalidated_hit_test_nodes = self.invalidated_hit_test_nodes.saturating_sub(1);
            }
        }
    }

    pub(in crate::tree) fn mark_invalidation_local(&mut self, node: NodeId, inv: Invalidation) {
        let (prev, next, layout_before, layout_after) = {
            let Some(n) = self.nodes.get_mut(node) else {
                return;
            };
            let prev = n.invalidation;
            let layout_before = n.invalidation.layout;
            Self::mark_node_invalidation_state(n, inv);
            let next = n.invalidation;
            let layout_after = n.invalidation.layout;
            (prev, next, layout_before, layout_after)
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
        self.update_invalidation_counters(prev, next);
    }

    pub(in crate::tree) fn recompute_node_subtree_layout_dirty_count_and_propagate(
        &mut self,
        node: NodeId,
    ) {
        // Compatibility shim: older retained-tree code maintained per-node subtree "layout dirty"
        // counters. The current pipeline relies on global invalidation counters plus targeted
        // passes (pending barrier relayouts, view-cache contained relayouts).
        //
        // Keep a conservative view-cache mark so contained cache roots stay discoverable via the
        // `dirty_cache_roots` set when callers toggle layout invalidation without an invalidation
        // walk.
        if !self.view_cache_active() {
            return;
        }
        let Some(root) = self.nearest_view_cache_root(node) else {
            return;
        };
        let Some(n) = self.nodes.get(root) else {
            return;
        };
        if n.view_cache.enabled && n.view_cache.contained_layout && n.invalidation.layout {
            self.mark_cache_root_dirty(
                root,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::Unknown,
            );
        }
    }

    pub(in crate::tree) fn note_layout_invalidation_transition_for_subtree_aggregation(
        &mut self,
        node: NodeId,
        before: bool,
        after: bool,
    ) {
        // Compatibility shim: see `recompute_node_subtree_layout_dirty_count_and_propagate`.
        if before == after {
            return;
        }
        if !after {
            return;
        }
        if !self.view_cache_active() {
            return;
        }
        let Some(n) = self.nodes.get(node) else {
            return;
        };
        if n.view_cache.enabled && n.view_cache.contained_layout {
            self.mark_cache_root_dirty(
                node,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::Unknown,
            );
        }
    }

    pub(in crate::tree) fn begin_prepaint_outputs_for_node(
        &mut self,
        node: NodeId,
        key: PaintCacheKey,
    ) {
        let Some(n) = self.nodes.get_mut(node) else {
            return;
        };
        n.prepaint_outputs.begin_frame(key);
    }

    pub(crate) fn set_prepaint_output<T: Any>(&mut self, node: NodeId, value: T) {
        let Some(n) = self.nodes.get_mut(node) else {
            return;
        };
        n.prepaint_outputs.set(value);
    }

    pub(crate) fn prepaint_output<T: Any>(&self, node: NodeId) -> Option<&T> {
        self.nodes.get(node)?.prepaint_outputs.get::<T>()
    }

    pub(crate) fn prepaint_output_mut<T: Any>(&mut self, node: NodeId) -> Option<&mut T> {
        self.nodes.get_mut(node)?.prepaint_outputs.get_mut::<T>()
    }
}
