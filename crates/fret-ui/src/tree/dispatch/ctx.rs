use super::*;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(in crate::tree) struct DispatchCx {
    pub(in crate::tree) frame_id: FrameId,
    pub(in crate::tree) window: Option<AppWindowId>,

    pub(in crate::tree) active_input_roots: Vec<NodeId>,
    pub(in crate::tree) input_barrier_root: Option<NodeId>,

    pub(in crate::tree) active_focus_roots: Vec<NodeId>,
    pub(in crate::tree) focus_barrier_root: Option<NodeId>,

    /// Combined barrier root used for focus-layer snapshotting and command availability gating.
    ///
    /// A focus barrier can be active even when there is no modal/input barrier (e.g. a focus trap
    /// overlay that does not block pointer events). Conversely, modal barriers typically also act
    /// as focus barriers, so we keep this value as `focus_barrier_root.or(input_barrier_root)`.
    pub(in crate::tree) barrier_root: Option<NodeId>,

    pub(in crate::tree) input_snapshot: UiDispatchSnapshot,
    pub(in crate::tree) focus_snapshot: UiDispatchSnapshot,
}

impl DispatchCx {
    pub(in crate::tree::dispatch) fn node_in_active_input_layers(&self, node: NodeId) -> bool {
        self.input_snapshot.pre.get(node).is_some()
    }

    pub(in crate::tree::dispatch) fn node_in_active_focus_layers(&self, node: NodeId) -> bool {
        self.focus_snapshot.pre.get(node).is_some()
    }
}

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree::dispatch) fn build_dispatch_cx(
        &self,
        frame_id: FrameId,
        active_input_roots: Vec<NodeId>,
        input_barrier_root: Option<NodeId>,
    ) -> DispatchCx {
        let (active_focus_roots, focus_barrier_root) = self.active_focus_layers();
        let active_focus_roots = active_focus_roots;
        let barrier_root = focus_barrier_root.or(input_barrier_root);

        let input_snapshot = self.build_dispatch_snapshot_for_layer_roots(
            frame_id,
            active_input_roots.as_slice(),
            input_barrier_root,
        );

        let focus_snapshot =
            if active_focus_roots == active_input_roots && barrier_root == input_barrier_root {
                let mut snapshot = input_snapshot.clone();
                snapshot.active_layer_roots = active_focus_roots.clone();
                snapshot.barrier_root = barrier_root;
                snapshot
            } else {
                self.build_dispatch_snapshot_for_layer_roots(
                    frame_id,
                    active_focus_roots.as_slice(),
                    barrier_root,
                )
            };

        DispatchCx {
            frame_id,
            window: self.window,
            active_input_roots,
            input_barrier_root,
            active_focus_roots,
            focus_barrier_root,
            barrier_root,
            input_snapshot,
            focus_snapshot,
        }
    }
}
