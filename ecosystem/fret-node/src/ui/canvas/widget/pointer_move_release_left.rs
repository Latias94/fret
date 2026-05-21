use super::*;

pub(super) fn handle_missing_left_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: pointer_up_cx::PointerUpCx<H, M>,
{
    let has_left_interaction = canvas.interaction.pending_marquee.is_some()
        || canvas.interaction.marquee.is_some()
        || canvas.interaction.pending_node_drag.is_some()
        || canvas.interaction.node_drag.is_some()
        || canvas.interaction.pending_group_drag.is_some()
        || canvas.interaction.group_drag.is_some()
        || canvas.interaction.pending_group_resize.is_some()
        || canvas.interaction.group_resize.is_some()
        || canvas.interaction.pending_node_resize.is_some()
        || canvas.interaction.node_resize.is_some()
        || canvas.interaction.pending_wire_drag.is_some()
        || canvas.interaction.wire_drag.is_some()
        || canvas.interaction.pending_edge_insert_drag.is_some()
        || canvas.interaction.edge_insert_drag.is_some()
        || canvas.interaction.edge_drag.is_some();

    if !has_left_interaction || buttons.left {
        return false;
    }

    let snapshot = canvas.sync_view_state(pointer_up_release_cx::PointerUpReleaseCx::host(cx));
    let _ = pointer_up::handle_pointer_up(
        canvas,
        cx,
        &snapshot,
        position,
        fret_core::MouseButton::Left,
        1,
        modifiers,
        snapshot.zoom,
    );
    true
}
