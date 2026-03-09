use super::*;

pub(super) fn focus_next_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    forward: bool,
) -> bool {
    let snapshot = canvas.sync_view_state(host);
    if !snapshot.interaction.elements_selectable {
        return false;
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
        });

    let Some(focused_node) = focused_node else {
        return false;
    };

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

    let ports = canvas
        .graph
        .read_ref(host, |g| {
            let (inputs, outputs) = node_ports(g, focused_node);
            let mut ports = Vec::with_capacity(inputs.len() + outputs.len());
            ports.extend(inputs);
            ports.extend(outputs);

            if let Some(wire_dir) = wire_dir {
                let want = match wire_dir {
                    PortDirection::In => PortDirection::Out,
                    PortDirection::Out => PortDirection::In,
                };
                ports.retain(|id| g.ports.get(id).is_some_and(|p| p.dir == want));
            }

            ports
        })
        .ok()
        .unwrap_or_default();

    if ports.is_empty() {
        return false;
    }

    let current = canvas
        .interaction
        .focused_port
        .filter(|id| ports.iter().any(|p| *p == *id));

    let next = match current.and_then(|id| ports.iter().position(|p| *p == id)) {
        Some(ix) => {
            let len = ports.len();
            let next_ix = if forward {
                (ix + 1) % len
            } else {
                (ix + len - 1) % len
            };
            ports[next_ix]
        }
        None => {
            if forward {
                ports[0]
            } else {
                ports[ports.len() - 1]
            }
        }
    };

    super::focus_session::focus_port(&mut canvas.interaction, focused_node, next);
    canvas.refresh_focused_port_hints(host);
    canvas.update_view_state(host, |s| {
        super::focus_session::select_only_node(s, focused_node, false);
    });
    true
}
