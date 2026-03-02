use super::super::*;
use super::{UiLayer, UiLayerId};

impl<H: UiHost> UiTree<H> {
    /// Returns the current UI layer order in paint order (back-to-front).
    ///
    /// This includes the base layer and any overlay layers (even if currently invisible).
    pub fn layer_ids_in_paint_order(&self) -> &[UiLayerId] {
        self.layer_order.as_slice()
    }

    /// Reorders layers in paint order (back-to-front).
    ///
    /// This is a mechanism-only API intended for component-layer overlay orchestration. Policy
    /// code should treat this as "stable z-order correction" rather than a per-component knob.
    ///
    /// Notes:
    /// - The base layer (when present) is always kept at the back (index 0).
    /// - Unknown/missing layer IDs are ignored, and missing existing layers are appended in their
    ///   previous relative order.
    pub fn reorder_layers_in_paint_order(&mut self, desired: Vec<UiLayerId>) {
        if self.layer_order.is_empty() {
            return;
        }

        let mut seen = std::collections::HashSet::<UiLayerId>::new();
        let mut next: Vec<UiLayerId> = Vec::with_capacity(self.layer_order.len());

        for id in desired {
            if !self.layers.contains_key(id) {
                continue;
            }
            if !seen.insert(id) {
                continue;
            }
            next.push(id);
        }

        // Preserve any layers not mentioned by the caller in their existing relative order.
        for &id in &self.layer_order {
            if !self.layers.contains_key(id) {
                continue;
            }
            if seen.insert(id) {
                next.push(id);
            }
        }

        if let Some(base) = self.base_layer {
            next.retain(|&id| id != base);
            next.insert(0, base);
        }

        if next == self.layer_order {
            return;
        }

        self.layer_order = next;

        // Layer order changes can move the active modal/focus barriers. Ensure focus/capture do
        // not remain under a barrier after reordering.
        let (active_roots, barrier_root) = self.active_input_layers();
        if barrier_root.is_some() {
            self.enforce_modal_barrier_scope(&active_roots);
        }
        let (active_focus_roots, focus_barrier_root) = self.active_focus_layers();
        if focus_barrier_root.is_some() {
            self.enforce_focus_barrier_scope(&active_focus_roots);
        }
    }

    pub(in crate::tree) fn enforce_modal_barrier_scope(&mut self, active_roots: &[NodeId]) {
        let (focus_roots, focus_barrier_root) = self.active_focus_layers();
        if focus_barrier_root.is_some()
            && self.focus.is_some_and(|n| {
                !self.is_reachable_from_any_root_via_children(n, focus_roots.as_slice())
            })
        {
            self.set_focus_unchecked(None, "layers: enforce modal barrier scope");
        }
        let to_remove: Vec<PointerId> = self
            .captured
            .iter()
            .filter_map(|(p, n)| {
                (!self.is_reachable_from_any_root_via_children(*n, active_roots)).then_some(*p)
            })
            .collect();
        for p in to_remove {
            self.captured.remove(&p);
        }
    }

    pub(in crate::tree) fn enforce_focus_barrier_scope(&mut self, active_roots: &[NodeId]) {
        if self
            .focus
            .is_some_and(|n| !self.is_reachable_from_any_root_via_children(n, active_roots))
        {
            self.set_focus_unchecked(None, "layers: enforce focus barrier scope");
        }
    }

    pub fn base_root(&self) -> Option<NodeId> {
        self.base_layer
            .and_then(|id| self.layers.get(id).map(|l| l.root))
    }

    pub fn set_base_root(&mut self, root: NodeId) -> UiLayerId {
        if let Some(id) = self.base_layer {
            self.update_layer_root(id, root);
            return id;
        }

        let id = self.layers.insert(UiLayer {
            root,
            visible: true,
            blocks_underlay_input: false,
            blocks_underlay_focus: false,
            hit_testable: true,
            pointer_occlusion: PointerOcclusion::None,
            wants_pointer_down_outside_events: false,
            consume_pointer_down_outside_events: false,
            pointer_down_outside_branches: Vec::new(),
            scroll_dismiss_elements: Vec::new(),
            wants_pointer_move_events: false,
            wants_timer_events: true,
        });
        self.root_to_layer.insert(root, id);
        self.layer_order.insert(0, id);
        self.base_layer = Some(id);
        id
    }

    pub fn push_overlay_root(&mut self, root: NodeId, blocks_underlay_input: bool) -> UiLayerId {
        self.push_overlay_root_ex(root, blocks_underlay_input, true)
    }

    pub fn push_overlay_root_ex(
        &mut self,
        root: NodeId,
        blocks_underlay_input: bool,
        hit_testable: bool,
    ) -> UiLayerId {
        let id = self.layers.insert(UiLayer {
            root,
            visible: true,
            blocks_underlay_input,
            blocks_underlay_focus: blocks_underlay_input,
            hit_testable,
            pointer_occlusion: PointerOcclusion::None,
            wants_pointer_down_outside_events: false,
            consume_pointer_down_outside_events: false,
            pointer_down_outside_branches: Vec::new(),
            scroll_dismiss_elements: Vec::new(),
            wants_pointer_move_events: false,
            wants_timer_events: false,
        });
        self.root_to_layer.insert(root, id);
        self.layer_order.push(id);

        if blocks_underlay_input {
            let (active_roots, _barrier_root) = self.active_input_layers();
            self.enforce_modal_barrier_scope(&active_roots);
        }

        id
    }

    /// Uninstalls an overlay layer and removes its root subtree.
    ///
    /// This is the symmetric operation to `push_overlay_root(_ex)` and exists to keep the overlay
    /// substrate contract minimal but complete (ADR 0066).
    ///
    /// Notes:
    /// - The base layer cannot be removed (use `set_base_root` instead).
    /// - This removes the layer root node, and recursively removes its children **unless** a child
    ///   subtree is itself a layer root (which is treated as an independent root).
    pub fn remove_layer(
        &mut self,
        services: &mut dyn UiServices,
        layer: UiLayerId,
    ) -> Option<NodeId> {
        if self.base_layer == Some(layer) {
            return None;
        }
        let root = self.layers.get(layer).map(|l| l.root)?;

        // Make the root removable by the existing subtree removal logic (which normally refuses to
        // delete layer roots).
        self.root_to_layer.remove(&root);

        self.layer_order.retain(|&id| id != layer);
        let _ = self.layers.remove(layer);

        let mut removed: Vec<NodeId> = Vec::new();
        self.remove_subtree_inner(services, root, &mut removed);

        Some(root)
    }

    #[track_caller]
    pub fn set_layer_visible(&mut self, layer: UiLayerId, visible: bool) {
        let prev_visible = self.layers.get(layer).map(|l| l.visible);
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.visible = visible;

        if !visible {
            let to_remove: Vec<fret_core::PointerId> = self
                .captured
                .iter()
                .filter_map(|(p, n)| {
                    (self.node_layer(*n).is_some_and(|lid| lid == layer)).then_some(*p)
                })
                .collect();
            for p in to_remove {
                self.captured.remove(&p);
            }
            if self
                .focus
                .is_some_and(|n| self.node_layer(n).is_some_and(|lid| lid == layer))
            {
                self.set_focus_unchecked(None, "layers: set_layer_visible(false)");
            }
        }

        // When visibility changes, the active modal barrier can appear/disappear or move. Ensure
        // focus/capture do not remain in layers that are now under the barrier (or otherwise
        // inactive).
        //
        // This is especially important for overlay managers that reuse layer roots and toggle
        // visibility instead of creating/removing roots each time (fearless refactors should keep
        // the behavior consistent).
        if prev_visible != Some(visible) {
            let (active_roots, barrier_root) = self.active_input_layers();
            if barrier_root.is_some() {
                self.enforce_modal_barrier_scope(&active_roots);
            }

            #[cfg(feature = "diagnostics")]
            if self.debug_enabled {
                let caller = std::panic::Location::caller();
                self.debug_layer_visible_writes
                    .push(UiDebugSetLayerVisibleWrite {
                        layer,
                        frame_id: self.debug_stats.frame_id,
                        prev_visible,
                        visible,
                        file: caller.file(),
                        line: caller.line(),
                        column: caller.column(),
                    });
            }
        }
    }

    pub fn set_layer_hit_testable(&mut self, layer: UiLayerId, hit_testable: bool) {
        let prev_hit_testable = self.layers.get(layer).map(|l| l.hit_testable);
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.hit_testable = hit_testable;

        if !hit_testable {
            let to_remove: Vec<fret_core::PointerId> = self
                .captured
                .iter()
                .filter_map(|(p, n)| {
                    (self.node_layer(*n).is_some_and(|lid| lid == layer)).then_some(*p)
                })
                .collect();
            for p in to_remove {
                self.captured.remove(&p);
            }
            if self
                .focus
                .is_some_and(|n| self.node_layer(n).is_some_and(|lid| lid == layer))
            {
                self.set_focus_unchecked(None, "layers: set_layer_hit_testable(false)");
            }
        }

        if prev_hit_testable != Some(hit_testable) {
            let (active_roots, barrier_root) = self.active_input_layers();
            if barrier_root.is_some() {
                self.enforce_modal_barrier_scope(&active_roots);
            }
        }
    }

    pub fn set_layer_pointer_occlusion(&mut self, layer: UiLayerId, occlusion: PointerOcclusion) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.pointer_occlusion = occlusion;
    }

    pub fn set_layer_blocks_underlay_focus(&mut self, layer: UiLayerId, blocks: bool) {
        let prev = self.layers.get(layer).map(|l| l.blocks_underlay_focus);
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.blocks_underlay_focus = blocks;

        if prev != Some(blocks) {
            let (active_roots, barrier_root) = self.active_focus_layers();
            if barrier_root.is_some() {
                self.enforce_focus_barrier_scope(&active_roots);
            }
        }
    }

    pub fn is_layer_visible(&self, layer: UiLayerId) -> bool {
        self.layers.get(layer).is_some_and(|l| l.visible)
    }

    pub fn layer_root(&self, layer: UiLayerId) -> Option<NodeId> {
        self.layers.get(layer).map(|l| l.root)
    }

    pub(crate) fn all_layer_roots(&self) -> Vec<NodeId> {
        self.layer_order
            .iter()
            .filter_map(|layer| self.layers.get(*layer).map(|l| l.root))
            .collect()
    }

    pub fn set_layer_wants_pointer_move_events(&mut self, layer: UiLayerId, wants: bool) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.wants_pointer_move_events = wants;
    }

    pub fn set_layer_wants_pointer_down_outside_events(&mut self, layer: UiLayerId, wants: bool) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.wants_pointer_down_outside_events = wants;
    }

    pub fn set_layer_consume_pointer_down_outside_events(
        &mut self,
        layer: UiLayerId,
        consume: bool,
    ) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.consume_pointer_down_outside_events = consume;
    }

    pub fn set_layer_pointer_down_outside_branches(
        &mut self,
        layer: UiLayerId,
        branches: Vec<NodeId>,
    ) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.pointer_down_outside_branches = branches;
    }

    /// Register elements that should dismiss this overlay when a scroll event targets an ancestor
    /// of any element's current node.
    ///
    /// This is intended for Radix-aligned tooltip behavior: when the tooltip trigger is scrolled,
    /// the tooltip should close (Radix closes when `event.target.contains(trigger)` on scroll).
    pub fn set_layer_scroll_dismiss_elements(
        &mut self,
        layer: UiLayerId,
        elements: Vec<crate::GlobalElementId>,
    ) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.scroll_dismiss_elements = elements;
    }

    pub fn set_layer_wants_timer_events(&mut self, layer: UiLayerId, wants: bool) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };
        l.wants_timer_events = wants;
    }

    pub fn node_layer(&self, node: NodeId) -> Option<UiLayerId> {
        let root = self.node_root(node)?;
        self.root_to_layer.get(&root).copied()
    }

    pub(in crate::tree) fn visible_layers_in_paint_order(
        &self,
    ) -> impl Iterator<Item = UiLayerId> + '_ {
        self.layer_order
            .iter()
            .copied()
            .filter(|id| self.layers.get(*id).is_some_and(|l| l.visible))
    }

    pub(in crate::tree) fn topmost_pointer_occlusion_layer(
        &self,
        barrier_root: Option<NodeId>,
    ) -> Option<(UiLayerId, PointerOcclusion)> {
        let mut hit_barrier = false;
        for &layer_id in self.layer_order.iter().rev() {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }
            if barrier_root.is_some() && hit_barrier {
                break;
            }

            let occlusion = layer.pointer_occlusion;
            if occlusion != PointerOcclusion::None {
                return Some((layer_id, occlusion));
            }

            if barrier_root == Some(layer.root) {
                hit_barrier = true;
            }
        }
        None
    }

    pub(in crate::tree) fn active_input_layers(&self) -> (Vec<NodeId>, Option<NodeId>) {
        let mut any_visible = false;
        let mut barrier_root: Option<NodeId> = None;
        for &layer_id in &self.layer_order {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }
            any_visible = true;

            // Modal/pointer barriers can be hit-test-inert (e.g. close transitions, pointer-only
            // underlay blocking). A barrier must still gate input even when it isn't hit-testable.
            if layer.blocks_underlay_input {
                barrier_root = Some(layer.root);
            }
        }

        if !any_visible {
            return (Vec::new(), None);
        }

        let mut roots: Vec<NodeId> = Vec::new();
        for &layer_id in self.layer_order.iter().rev() {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }

            // Focus barriers can become active before a layer is hit-testable (e.g. during open/close
            // transitions). Include the barrier root itself so focus can still be moved into the
            // barrier scope; otherwise `set_focus` can reject all targets while the underlay is
            // blocked, leading to spurious focus loss.
            if layer.hit_testable || barrier_root == Some(layer.root) {
                roots.push(layer.root);
            }

            if barrier_root == Some(layer.root) {
                break;
            }
        }
        (roots, barrier_root)
    }

    pub(in crate::tree) fn active_pointer_down_outside_layer_roots(
        &self,
        barrier_root: Option<NodeId>,
    ) -> Vec<NodeId> {
        let mut any_visible = false;
        for &layer_id in &self.layer_order {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }
            any_visible = true;
            if layer.blocks_underlay_input {
                break;
            }
        }

        if !any_visible {
            return Vec::new();
        }

        let mut roots: Vec<NodeId> = Vec::new();
        for &layer_id in self.layer_order.iter().rev() {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }

            roots.push(layer.root);

            if barrier_root == Some(layer.root) {
                break;
            }
        }
        roots
    }

    pub(in crate::tree) fn active_focus_layers(&self) -> (Vec<NodeId>, Option<NodeId>) {
        let mut any_visible = false;
        let mut barrier_root: Option<NodeId> = None;
        for &layer_id in &self.layer_order {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }
            any_visible = true;

            if layer.blocks_underlay_focus {
                barrier_root = Some(layer.root);
            }
        }

        if !any_visible {
            return (Vec::new(), None);
        }

        let mut roots: Vec<NodeId> = Vec::new();
        for &layer_id in self.layer_order.iter().rev() {
            let Some(layer) = self.layers.get(layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }

            // Focus barriers can become active while the barrier layer is hit-test-inert (e.g.
            // open/close transitions or pointer-only underlay blocking). Include the barrier root
            // itself so focus can remain inside (and be moved within) the barrier scope.
            if layer.hit_testable || barrier_root == Some(layer.root) {
                roots.push(layer.root);
            }

            if barrier_root == Some(layer.root) {
                break;
            }
        }
        (roots, barrier_root)
    }

    fn update_layer_root(&mut self, layer: UiLayerId, root: NodeId) {
        let Some(l) = self.layers.get_mut(layer) else {
            return;
        };

        self.root_to_layer.remove(&l.root);
        l.root = root;
        self.root_to_layer.insert(root, layer);
    }
}
