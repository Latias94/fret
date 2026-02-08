use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, GraphImport, GraphImportError, Node,
    NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind, Symbol,
    SymbolId, collect_subgraph_targets, collect_symbol_ref_targets, resolve_import_closure,
    validate_graph,
};
use crate::core::{CanvasSize, GraphValidationError, GroupId, validate_graph_structural};
use crate::core::{SUBGRAPH_NODE_KIND, validate_subgraph_targets_are_imported};
use crate::core::{SYMBOL_REF_NODE_KIND, validate_symbol_ref_targets_are_declared};

fn make_node(kind: &str) -> Node {
    Node {
        kind: NodeKindKey::new(kind),
        kind_version: 0,
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
        ports: Vec::new(),
        data: serde_json::Value::Null,
    }
}

fn make_port(
    node: NodeId,
    key: &str,
    dir: PortDirection,
    kind: PortKind,
    capacity: PortCapacity,
) -> Port {
    Port {
        node,
        key: PortKey::new(key),
        dir,
        kind,
        capacity,
        connectable: None,
        connectable_start: None,
        connectable_end: None,
        ty: None,
        data: serde_json::Value::Null,
    }
}

#[test]
fn validate_allows_edges_regardless_of_port_direction() {
    let mut graph = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out_a = PortId::new();
    let out_b = PortId::new();
    graph.ports.insert(
        out_a,
        make_port(
            a,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        out_b,
        make_port(
            b,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out_a,
            to: out_b,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let report = validate_graph(&graph);
    assert!(report.is_ok());
}

#[test]
fn validate_rejects_node_with_missing_parent_group() {
    let mut graph = Graph::default();
    let n = NodeId::new();
    let mut node = make_node("core.a");
    node.parent = Some(GroupId::new());
    graph.nodes.insert(n, node);

    let report = validate_graph(&graph);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        GraphValidationError::NodeParentMissingGroup { node, .. } if *node == n
    )));
}

#[test]
fn validate_rejects_node_with_invalid_size() {
    let mut graph = Graph::default();
    let n = NodeId::new();
    let mut node = make_node("core.a");
    node.size = Some(CanvasSize {
        width: -1.0,
        height: 10.0,
    });
    graph.nodes.insert(n, node);

    let report = validate_graph(&graph);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        GraphValidationError::NodeInvalidSize { node, .. } if *node == n
    )));
}

#[test]
fn validate_rejects_edge_kind_that_does_not_match_port_kind() {
    let mut graph = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out_a = PortId::new();
    let in_b = PortId::new();
    graph.ports.insert(
        out_a,
        make_port(
            a,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        in_b,
        make_port(
            b,
            "in",
            PortDirection::In,
            PortKind::Data,
            PortCapacity::Single,
        ),
    );

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Exec,
            from: out_a,
            to: in_b,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let report = validate_graph(&graph);
    assert!(!report.is_ok());
}

#[test]
fn import_closure_is_deterministic_and_postordered() {
    let a = GraphId::from_u128(1);
    let b = GraphId::from_u128(2);
    let c = GraphId::from_u128(3);
    let d = GraphId::from_u128(4);

    let mut g_a = Graph::new(a);
    g_a.imports.insert(b, GraphImport::default());
    g_a.imports.insert(c, GraphImport::default());

    let mut g_b = Graph::new(b);
    g_b.imports.insert(d, GraphImport::default());

    let g_c = Graph::new(c);
    let g_d = Graph::new(d);

    let mut db = std::collections::BTreeMap::new();
    db.insert(a, g_a);
    db.insert(b, g_b);
    db.insert(c, g_c);
    db.insert(d, g_d);

    let root = db.get(&a).expect("root graph must exist");
    let closure = resolve_import_closure(root, |id| db.get(&id)).expect("resolve imports");

    assert_eq!(closure.order, vec![d, b, c]);
    assert!(closure.contains(b));
    assert!(closure.contains(c));
    assert!(closure.contains(d));
    assert!(!closure.contains(a));
}

#[test]
fn import_closure_rejects_missing_graph() {
    let a = GraphId::from_u128(1);
    let missing = GraphId::from_u128(9);

    let mut g_a = Graph::new(a);
    g_a.imports.insert(missing, GraphImport::default());

    let mut db = std::collections::BTreeMap::new();
    db.insert(a, g_a);

    let root = db.get(&a).expect("root graph must exist");
    let err =
        resolve_import_closure(root, |id| db.get(&id)).expect_err("expected missing graph error");
    assert_eq!(
        err,
        GraphImportError::MissingGraph {
            from: a,
            to: missing
        }
    );
}

#[test]
fn import_closure_rejects_cycles_with_stable_path() {
    let a = GraphId::from_u128(1);
    let b = GraphId::from_u128(2);
    let c = GraphId::from_u128(3);

    let mut g_a = Graph::new(a);
    g_a.imports.insert(b, GraphImport::default());
    let mut g_b = Graph::new(b);
    g_b.imports.insert(c, GraphImport::default());
    let mut g_c = Graph::new(c);
    g_c.imports.insert(a, GraphImport::default());

    let mut db = std::collections::BTreeMap::new();
    db.insert(a, g_a);
    db.insert(b, g_b);
    db.insert(c, g_c);

    let root = db.get(&a).expect("root graph must exist");
    let err = resolve_import_closure(root, |id| db.get(&id)).expect_err("expected cycle error");
    assert_eq!(
        err,
        GraphImportError::Cycle {
            cycle: vec![a, b, c, a]
        }
    );
}

#[test]
fn subgraph_nodes_must_reference_declared_imports() {
    let mut graph = Graph::default();

    let node_id = NodeId::new();
    let mut node = make_node(SUBGRAPH_NODE_KIND);
    let imported = GraphId::from_u128(2);
    node.data = serde_json::json!({ "graph_id": imported });
    graph.nodes.insert(node_id, node);

    assert!(
        validate_subgraph_targets_are_imported(&graph).is_err(),
        "expected missing import to be rejected"
    );

    graph.imports.insert(imported, GraphImport::default());
    assert!(
        validate_subgraph_targets_are_imported(&graph).is_ok(),
        "expected declared import to satisfy binding"
    );
}

#[test]
fn subgraph_targets_must_resolve_through_import_closure() {
    let a = GraphId::from_u128(1);
    let b = GraphId::from_u128(2);
    let c = GraphId::from_u128(3);

    let mut g_a = Graph::new(a);
    g_a.imports.insert(b, GraphImport::default());

    let node_id = NodeId::new();
    let mut node = make_node(SUBGRAPH_NODE_KIND);
    node.data = serde_json::json!({ "graph_id": b });
    g_a.nodes.insert(node_id, node);

    let mut g_b = Graph::new(b);
    g_b.imports.insert(c, GraphImport::default());

    let g_c = Graph::new(c);

    let mut db = std::collections::BTreeMap::new();
    db.insert(a, g_a);
    db.insert(b, g_b);
    db.insert(c, g_c);

    let root = db.get(&a).expect("root graph must exist");

    validate_subgraph_targets_are_imported(root).expect("targets must be declared imports");
    let targets = collect_subgraph_targets(root).expect("collect targets");
    assert_eq!(targets.iter().copied().collect::<Vec<_>>(), vec![b]);

    let closure = resolve_import_closure(root, |id| db.get(&id)).expect("resolve closure");
    assert_eq!(closure.order, vec![c, b]);
    assert!(closure.contains(b));
    assert!(closure.contains(c));

    for target in targets {
        assert!(closure.contains(target));
    }
}

#[test]
fn validate_graph_reports_subgraph_import_binding_errors() {
    let mut graph = Graph::default();

    let node_id = NodeId::new();
    let mut node = make_node(SUBGRAPH_NODE_KIND);
    let imported = GraphId::from_u128(2);
    node.data = serde_json::json!({ "graph_id": imported });
    graph.nodes.insert(node_id, node);

    let report = validate_graph_structural(&graph);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        GraphValidationError::SubgraphTargetNotImported { node, graph_id }
            if *node == node_id && *graph_id == imported
    )));

    graph.imports.insert(imported, GraphImport::default());
    let report = validate_graph(&graph);
    assert!(report.is_ok());

    let mut bad = make_node(SUBGRAPH_NODE_KIND);
    bad.data = serde_json::json!({});
    let bad_id = NodeId::new();
    graph.nodes.insert(bad_id, bad);
    let report = validate_graph_structural(&graph);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        GraphValidationError::SubgraphNodeMissingGraphId { node } if *node == bad_id
    )));
}

#[test]
fn symbol_ref_nodes_must_reference_declared_symbols() {
    let mut graph = Graph::default();

    let symbol_id = SymbolId::from_u128(10);
    graph.symbols.insert(
        symbol_id,
        Symbol {
            name: "S".to_string(),
            ty: None,
            default_value: None,
            meta: serde_json::Value::Null,
        },
    );

    let node_id = NodeId::new();
    let mut node = make_node(SYMBOL_REF_NODE_KIND);
    node.data = serde_json::json!({ "symbol_id": symbol_id });
    graph.nodes.insert(node_id, node);

    assert!(
        validate_symbol_ref_targets_are_declared(&graph).is_ok(),
        "expected declared symbol to satisfy binding"
    );

    let targets = collect_symbol_ref_targets(&graph).expect("collect symbol ref targets");
    assert_eq!(targets.iter().copied().collect::<Vec<_>>(), vec![symbol_id]);
}

#[test]
fn validate_graph_reports_symbol_ref_binding_errors() {
    let mut graph = Graph::default();

    let node_id = NodeId::new();
    let mut node = make_node(SYMBOL_REF_NODE_KIND);
    let symbol_id = SymbolId::from_u128(99);
    node.data = serde_json::json!({ "symbol_id": symbol_id });
    graph.nodes.insert(node_id, node);

    let report = validate_graph_structural(&graph);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        GraphValidationError::SymbolRefTargetNotDeclared { node, symbol_id: s }
            if *node == node_id && *s == symbol_id
    )));

    graph.symbols.insert(
        symbol_id,
        Symbol {
            name: "S".to_string(),
            ty: None,
            default_value: None,
            meta: serde_json::Value::Null,
        },
    );
    let report = validate_graph(&graph);
    assert!(report.is_ok());

    let mut bad = make_node(SYMBOL_REF_NODE_KIND);
    bad.data = serde_json::json!({});
    let bad_id = NodeId::new();
    graph.nodes.insert(bad_id, bad);
    let report = validate_graph_structural(&graph);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        GraphValidationError::SymbolRefNodeMissingSymbolId { node } if *node == bad_id
    )));
}
