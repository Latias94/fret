use std::collections::BTreeSet;

use super::super::super::*;

#[test]
fn collect_box_select_edges_connected_selects_any_connected_endpoint() {
    let (graph, node_a, node_b, _node_c, edge_ab, edge_bc, edge_ac_hidden) =
        super::test_support::sample_graph();
    let nodes = BTreeSet::from([node_a, node_b]);
    let interaction = NodeGraphInteractionState::default();

    let edges = super::graph::collect_box_select_edges_from_graph(&graph, &interaction, &nodes);

    assert_eq!(edges, vec![edge_ab, edge_bc]);
    assert!(!edges.contains(&edge_ac_hidden));
}

#[test]
fn collect_box_select_edges_both_endpoints_requires_both_nodes_selected() {
    let (graph, node_a, node_b, _node_c, edge_ab, _edge_bc, _edge_ac_hidden) =
        super::test_support::sample_graph();
    let nodes = BTreeSet::from([node_a, node_b]);
    let mut interaction = NodeGraphInteractionState::default();
    interaction.box_select_edges = crate::io::NodeGraphBoxSelectEdges::BothEndpoints;

    let edges = super::graph::collect_box_select_edges_from_graph(&graph, &interaction, &nodes);

    assert_eq!(edges, vec![edge_ab]);
}

#[test]
fn collect_box_select_edges_respects_global_selection_gates() {
    let (graph, node_a, node_b, _node_c, _edge_ab, _edge_bc, _edge_ac_hidden) =
        super::test_support::sample_graph();
    let nodes = BTreeSet::from([node_a, node_b]);

    let mut interaction = NodeGraphInteractionState::default();
    interaction.elements_selectable = false;
    assert!(
        super::graph::collect_box_select_edges_from_graph(&graph, &interaction, &nodes).is_empty()
    );

    interaction = NodeGraphInteractionState::default();
    interaction.edges_selectable = false;
    assert!(
        super::graph::collect_box_select_edges_from_graph(&graph, &interaction, &nodes).is_empty()
    );

    interaction = NodeGraphInteractionState::default();
    interaction.box_select_edges = crate::io::NodeGraphBoxSelectEdges::None;
    assert!(
        super::graph::collect_box_select_edges_from_graph(&graph, &interaction, &nodes).is_empty()
    );
}
