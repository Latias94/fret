use super::super::insert_candidates::reroute_insert_candidate;
use super::super::*;
use super::*;
use crate::core::{EdgeId, GroupId, NodeKindKey};
use crate::rules::{DiagnosticSeverity, DiagnosticTarget};
use serde_json::Value;

fn regular_candidate() -> InsertNodeCandidate {
    InsertNodeCandidate {
        kind: NodeKindKey::new("regular"),
        label: Arc::<str>::from("Regular"),
        enabled: true,
        template: None,
        payload: Value::Null,
    }
}

#[test]
fn reroute_insert_candidate_detection_is_kind_based() {
    assert!(is_reroute_insert_candidate(&reroute_insert_candidate()));
    assert!(!is_reroute_insert_candidate(&regular_candidate()));
}

#[test]
fn select_inserted_node_clears_other_selection_kinds() {
    let node_id = GraphNodeId::new();
    let existing_node = GraphNodeId::new();
    let edge_id = EdgeId::new();
    let group_id = GroupId::new();
    let mut view_state = NodeGraphViewState::default();
    view_state.selected_nodes.push(existing_node);
    view_state.selected_edges.push(edge_id);
    view_state.selected_groups.push(group_id);

    select_inserted_node_in_view_state(&mut view_state, node_id);

    assert_eq!(view_state.selected_nodes, vec![node_id]);
    assert!(view_state.selected_edges.is_empty());
    assert!(view_state.selected_groups.is_empty());
}

#[test]
fn select_inserted_node_moves_node_to_draw_order_tail() {
    let node_id = GraphNodeId::new();
    let other = GraphNodeId::new();
    let mut view_state = NodeGraphViewState::default();
    view_state.draw_order.extend([node_id, other]);

    select_inserted_node_in_view_state(&mut view_state, node_id);

    assert_eq!(view_state.draw_order, vec![other, node_id]);
}

#[test]
fn split_edge_candidate_rejection_toast_uses_first_diagnostic_message() {
    let candidate = regular_candidate();
    let toast =
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::split_edge_candidate_rejection_toast(
            &candidate,
            &[Diagnostic {
                key: "insert_rejected".into(),
                severity: DiagnosticSeverity::Warning,
                target: DiagnosticTarget::Graph,
                message: "insert was rejected".into(),
                fixes: Vec::new(),
            }],
        );

    assert_eq!(toast.0, DiagnosticSeverity::Warning);
    assert_eq!(&*toast.1, "insert was rejected");
}

#[test]
fn split_edge_candidate_rejection_toast_falls_back_to_candidate_kind() {
    let candidate = regular_candidate();
    let toast =
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::split_edge_candidate_rejection_toast(
            &candidate,
            &[Diagnostic {
                key: "insert_rejected".into(),
                severity: DiagnosticSeverity::Info,
                target: DiagnosticTarget::Graph,
                message: String::new(),
                fixes: Vec::new(),
            }],
        );

    assert_eq!(toast.0, DiagnosticSeverity::Error);
    assert_eq!(&*toast.1, "node insertion was rejected: regular");
}
