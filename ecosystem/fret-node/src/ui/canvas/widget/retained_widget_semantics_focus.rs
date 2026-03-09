use super::*;

pub(super) fn active_descendant<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &SemanticsCx<'_, H>,
) -> Option<fret_core::NodeId> {
    match (
        canvas.interaction.focused_port.is_some(),
        canvas.interaction.focused_edge.is_some(),
        canvas.interaction.focused_node.is_some(),
    ) {
        (true, _, _) => cx.children.first().copied(),
        (false, true, _) => cx.children.get(1).copied(),
        (false, false, true) => cx.children.get(2).copied(),
        _ => None,
    }
}
