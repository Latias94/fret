use super::*;

#[test]
fn toast_rect_clamps_box_width_to_minimum() {
    let layout = toast_layout(1.0, 0.0, 0.0, 200.0);
    let rect = toast_rect(layout, 24.0, 18.0);
    assert_eq!(rect.size.width, Px(120.0));
}

#[test]
fn toast_rect_clamps_box_width_to_maximum() {
    let layout = toast_layout(1.0, 0.0, 0.0, 200.0);
    let rect = toast_rect(layout, 500.0, 18.0);
    assert_eq!(rect.size.width, Px(420.0));
}

#[test]
fn toast_rect_places_box_at_viewport_bottom_left() {
    let layout = toast_layout(2.0, 30.0, 40.0, 300.0);
    let rect = toast_rect(layout, 100.0, 20.0);
    assert_eq!(rect.origin.x, Px(36.0));
    assert_eq!(rect.origin.y, Px(304.0));
    assert_eq!(rect.size.width, Px(110.0));
    assert_eq!(rect.size.height, Px(30.0));
}
