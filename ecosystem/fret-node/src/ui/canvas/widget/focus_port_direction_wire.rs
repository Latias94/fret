use fret_ui::UiHost;

use crate::core::PortDirection;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, WireDragKind};

pub(super) fn required_port_direction_from_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
) -> Option<PortDirection> {
    canvas.interaction.wire_drag.as_ref().and_then(|wire_drag| {
        let source_port = match &wire_drag.kind {
            WireDragKind::New { from, .. } => Some(*from),
            WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
            WireDragKind::ReconnectMany { edges } => edges.first().map(|edge| edge.2),
        }?;
        let source_dir = canvas
            .graph
            .read_ref(host, |graph| {
                graph.ports.get(&source_port).map(|port| port.dir)
            })
            .ok()
            .flatten()?;
        Some(opposite_port_direction(source_dir))
    })
}

fn opposite_port_direction(direction: PortDirection) -> PortDirection {
    match direction {
        PortDirection::In => PortDirection::Out,
        PortDirection::Out => PortDirection::In,
    }
}

#[cfg(test)]
mod tests;
