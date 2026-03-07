use std::collections::HashSet;

use super::*;

pub(super) fn focus_next_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    forward: bool,
) -> bool {
    let snapshot = canvas.sync_view_state(host);
    if !snapshot.interaction.elements_selectable
        || !snapshot.interaction.edges_selectable
        || !snapshot.interaction.edges_focusable
    {
        return false;
    }

    let mut edges: Vec<EdgeId> = canvas
        .graph
        .read_ref(host, |g| {
            g.edges
                .keys()
                .copied()
                .filter(|id| {
                    NodeGraphCanvasWith::<M>::edge_is_selectable(g, &snapshot.interaction, *id)
                })
                .collect()
        })
        .ok()
        .unwrap_or_default();
    if edges.is_empty() {
        return false;
    }
    edges.sort_unstable();

    let current = canvas
        .interaction
        .focused_edge
        .or_else(|| snapshot.selected_edges.first().copied());

    let next = match current.and_then(|id| edges.iter().position(|e| *e == id)) {
        Some(ix) => {
            let len = edges.len();
            let next_ix = if forward {
                (ix + 1) % len
            } else {
                (ix + len - 1) % len
            };
            edges[next_ix]
        }
        None => {
            if forward {
                edges[0]
            } else {
                edges[edges.len() - 1]
            }
        }
    };

    canvas.interaction.focused_edge = Some(next);
    canvas.interaction.focused_node = None;
    canvas.interaction.focused_port = None;
    canvas.interaction.focused_port_valid = false;
    canvas.interaction.focused_port_convertible = false;
    canvas.update_view_state(host, |s| {
        s.selected_nodes.clear();
        s.selected_groups.clear();
        s.selected_edges.clear();
        s.selected_edges.push(next);
    });
    true
}

pub(super) fn focus_next_node<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    forward: bool,
) -> bool {
    let snapshot = canvas.sync_view_state(host);
    if !snapshot.interaction.elements_selectable {
        return false;
    }

    let ordered: Vec<GraphNodeId> = canvas
        .graph
        .read_ref(host, |g| {
            let mut out: Vec<GraphNodeId> = Vec::new();
            let mut used: HashSet<GraphNodeId> = HashSet::new();

            for id in &snapshot.draw_order {
                if NodeGraphCanvasWith::<M>::node_is_selectable(g, &snapshot.interaction, *id)
                    && used.insert(*id)
                {
                    out.push(*id);
                }
            }

            let mut rest: Vec<GraphNodeId> = g
                .nodes
                .keys()
                .copied()
                .filter(|id| {
                    NodeGraphCanvasWith::<M>::node_is_selectable(g, &snapshot.interaction, *id)
                })
                .filter(|id| used.insert(*id))
                .collect();
            rest.sort_unstable();
            out.extend(rest);
            out
        })
        .ok()
        .unwrap_or_default();

    if ordered.is_empty() {
        return false;
    }

    let current = canvas
        .interaction
        .focused_node
        .or_else(|| snapshot.selected_nodes.first().copied());

    let next = match current.and_then(|id| ordered.iter().position(|e| *e == id)) {
        Some(ix) => {
            let len = ordered.len();
            let next_ix = if forward {
                (ix + 1) % len
            } else {
                (ix + len - 1) % len
            };
            ordered[next_ix]
        }
        None => {
            if forward {
                ordered[0]
            } else {
                ordered[ordered.len() - 1]
            }
        }
    };

    canvas.interaction.focused_node = Some(next);
    canvas.interaction.focused_edge = None;
    canvas.interaction.focused_port = None;
    canvas.interaction.focused_port_valid = false;
    canvas.interaction.focused_port_convertible = false;
    canvas.update_view_state(host, |s| {
        s.selected_edges.clear();
        s.selected_groups.clear();
        s.selected_nodes.clear();
        s.selected_nodes.push(next);
        s.draw_order.retain(|id| *id != next);
        s.draw_order.push(next);
    });

    let snapshot = canvas.sync_view_state(host);
    if snapshot.interaction.auto_pan.on_node_focus {
        canvas.stop_viewport_animation_timer(host);
        let (geom, _index) = canvas.canvas_derived(&*host, &snapshot);
        if let Some(ng) = geom.nodes.get(&next) {
            let rect = ng.rect;
            let center = CanvasPoint {
                x: rect.origin.x.0 + 0.5 * rect.size.width.0,
                y: rect.origin.y.0 + 0.5 * rect.size.height.0,
            };
            canvas.ensure_canvas_point_visible(host, &snapshot, center);
        }
    }
    true
}

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

    canvas.interaction.focused_node = Some(focused_node);
    canvas.interaction.focused_edge = None;
    canvas.interaction.focused_port = Some(next);
    canvas.refresh_focused_port_hints(host);
    canvas.update_view_state(host, |s| {
        s.selected_edges.clear();
        s.selected_groups.clear();
        s.selected_nodes.clear();
        s.selected_nodes.push(focused_node);
    });
    true
}
