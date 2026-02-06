use std::collections::BTreeSet;

use crate::core::{Graph, GroupId, NodeId, PortDirection, PortId};

pub(crate) fn node_order(graph: &Graph, draw_order: &[NodeId]) -> Vec<NodeId> {
    let mut seen: BTreeSet<NodeId> = BTreeSet::new();
    let mut out: Vec<NodeId> = Vec::new();

    let visible = |id: &NodeId| graph.nodes.get(id).is_some_and(|n| !n.hidden);

    for id in draw_order {
        if visible(id) && seen.insert(*id) {
            out.push(*id);
        }
    }

    for id in graph.nodes.keys() {
        if !visible(id) {
            continue;
        }
        if seen.insert(*id) {
            out.push(*id);
        }
    }

    out
}

pub(crate) fn group_order(graph: &Graph, draw_order: &[GroupId]) -> Vec<GroupId> {
    let mut seen: BTreeSet<GroupId> = BTreeSet::new();
    let mut out: Vec<GroupId> = Vec::new();

    for id in draw_order {
        if graph.groups.contains_key(id) && seen.insert(*id) {
            out.push(*id);
        }
    }

    for id in graph.groups.keys() {
        if seen.insert(*id) {
            out.push(*id);
        }
    }

    out
}

pub(crate) fn node_ports(graph: &Graph, node: NodeId) -> (Vec<PortId>, Vec<PortId>) {
    let Some(n) = graph.nodes.get(&node) else {
        return (Vec::new(), Vec::new());
    };

    let mut inputs: Vec<PortId> = Vec::new();
    let mut outputs: Vec<PortId> = Vec::new();
    for port_id in &n.ports {
        let Some(p) = graph.ports.get(port_id) else {
            continue;
        };
        match p.dir {
            PortDirection::In => inputs.push(*port_id),
            PortDirection::Out => outputs.push(*port_id),
        }
    }

    (inputs, outputs)
}
