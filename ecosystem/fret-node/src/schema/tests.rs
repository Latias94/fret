use std::sync::Arc;

use serde_json::json;

use crate::core::{Graph, GraphId, Node, NodeId, NodeKindKey};
use crate::ops::apply_transaction;
use crate::schema::{NodeKindMigrateError, NodeKindMigrator, NodeRegistry, NodeSchema};

struct DummyMigrator;

impl NodeKindMigrator for DummyMigrator {
    fn migrate(
        &self,
        from_version: u32,
        to_version: u32,
        data: &serde_json::Value,
    ) -> Result<serde_json::Value, NodeKindMigrateError> {
        Ok(json!({
            "from_version": from_version,
            "to_version": to_version,
            "prev": data,
            "migrated": true,
        }))
    }
}

#[test]
fn canonicalize_kinds_rewrites_aliases_to_canonical() {
    let mut registry = NodeRegistry::new();
    registry.register(NodeSchema {
        kind: NodeKindKey::new("demo.add"),
        latest_kind_version: 1,
        kind_aliases: vec![NodeKindKey::new("demo.add.v0")],
        title: "Add".into(),
        category: Vec::new(),
        keywords: Vec::new(),
        ports: Vec::new(),
        default_data: serde_json::Value::Null,
    });

    let id = NodeId::new();
    let mut graph = Graph::new(GraphId::new());
    graph.nodes.insert(
        id,
        Node {
            kind: NodeKindKey::new("demo.add.v0"),
            kind_version: 0,
            pos: crate::core::CanvasPoint { x: 0.0, y: 0.0 },
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
        },
    );

    let plan = registry.plan_canonicalize_kinds(&graph);
    assert_eq!(plan.rewrites.len(), 1);
    assert_eq!(plan.rewrites[0].node, id);

    apply_transaction(&mut graph, &plan.tx).unwrap();
    assert_eq!(
        graph.nodes.get(&id).unwrap().kind,
        NodeKindKey::new("demo.add")
    );
}

#[test]
fn migrate_nodes_emits_set_node_data_and_version_and_reports_upgraded() {
    let mut registry = NodeRegistry::new();
    registry.register(NodeSchema {
        kind: NodeKindKey::new("demo.add"),
        latest_kind_version: 2,
        kind_aliases: Vec::new(),
        title: "Add".into(),
        category: Vec::new(),
        keywords: Vec::new(),
        ports: Vec::new(),
        default_data: serde_json::Value::Null,
    });
    registry.register_migrator(NodeKindKey::new("demo.add"), Arc::new(DummyMigrator));

    let id = NodeId::new();
    let mut graph = Graph::new(GraphId::new());
    graph.nodes.insert(
        id,
        Node {
            kind: NodeKindKey::new("demo.add"),
            kind_version: 0,
            pos: crate::core::CanvasPoint { x: 0.0, y: 0.0 },
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
            data: json!({"x": 1}),
        },
    );

    let plan = registry.plan_migrate_nodes(&graph);
    assert_eq!(plan.report.upgraded.len(), 1);
    assert!(plan.report.missing_schema.is_empty());
    assert!(plan.report.missing_migrator.is_empty());
    assert!(plan.report.errors.is_empty());

    apply_transaction(&mut graph, &plan.tx).unwrap();
    let node = graph.nodes.get(&id).unwrap();
    assert_eq!(node.kind_version, 2);
    assert_eq!(node.data["migrated"], json!(true));
}

#[test]
fn migrate_nodes_reports_missing_migrator_and_emits_no_tx() {
    let mut registry = NodeRegistry::new();
    registry.register(NodeSchema {
        kind: NodeKindKey::new("demo.add"),
        latest_kind_version: 2,
        kind_aliases: Vec::new(),
        title: "Add".into(),
        category: Vec::new(),
        keywords: Vec::new(),
        ports: Vec::new(),
        default_data: serde_json::Value::Null,
    });

    let id = NodeId::new();
    let mut graph = Graph::new(GraphId::new());
    graph.nodes.insert(
        id,
        Node {
            kind: NodeKindKey::new("demo.add"),
            kind_version: 0,
            pos: crate::core::CanvasPoint { x: 0.0, y: 0.0 },
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
        },
    );

    let plan = registry.plan_migrate_nodes(&graph);
    assert_eq!(plan.report.missing_migrator.len(), 1);
    assert!(plan.tx.is_empty());
}
