mod drag;
mod node_resize;

use std::collections::HashMap;

use super::*;

pub(super) fn ports_for_node<'a>(
    node_ports: &'a HashMap<GraphNodeId, Vec<PortId>>,
    node_id: GraphNodeId,
) -> &'a [PortId] {
    node_ports
        .get(&node_id)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

pub(super) fn resolve_edge_endpoints_from_model<H: UiHost>(
    host: &H,
    graph: &fret_runtime::Model<Graph>,
    edge_ids: &[EdgeId],
) -> Vec<(EdgeId, PortId, PortId)> {
    graph
        .read_ref(host, |g| {
            edge_ids
                .iter()
                .filter_map(|edge_id| g.edges.get(edge_id).map(|e| (*edge_id, e.from, e.to)))
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_default()
}

pub(super) fn extend_ports_for_nodes(
    node_ports: &HashMap<GraphNodeId, Vec<PortId>>,
    nodes: impl IntoIterator<Item = GraphNodeId>,
    out: &mut Vec<PortId>,
) {
    out.clear();
    for node_id in nodes {
        if let Some(ports) = node_ports.get(&node_id) {
            out.extend(ports.iter().copied());
        }
    }
}

pub(super) fn moved_nodes_and_next_positions(
    current_positions: &HashMap<GraphNodeId, CanvasPoint>,
    nodes: &[(GraphNodeId, CanvasPoint)],
) -> Option<(
    Vec<(GraphNodeId, CanvasPoint, CanvasPoint)>,
    HashMap<GraphNodeId, CanvasPoint>,
)> {
    let mut moved_nodes: Vec<(GraphNodeId, CanvasPoint, CanvasPoint)> = Vec::new();
    for (id, pos) in nodes {
        let prev = current_positions.get(id).copied().unwrap_or_default();
        if prev != *pos {
            moved_nodes.push((*id, prev, *pos));
        }
    }
    if moved_nodes.is_empty() {
        return None;
    }

    let mut next_positions = current_positions.clone();
    for (id, _prev, next) in &moved_nodes {
        next_positions.insert(*id, *next);
    }

    Some((moved_nodes, next_positions))
}
