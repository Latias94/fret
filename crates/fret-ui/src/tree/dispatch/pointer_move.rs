use super::*;
use std::collections::HashMap;

impl<H: UiHost> UiTree<H> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn dispatch_pointer_move_layer_observers(
        &mut self,
        app: &mut H,
        services: &mut dyn UiServices,
        input_ctx: &InputContext,
        barrier_root: Option<NodeId>,
        event: &Event,
        needs_redraw: &mut bool,
        invalidation_visited: &mut HashMap<NodeId, u8>,
    ) {
        let Event::Pointer(PointerEvent::Move {
            pointer_id,
            pointer_type,
            ..
        }) = event
        else {
            return;
        };

        if !pointer_type_supports_hover(*pointer_type) {
            return;
        }

        let captured_layer_for_pointer_move = self
            .captured
            .get(pointer_id)
            .copied()
            .and_then(|n| self.node_layer(n));
        let pointer_move_occlusion_layer = captured_layer_for_pointer_move
            .is_none()
            .then(|| self.topmost_pointer_occlusion_layer(barrier_root))
            .flatten()
            .filter(|(_, occlusion)| *occlusion != PointerOcclusion::None)
            .map(|(layer, _)| layer);
        let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        let mut hit_barrier = false;
        for layer_id in layers.into_iter().rev() {
            let Some((layer_root, visible, wants_pointer_move_events)) = self
                .layers
                .get(layer_id)
                .map(|layer| (layer.root, layer.visible, layer.wants_pointer_move_events))
            else {
                continue;
            };
            if !visible {
                continue;
            }
            if barrier_root.is_some() && hit_barrier {
                break;
            }
            if !wants_pointer_move_events {
                if barrier_root == Some(layer_root) {
                    hit_barrier = true;
                }
                if pointer_move_occlusion_layer == Some(layer_id) {
                    break;
                }
                continue;
            }
            if captured_layer_for_pointer_move.is_some_and(|layer| layer != layer_id) {
                // Pointer-move observer hooks are used by overlay policies (e.g. Radix menu safe
                // corridor). When a pointer is captured by a different layer (viewport tools,
                // docking drags, etc.), do not let unrelated overlay layers observe that move
                // stream. This keeps captured interactions stable and avoids cross-layer
                // arbitration fights during drags.
                if barrier_root == Some(layer_root) {
                    hit_barrier = true;
                }
                continue;
            }
            if self.dispatch_event_to_node_chain_observer(
                app,
                services,
                input_ctx,
                layer_root,
                event,
                invalidation_visited,
            ) {
                *needs_redraw = true;
            }
            if barrier_root == Some(layer_root) {
                hit_barrier = true;
            }
            if pointer_move_occlusion_layer == Some(layer_id) {
                break;
            }
        }
    }
}
