use std::collections::BTreeSet;

use super::super::super::*;

pub(super) fn box_select_edge_mode(
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

pub(super) fn edge_matches_box_select_mode(
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
