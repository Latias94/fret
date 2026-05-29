use std::collections::BTreeSet;

use fret_core::Point;

use crate::core::{EdgeId, EdgeReconnectable, EdgeReconnectableEndpoint, Graph, PortId};
use crate::io::{NodeGraphInteractionState, NodeGraphViewState};
use crate::rules::EdgeEndpoint;
use crate::ui::internals::NodeGraphInternalsSnapshot;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct EdgeUpdateAnchorInfo {
    pub(super) edge: EdgeId,
    pub(super) endpoint: EdgeEndpoint,
    pub(super) anchor_port: PortId,
    pub(super) opposite_port: PortId,
    pub(super) center_window: Point,
    pub(super) radius: f32,
}

pub(super) fn collect_edge_update_anchor_infos(
    graph: &Graph,
    view_state: &NodeGraphViewState,
    internals: &NodeGraphInternalsSnapshot,
    interaction: &NodeGraphInteractionState,
) -> Vec<EdgeUpdateAnchorInfo> {
    let radius = normalized_reconnect_radius(interaction.reconnect_radius);
    if radius <= 0.0 {
        return Vec::new();
    }

    let mut candidates = BTreeSet::<EdgeId>::new();
    candidates.extend(view_state.selected_edges.iter().copied());
    if let Some(focused_edge) = internals.focused_edge {
        candidates.insert(focused_edge);
    }

    let mut out = Vec::new();
    for edge_id in candidates {
        let Some(edge) = graph.edges.get(&edge_id) else {
            continue;
        };

        for endpoint in [EdgeEndpoint::From, EdgeEndpoint::To] {
            if !edge_reconnect_endpoint_enabled(
                edge.reconnectable,
                interaction.edges_reconnectable,
                endpoint,
            ) {
                continue;
            }

            let (anchor_port, opposite_port) = match endpoint {
                EdgeEndpoint::From => (edge.from, edge.to),
                EdgeEndpoint::To => (edge.to, edge.from),
            };
            let Some(center_window) = internals.port_centers_window.get(&anchor_port).copied()
            else {
                continue;
            };

            out.push(EdgeUpdateAnchorInfo {
                edge: edge_id,
                endpoint,
                anchor_port,
                opposite_port,
                center_window,
                radius,
            });
        }
    }

    out
}

pub(super) fn edge_reconnect_endpoint_enabled(
    edge_reconnectable: Option<EdgeReconnectable>,
    global_edges_reconnectable: bool,
    endpoint: EdgeEndpoint,
) -> bool {
    match edge_reconnectable {
        None => global_edges_reconnectable,
        Some(EdgeReconnectable::Bool(enabled)) => enabled,
        Some(EdgeReconnectable::Endpoint(EdgeReconnectableEndpoint::Source)) => {
            endpoint == EdgeEndpoint::From
        }
        Some(EdgeReconnectable::Endpoint(EdgeReconnectableEndpoint::Target)) => {
            endpoint == EdgeEndpoint::To
        }
    }
}

fn normalized_reconnect_radius(radius: f32) -> f32 {
    if radius.is_finite() && radius > 0.0 {
        radius
    } else {
        0.0
    }
}
