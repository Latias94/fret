use super::*;

impl<H: UiHost> UiTree<H> {
    pub fn captured_for(&self, pointer_id: PointerId) -> Option<NodeId> {
        self.captured.get(&pointer_id).copied()
    }

    pub fn captured(&self) -> Option<NodeId> {
        self.captured_for(PointerId(0))
    }

    pub fn any_captured_node(&self) -> Option<NodeId> {
        self.captured.values().copied().next()
    }

    pub fn input_arbitration_snapshot(&self) -> UiInputArbitrationSnapshot {
        let (_active, barrier_root) = self.active_input_layers();
        let (_focus_active, focus_barrier_root) = self.active_focus_layers();

        let (pointer_occlusion_layer, pointer_occlusion) = self
            .topmost_pointer_occlusion_layer(barrier_root)
            .map(|(layer, occlusion)| (Some(layer), occlusion))
            .unwrap_or((None, PointerOcclusion::None));

        let mut pointer_capture_active = false;
        let mut pointer_capture_layer: Option<UiLayerId> = None;
        let mut pointer_capture_multiple_layers = false;
        for &node in self.captured.values() {
            pointer_capture_active = true;
            let Some(layer) = self.node_layer(node) else {
                pointer_capture_layer = None;
                pointer_capture_multiple_layers = true;
                break;
            };

            match pointer_capture_layer {
                None => pointer_capture_layer = Some(layer),
                Some(prev) => {
                    if prev != layer {
                        pointer_capture_layer = None;
                        pointer_capture_multiple_layers = true;
                        break;
                    }
                }
            }
        }

        UiInputArbitrationSnapshot {
            modal_barrier_root: barrier_root,
            focus_barrier_root,
            pointer_occlusion,
            pointer_occlusion_layer,
            pointer_capture_active,
            pointer_capture_layer,
            pointer_capture_multiple_layers,
        }
    }

    pub(super) fn window_input_arbitration_snapshot(
        &self,
    ) -> fret_runtime::WindowInputArbitrationSnapshot {
        let snapshot = self.input_arbitration_snapshot();
        fret_runtime::WindowInputArbitrationSnapshot {
            modal_barrier_root: snapshot.modal_barrier_root,
            focus_barrier_root: snapshot.focus_barrier_root,
            pointer_occlusion: match snapshot.pointer_occlusion {
                PointerOcclusion::None => fret_runtime::WindowPointerOcclusion::None,
                PointerOcclusion::BlockMouse => fret_runtime::WindowPointerOcclusion::BlockMouse,
                PointerOcclusion::BlockMouseExceptScroll => {
                    fret_runtime::WindowPointerOcclusion::BlockMouseExceptScroll
                }
            },
            pointer_occlusion_root: snapshot
                .pointer_occlusion_layer
                .and_then(|layer| self.layers.get(layer).map(|l| l.root)),
            pointer_capture_active: snapshot.pointer_capture_active,
            pointer_capture_root: snapshot
                .pointer_capture_layer
                .and_then(|layer| self.layers.get(layer).map(|l| l.root)),
            pointer_capture_multiple_roots: snapshot.pointer_capture_multiple_layers
                || (snapshot.pointer_capture_active && snapshot.pointer_capture_layer.is_none()),
        }
    }
}
