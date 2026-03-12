use super::*;
use fret_core::{Point, Px};

#[test]
fn should_activate_pending_edge_insert_drag_respects_threshold() {
    let start = Point::new(Px(0.0), Px(0.0));
    assert!(!should_activate_pending_edge_insert_drag(
        start,
        Point::new(Px(4.0), Px(0.0)),
        5.0,
        1.0,
    ));
    assert!(should_activate_pending_edge_insert_drag(
        start,
        Point::new(Px(6.0), Px(0.0)),
        5.0,
        1.0,
    ));
}
