use std::collections::BTreeSet;

use super::super::*;

fn box_select_edge_mode(
    interaction: &NodeGraphInteractionState,
) -> Option<crate::io::NodeGraphBoxSelectEdges> {
    if !interaction.elements_selectable || !interaction.edges_selectable {
        return None;
    }

    match interaction.box_select_edges {
        crate::io::NodeGraphBoxSelectEdges::None => None,
        crate::io::NodeGraphBoxSelectEdges::Connected => {
            Some(crate::io::NodeGraphBoxSelectEdges::Connected)
        }
        crate::io::NodeGraphBoxSelectEdges::BothEndpoints => {
            Some(crate::io::NodeGraphBoxSelectEdges::BothEndpoints)
        }
    }
}

fn edge_matches_box_select_mode(
    mode: crate::io::NodeGraphBoxSelectEdges,
    nodes: &BTreeSet<GraphNodeId>,
    source_node: GraphNodeId,
    target_node: GraphNodeId,
) -> bool {
    match mode {
        crate::io::NodeGraphBoxSelectEdges::None => false,
        crate::io::NodeGraphBoxSelectEdges::Connected => {
            nodes.contains(&source_node) || nodes.contains(&target_node)
        }
        crate::io::NodeGraphBoxSelectEdges::BothEndpoints => {
            nodes.contains(&source_node) && nodes.contains(&target_node)
        }
    }
}

fn collect_box_select_edges_from_graph_with_mode(
    graph: &Graph,
    mode: crate::io::NodeGraphBoxSelectEdges,
    nodes: &BTreeSet<GraphNodeId>,
) -> Vec<EdgeId> {
    graph
        .edges
        .iter()
        .filter_map(|(edge_id, edge)| {
            if !edge.selectable.unwrap_or(true) {
                return None;
            }
            let from_node = graph.ports.get(&edge.from).map(|port| port.node)?;
            let to_node = graph.ports.get(&edge.to).map(|port| port.node)?;
            edge_matches_box_select_mode(mode, nodes, from_node, to_node).then_some(*edge_id)
        })
        .collect()
}

#[cfg(test)]
fn collect_box_select_edges_from_graph(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    nodes: &BTreeSet<GraphNodeId>,
) -> Vec<EdgeId> {
    let Some(mode) = box_select_edge_mode(interaction) else {
        return Vec::new();
    };
    collect_box_select_edges_from_graph_with_mode(graph, mode, nodes)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn box_select_edges_for_nodes<H: UiHost>(
        &self,
        host: &mut H,
        interaction: &NodeGraphInteractionState,
        nodes: &BTreeSet<GraphNodeId>,
    ) -> Vec<EdgeId> {
        let Some(mode) = box_select_edge_mode(interaction) else {
            return Vec::new();
        };

        if let Some(store) = self.store.as_ref() {
            if let Ok(out) = store.read_ref(host, |s| {
                let graph = s.graph();
                let mut out = BTreeSet::new();
                for &node in nodes {
                    let Some(conns) = s.lookups().connections_for_node(node) else {
                        continue;
                    };
                    for (edge_id, conn) in conns {
                        let Some(edge) = graph.edges.get(edge_id) else {
                            continue;
                        };
                        if !edge.selectable.unwrap_or(true) {
                            continue;
                        }
                        if edge_matches_box_select_mode(
                            mode,
                            nodes,
                            conn.source_node,
                            conn.target_node,
                        ) {
                            out.insert(*edge_id);
                        }
                    }
                }
                out.into_iter().collect::<Vec<_>>()
            }) {
                return out;
            }
        }

        self.graph
            .read_ref(host, |graph| {
                collect_box_select_edges_from_graph_with_mode(graph, mode, nodes)
            })
            .ok()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::core::{
        CanvasPoint, Edge, EdgeKind, GraphId, Node, NodeKindKey, Port, PortCapacity, PortDirection,
        PortKey, PortKind,
    };
    use serde_json::Value;

    fn sample_graph() -> (
        Graph,
        GraphNodeId,
        GraphNodeId,
        GraphNodeId,
        EdgeId,
        EdgeId,
        EdgeId,
    ) {
        let graph_id = GraphId::from_u128(1);
        let node_a = GraphNodeId::from_u128(10);
        let node_b = GraphNodeId::from_u128(11);
        let node_c = GraphNodeId::from_u128(12);
        let a_out = PortId::from_u128(20);
        let b_in = PortId::from_u128(21);
        let b_out = PortId::from_u128(22);
        let c_in = PortId::from_u128(23);
        let c_in_secondary = PortId::from_u128(24);
        let edge_ab = EdgeId::from_u128(30);
        let edge_bc = EdgeId::from_u128(31);
        let edge_ac_hidden = EdgeId::from_u128(32);

        let mut graph = Graph::new(graph_id);
        graph.nodes.insert(
            node_a,
            Node {
                kind: NodeKindKey::new("test.a"),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
                collapsed: false,
                ports: vec![a_out],
                data: Value::Null,
            },
        );
        graph.nodes.insert(
            node_b,
            Node {
                kind: NodeKindKey::new("test.b"),
                kind_version: 1,
                pos: CanvasPoint { x: 100.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
                collapsed: false,
                ports: vec![b_in, b_out],
                data: Value::Null,
            },
        );
        graph.nodes.insert(
            node_c,
            Node {
                kind: NodeKindKey::new("test.c"),
                kind_version: 1,
                pos: CanvasPoint { x: 200.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
                collapsed: false,
                ports: vec![c_in, c_in_secondary],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            a_out,
            Port {
                node: node_a,
                key: PortKey::new("a.out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            b_in,
            Port {
                node: node_b,
                key: PortKey::new("b.in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            b_out,
            Port {
                node: node_b,
                key: PortKey::new("b.out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            c_in,
            Port {
                node: node_c,
                key: PortKey::new("c.in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            c_in_secondary,
            Port {
                node: node_c,
                key: PortKey::new("c.in.secondary"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.edges.insert(
            edge_ab,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: b_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        graph.edges.insert(
            edge_bc,
            Edge {
                kind: EdgeKind::Data,
                from: b_out,
                to: c_in,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
        graph.edges.insert(
            edge_ac_hidden,
            Edge {
                kind: EdgeKind::Data,
                from: a_out,
                to: c_in_secondary,
                selectable: Some(false),
                deletable: None,
                reconnectable: None,
            },
        );

        (
            graph,
            node_a,
            node_b,
            node_c,
            edge_ab,
            edge_bc,
            edge_ac_hidden,
        )
    }

    #[test]
    fn collect_box_select_edges_connected_selects_any_connected_endpoint() {
        let (graph, node_a, node_b, _node_c, edge_ab, edge_bc, edge_ac_hidden) = sample_graph();
        let nodes = BTreeSet::from([node_a, node_b]);
        let interaction = NodeGraphInteractionState::default();

        let edges = collect_box_select_edges_from_graph(&graph, &interaction, &nodes);

        assert_eq!(edges, vec![edge_ab, edge_bc]);
        assert!(!edges.contains(&edge_ac_hidden));
    }

    #[test]
    fn collect_box_select_edges_both_endpoints_requires_both_nodes_selected() {
        let (graph, node_a, node_b, _node_c, edge_ab, _edge_bc, _edge_ac_hidden) = sample_graph();
        let nodes = BTreeSet::from([node_a, node_b]);
        let mut interaction = NodeGraphInteractionState::default();
        interaction.box_select_edges = crate::io::NodeGraphBoxSelectEdges::BothEndpoints;

        let edges = collect_box_select_edges_from_graph(&graph, &interaction, &nodes);

        assert_eq!(edges, vec![edge_ab]);
    }

    #[test]
    fn collect_box_select_edges_respects_global_selection_gates() {
        let (graph, node_a, node_b, _node_c, _edge_ab, _edge_bc, _edge_ac_hidden) = sample_graph();
        let nodes = BTreeSet::from([node_a, node_b]);

        let mut interaction = NodeGraphInteractionState::default();
        interaction.elements_selectable = false;
        assert!(collect_box_select_edges_from_graph(&graph, &interaction, &nodes).is_empty());

        interaction = NodeGraphInteractionState::default();
        interaction.edges_selectable = false;
        assert!(collect_box_select_edges_from_graph(&graph, &interaction, &nodes).is_empty());

        interaction = NodeGraphInteractionState::default();
        interaction.box_select_edges = crate::io::NodeGraphBoxSelectEdges::None;
        assert!(collect_box_select_edges_from_graph(&graph, &interaction, &nodes).is_empty());
    }
}
