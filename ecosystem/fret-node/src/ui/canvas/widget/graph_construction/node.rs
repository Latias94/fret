use super::super::*;

fn reroute_node(at: CanvasPoint) -> crate::core::Node {
    crate::core::Node {
        kind: NodeKindKey::new(REROUTE_KIND),
        kind_version: 1,
        pos: at,
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
        ports: Vec::new(),
        data: serde_json::Value::Null,
    }
}

fn reroute_port(node_id: GraphNodeId, key: &'static str, dir: PortDirection) -> crate::core::Port {
    crate::core::Port {
        node: node_id,
        key: crate::core::PortKey::new(key),
        dir,
        kind: crate::core::PortKind::Data,
        capacity: match dir {
            PortDirection::In => crate::core::PortCapacity::Single,
            PortDirection::Out => crate::core::PortCapacity::Multi,
        },
        connectable: None,
        connectable_start: None,
        connectable_end: None,
        ty: None,
        data: serde_json::Value::Null,
    }
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn first_added_node_id(ops: &[GraphOp]) -> Option<GraphNodeId> {
        for op in ops {
            if let GraphOp::AddNode { id, .. } = op {
                return Some(*id);
            }
        }
        None
    }

    pub(in super::super) fn build_reroute_create_ops(at: CanvasPoint) -> Vec<GraphOp> {
        let node_id = GraphNodeId::new();
        let in_port_id = PortId::new();
        let out_port_id = PortId::new();

        vec![
            GraphOp::AddNode {
                id: node_id,
                node: reroute_node(at),
            },
            GraphOp::AddPort {
                id: in_port_id,
                port: reroute_port(node_id, "in", PortDirection::In),
            },
            GraphOp::AddPort {
                id: out_port_id,
                port: reroute_port(node_id, "out", PortDirection::Out),
            },
            GraphOp::SetNodePorts {
                id: node_id,
                from: Vec::new(),
                to: vec![in_port_id, out_port_id],
            },
        ]
    }
}

#[cfg(test)]
mod tests;
