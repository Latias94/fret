use crate::ui::canvas::widget::*;

pub(super) fn traversal_snapshot<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) -> Option<ViewSnapshot> {
    let snapshot = canvas.sync_view_state(host);
    if !snapshot.interaction.elements_selectable {
        return None;
    }

    Some(snapshot)
}

pub(super) fn current_node(
    canvas: &NodeGraphCanvasWith<impl NodeGraphCanvasMiddleware>,
    snapshot: &ViewSnapshot,
) -> Option<GraphNodeId> {
    canvas
        .interaction
        .focused_node
        .or_else(|| snapshot.selected_nodes.first().copied())
}
