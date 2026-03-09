//! Reusable editor workflows for the node graph canvas.
//!
//! This module is intentionally UI-light: it focuses on planning ops for common editor-grade
//! workflows (XyFlow/ImGui-node-editor parity), while letting the canvas decide how to surface UI
//! (context menus, searchers, overlays).

use std::sync::Arc;

use crate::core::{Graph, NodeId, PortDirection, PortId};
use crate::interaction::NodeGraphConnectionMode;
use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
use crate::rules::{ConnectDecision, DiagnosticSeverity};
use crate::ui::presenter::NodeGraphPresenter;

mod workflow_insert;
pub(crate) use workflow_insert::plan_wire_drop_insert;

#[derive(Debug, Clone)]
pub(crate) struct WireDropInsertPlan {
    pub ops: Vec<GraphOp>,
    pub created_node: Option<NodeId>,
    pub continue_from: Option<PortId>,
    pub toast: Option<(DiagnosticSeverity, Arc<str>)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::{CanvasPoint, Node, NodeKindKey, Port, PortCapacity, PortKey, PortKind};
    use crate::schema::{NodeRegistry, NodeSchema, PortDecl};
    use crate::types::TypeDesc;
    use crate::ui::presenter::{InsertNodeCandidate, RegistryNodeGraphPresenter};

    #[test]
    fn wire_drop_insert_autoconnects_first_compatible_port() {
        let mut reg = NodeRegistry::default();
        reg.register(NodeSchema {
            kind: NodeKindKey::new("demo.pipe"),
            latest_kind_version: 1,
            kind_aliases: vec![],
            title: "Pipe".to_string(),
            category: vec![],
            keywords: vec![],
            ports: vec![
                PortDecl {
                    key: PortKey::new("in"),
                    dir: PortDirection::In,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Single,
                    ty: Some(TypeDesc::Float),
                    label: None,
                },
                PortDecl {
                    key: PortKey::new("out"),
                    dir: PortDirection::Out,
                    kind: PortKind::Data,
                    capacity: PortCapacity::Multi,
                    ty: Some(TypeDesc::Float),
                    label: None,
                },
            ],
            default_data: serde_json::Value::Null,
        });

        let mut graph = Graph::default();
        let src_node = NodeId::new();
        let src_out = PortId::new();
        graph.nodes.insert(
            src_node,
            Node {
                kind: NodeKindKey::new("demo.src"),
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
                ports: vec![src_out],
                data: serde_json::Value::Null,
            },
        );
        graph.ports.insert(
            src_out,
            Port {
                node: src_node,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: Some(TypeDesc::Float),
                data: serde_json::Value::Null,
            },
        );

        let mut presenter = RegistryNodeGraphPresenter::new(reg);
        let candidate = InsertNodeCandidate {
            kind: NodeKindKey::new("demo.pipe"),
            label: Arc::<str>::from("Pipe"),
            enabled: true,
            template: None,
            payload: serde_json::Value::Null,
        };
        let insert_ops = presenter
            .plan_create_node(&graph, &candidate, CanvasPoint { x: 10.0, y: 20.0 })
            .expect("create ops");

        let planned = plan_wire_drop_insert(
            &mut presenter,
            &graph,
            src_out,
            crate::interaction::NodeGraphConnectionMode::Strict,
            insert_ops,
        );
        assert!(planned.toast.is_none());
        assert!(planned.created_node.is_some());
        assert!(planned.continue_from.is_some());
        assert!(!planned.ops.is_empty());

        let tx = GraphTransaction {
            label: None,
            ops: planned.ops,
        };
        apply_transaction(&mut graph, &tx).expect("apply");
        assert_eq!(graph.edges.len(), 1);
    }
}
