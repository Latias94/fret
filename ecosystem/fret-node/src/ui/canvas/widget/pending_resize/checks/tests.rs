use super::should_activate_pending_node_resize;
use fret_core::{Point, Px};

#[test]
fn should_activate_pending_node_resize_respects_threshold() {
    let start = Point::new(Px(0.0), Px(0.0));
    assert!(!should_activate_pending_node_resize(
        start,
        Point::new(Px(4.0), Px(0.0)),
        5.0,
        1.0,
    ));
    assert!(should_activate_pending_node_resize(
        start,
        Point::new(Px(6.0), Px(0.0)),
        5.0,
        1.0,
    ));
}
