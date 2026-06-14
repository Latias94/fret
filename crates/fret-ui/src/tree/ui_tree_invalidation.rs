use super::*;
use std::any::{Any, TypeId};

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn mark_node_invalidation_state(
        node: &mut Node<H>,
        inv: Invalidation,
        local: bool,
    ) {
        match inv {
            Invalidation::HitTestOnly => {
                if local && !node.invalidation.paint {
                    node.paint_invalidated_by_hit_test_only = true;
                } else if !local {
                    node.paint_invalidated_by_hit_test_only = false;
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

    pub(in crate::tree) fn bump_command_availability_revision(&mut self) {
        self.command_availability_revision = self.command_availability_revision.wrapping_add(1);
        self.focus_traversal_availability_cache = None;
        self.command_availability_interest_cache = None;
    }

    pub(in crate::tree) fn mark_invalidation_local_with_detail(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        detail: UiDebugInvalidationDetail,
    ) {
        self.mark_invalidation_local_with_source_and_detail(
            node,
            inv,
            UiDebugInvalidationSource::Other,
            detail,
        );
    }

    pub(in crate::tree) fn mark_invalidation_local_with_source_and_detail(
        &mut self,
        node: NodeId,
        inv: Invalidation,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        let node_exists = self.nodes.contains_key(node);
        if node_exists && Self::invalidation_may_affect_command_availability(source, inv, detail) {
            self.bump_command_availability_revision();
        }
        if node_exists && Self::invalidation_may_affect_semantics(source, inv, detail) {
            self.mark_semantics_dirty_for_node(node);
        }

        let (prev, next, layout_before, layout_after) = {
            let Some(n) = self.nodes.get_mut(node) else {
                return;
            };
            let prev = n.invalidation;
            let layout_before = n.invalidation.layout;
            Self::mark_node_invalidation_state(n, inv, true);
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
        if !layout_before && layout_after {
            self.debug_note_layout_dirty_source(node, node, source, detail);
        } else if layout_before && !layout_after {
            self.debug_clear_layout_dirty_source(node);
        }
        self.update_invalidation_counters(prev, next);
    }

    pub(in crate::tree) fn mark_subtree_invalidation_local_with_detail(
        &mut self,
        root: NodeId,
        inv: Invalidation,
        detail: UiDebugInvalidationDetail,
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
            self.mark_invalidation_local_with_detail(node, inv, detail);
            self.scratch_node_stack.extend(children);
        }
    }

    pub(in crate::tree) fn begin_prepaint_outputs_for_node(
        &mut self,
        node: NodeId,
        key: PaintCacheKey,
    ) {
        let Some(boundary) = self.ensure_view_boundary_state(node) else {
            return;
        };
        boundary.prepaint.begin_outputs(key);
    }

    pub(in crate::tree) fn begin_scene_fragment_for_node(
        &mut self,
        node: NodeId,
        key: PaintCacheKey,
    ) {
        let Some(boundary) = self.ensure_view_boundary_state(node) else {
            return;
        };
        boundary.scene_fragment.begin_fragment(key);
    }

    pub(crate) fn set_prepaint_output<T: Any>(&mut self, node: NodeId, value: T) {
        let Some(boundary) = self.ensure_view_boundary_state(node) else {
            return;
        };
        boundary.prepaint.set_output(value);
    }

    pub(crate) fn set_prepaint_output_box(
        &mut self,
        node: NodeId,
        ty: TypeId,
        value: Box<dyn Any>,
    ) {
        let Some(boundary) = self.ensure_view_boundary_state(node) else {
            return;
        };
        boundary.prepaint.set_output_box(ty, value);
    }

    pub(crate) fn prepaint_output<T: Any>(&self, node: NodeId) -> Option<&T> {
        self.view_boundaries.get(node)?.prepaint.output::<T>()
    }

    pub(crate) fn prepaint_output_mut<T: Any>(&mut self, node: NodeId) -> Option<&mut T> {
        self.view_boundaries
            .get_mut(node)?
            .prepaint
            .output_mut::<T>()
    }

    pub(crate) fn prepaint_output_any(&self, node: NodeId, ty: TypeId) -> Option<&dyn Any> {
        self.view_boundaries.get(node)?.prepaint.output_any(ty)
    }

    pub(crate) fn prepaint_output_any_mut(
        &mut self,
        node: NodeId,
        ty: TypeId,
    ) -> Option<&mut dyn Any> {
        self.view_boundaries
            .get_mut(node)?
            .prepaint
            .output_any_mut(ty)
    }

    pub(crate) fn set_scene_fragment<T: Any>(&mut self, node: NodeId, value: T) {
        let Some(boundary) = self.ensure_view_boundary_state(node) else {
            return;
        };
        boundary.scene_fragment.set_fragment(value);
    }

    pub(crate) fn set_scene_fragment_debug<T: crate::tree::BoundarySceneFragmentDebug>(
        &mut self,
        node: NodeId,
        value: T,
    ) {
        let entry_count = value.boundary_scene_fragment_entry_count();
        let Some(boundary) = self.ensure_view_boundary_state(node) else {
            return;
        };
        boundary
            .scene_fragment
            .set_fragment_with_entry_count(value, entry_count);
    }

    pub(crate) fn set_scene_fragment_box(&mut self, node: NodeId, ty: TypeId, value: Box<dyn Any>) {
        let Some(boundary) = self.ensure_view_boundary_state(node) else {
            return;
        };
        boundary.scene_fragment.set_fragment_box(ty, value);
    }

    pub(crate) fn set_scene_fragment_box_with_entry_count(
        &mut self,
        node: NodeId,
        ty: TypeId,
        value: Box<dyn Any>,
        entry_count: usize,
    ) {
        let Some(boundary) = self.ensure_view_boundary_state(node) else {
            return;
        };
        boundary
            .scene_fragment
            .set_fragment_box_with_entry_count(ty, value, entry_count);
    }

    pub(crate) fn scene_fragment<T: Any>(&self, node: NodeId) -> Option<&T> {
        self.view_boundaries
            .get(node)?
            .scene_fragment
            .fragment::<T>()
    }

    pub(crate) fn scene_fragment_mut<T: Any>(&mut self, node: NodeId) -> Option<&mut T> {
        self.view_boundaries
            .get_mut(node)?
            .scene_fragment
            .fragment_mut::<T>()
    }

    pub(crate) fn scene_fragment_any(&self, node: NodeId, ty: TypeId) -> Option<&dyn Any> {
        self.view_boundaries
            .get(node)?
            .scene_fragment
            .fragment_any(ty)
    }

    pub(crate) fn scene_fragment_any_mut(
        &mut self,
        node: NodeId,
        ty: TypeId,
    ) -> Option<&mut dyn Any> {
        self.view_boundaries
            .get_mut(node)?
            .scene_fragment
            .fragment_any_mut(ty)
    }

    pub(crate) fn record_scene_fragment_used_entries(&mut self, node: NodeId, count: usize) {
        let Some(boundary) = self.view_boundaries.get_mut(node) else {
            return;
        };
        boundary.scene_fragment.record_used_entries(count);
    }

    pub(crate) fn record_scene_fragment_rejected_entries(
        &mut self,
        node: NodeId,
        count: usize,
        reason: &'static str,
    ) {
        let Some(boundary) = self.view_boundaries.get_mut(node) else {
            return;
        };
        boundary
            .scene_fragment
            .record_rejected_entries(count, reason);
    }
}
