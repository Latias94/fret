use super::super::*;

pub(super) fn emit_view_callbacks<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    changes: &[ViewChange],
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    if changes.is_empty() {
        return;
    }

    callbacks.on_view_change(changes);
    for change in changes {
        match change {
            ViewChange::Viewport { pan, zoom } => {
                callbacks.on_viewport_change(*pan, *zoom);
                callbacks.on_move(*pan, *zoom);
            }
            ViewChange::Selection {
                nodes,
                edges,
                groups,
            } => callbacks.on_selection_change(crate::runtime::callbacks::SelectionChange {
                nodes: nodes.clone(),
                edges: edges.clone(),
                groups: groups.clone(),
            }),
        }
    }
}
