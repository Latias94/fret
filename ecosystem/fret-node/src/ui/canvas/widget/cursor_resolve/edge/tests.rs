use super::*;

#[test]
fn target_edge_prefers_focused_edge_over_single_selected_edge() {
    let focused = EdgeId::new();
    let selected = EdgeId::new();

    assert_eq!(
        target_edge_for_anchor(Some(focused), &[selected]),
        Some(focused)
    );
}

#[test]
fn target_edge_uses_single_selected_edge_when_focused_edge_is_missing() {
    let selected = EdgeId::new();

    assert_eq!(target_edge_for_anchor(None, &[selected]), Some(selected));
    assert_eq!(target_edge_for_anchor(None, &[]), None);
}
