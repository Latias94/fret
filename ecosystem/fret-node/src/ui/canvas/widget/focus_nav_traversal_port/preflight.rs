use crate::ui::canvas::widget::*;

pub(super) struct PortTraversalInput {
    pub focused_node: GraphNodeId,
    pub wire_dir: Option<PortDirection>,
}

pub(super) fn traversal_input<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) -> Option<PortTraversalInput> {
    let snapshot = canvas.sync_view_state(host);
    if !snapshot.interaction.elements_selectable {
        return None;
    }

    let focused_node = canvas
        .interaction
        .focused_node
        .or_else(|| snapshot.selected_nodes.first().copied())
        .or_else(|| {
            canvas
                .graph
                .read_ref(host, |g| g.nodes.keys().next().copied())
                .ok()
                .flatten()
        })?;

    let wire_dir = canvas.interaction.wire_drag.as_ref().and_then(|w| {
        let from_port = match &w.kind {
            WireDragKind::New { from, .. } => Some(*from),
            WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
            WireDragKind::ReconnectMany { edges } => edges.first().map(|e| e.2),
        }?;
        canvas
            .graph
            .read_ref(host, |g| g.ports.get(&from_port).map(|p| p.dir))
            .ok()
            .flatten()
    });

    Some(PortTraversalInput {
        focused_node,
        wire_dir,
    })
}
