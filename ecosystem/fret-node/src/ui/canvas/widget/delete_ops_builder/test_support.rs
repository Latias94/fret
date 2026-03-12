use crate::core::{
    CanvasPoint, Node, NodeId as GraphNodeId, NodeKindKey, Port, PortCapacity, PortDirection,
    PortKey, PortKind,
};
use serde_json::Value;

pub(super) fn test_node(kind: &str, pos: CanvasPoint) -> Node {
    Node {
        kind: NodeKindKey::new(kind),
        kind_version: 1,
        pos,
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
        data: Value::Null,
    }
}

pub(super) fn test_port(node: GraphNodeId, key: &str, dir: PortDirection) -> Port {
    Port {
        node,
        key: PortKey::new(key),
        dir,
        kind: PortKind::Data,
        capacity: PortCapacity::Single,
        connectable: None,
        connectable_start: None,
        connectable_end: None,
        ty: None,
        data: Value::Null,
    }
}
