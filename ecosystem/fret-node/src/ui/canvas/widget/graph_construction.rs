use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn first_added_node_id(ops: &[GraphOp]) -> Option<GraphNodeId> {
        for op in ops {
            if let GraphOp::AddNode { id, .. } = op {
                return Some(*id);
            }
        }
        None
    }

    pub(super) fn build_reroute_create_ops(at: CanvasPoint) -> Vec<GraphOp> {
        let node_id = GraphNodeId::new();
        let in_port_id = PortId::new();
        let out_port_id = PortId::new();

        let node = crate::core::Node {
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
        };

        let in_port = crate::core::Port {
            node: node_id,
            key: crate::core::PortKey::new("in"),
            dir: PortDirection::In,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        };

        let out_port = crate::core::Port {
            node: node_id,
            key: crate::core::PortKey::new("out"),
            dir: PortDirection::Out,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        };

        vec![
            GraphOp::AddNode { id: node_id, node },
            GraphOp::AddPort {
                id: in_port_id,
                port: in_port,
            },
            GraphOp::AddPort {
                id: out_port_id,
                port: out_port,
            },
            GraphOp::SetNodePorts {
                id: node_id,
                from: Vec::new(),
                to: vec![in_port_id, out_port_id],
            },
        ]
    }

    pub(super) fn create_group_at<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        at: CanvasPoint,
    ) {
        let size = crate::core::CanvasSize {
            width: 480.0,
            height: 320.0,
        };
        let origin = crate::core::CanvasPoint {
            x: at.x - 0.5 * size.width,
            y: at.y - 0.5 * size.height,
        };
        let group = crate::core::Group {
            title: "Group".to_string(),
            rect: crate::core::CanvasRect { origin, size },
            color: None,
        };
        let group_id = crate::core::GroupId::new();
        let ops = vec![GraphOp::AddGroup {
            id: group_id,
            group,
        }];
        if self.commit_ops(host, window, Some("Create Group"), ops) {
            self.update_view_state(host, |s| {
                s.selected_nodes.clear();
                s.selected_edges.clear();
                s.selected_groups.clear();
                s.selected_groups.push(group_id);
                s.group_draw_order.retain(|id| *id != group_id);
                s.group_draw_order.push(group_id);
            });
        }
    }
}
