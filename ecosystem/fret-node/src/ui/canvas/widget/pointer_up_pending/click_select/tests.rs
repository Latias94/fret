use fret_core::{Point, Px};

use super::*;

fn point(x: f32, y: f32) -> Point {
    Point::new(Px(x), Px(y))
}

#[test]
fn is_click_release_accepts_zero_threshold() {
    assert!(is_click_release(
        point(0.0, 0.0),
        point(20.0, 30.0),
        0.0,
        1.0
    ));
}

#[test]
fn is_click_release_rejects_distance_past_threshold() {
    assert!(!is_click_release(
        point(0.0, 0.0),
        point(10.0, 0.0),
        4.0,
        1.0,
    ));
    assert!(is_click_release(point(0.0, 0.0), point(3.0, 0.0), 4.0, 1.0,));
}

#[test]
fn apply_pending_node_selection_toggles_selection_and_keeps_node_last_in_draw_order() {
    let node = NodeId::new();
    let other = NodeId::new();
    let mut view = NodeGraphViewState {
        selected_nodes: vec![other],
        draw_order: vec![node, other],
        ..Default::default()
    };

    apply_pending_node_selection(&mut view, node, PendingNodeSelectAction::Toggle);
    assert_eq!(view.selected_nodes, vec![other, node]);
    assert_eq!(view.draw_order, vec![other, node]);

    apply_pending_node_selection(&mut view, node, PendingNodeSelectAction::Toggle);
    assert_eq!(view.selected_nodes, vec![other]);
    assert_eq!(view.draw_order, vec![other, node]);
}
