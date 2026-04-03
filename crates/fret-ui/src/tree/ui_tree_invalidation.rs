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

    pub(in crate::tree) fn mark_subtree_invalidation_local(
        &mut self,
        root: NodeId,
        inv: Invalidation,
    ) {
        if !self.nodes.contains_key(root) {
            return;
        }

        self.scratch_node_stack.clear();
        self.scratch_node_stack.push(root);
        while let Some(node) = self.scratch_node_stack.pop() {
            let children: Vec<NodeId> = self
                .nodes
                .get(node)
                .map(|entry| entry.children.to_vec())
                .unwrap_or_default();
            self.mark_invalidation_local(node, inv);
            self.scratch_node_stack.extend(children);
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
