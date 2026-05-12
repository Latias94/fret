use super::*;

#[test]
fn drag_autoscroll_delta_y_is_zero_inside_safe_band() {
    let delta = drag_autoscroll_delta_y(Px(100.0), Px(30.0), Px(50.0));
    assert_eq!(delta, Px(0.0));
}

#[test]
fn drag_autoscroll_delta_y_uses_direction_and_clamp() {
    let up = drag_autoscroll_delta_y(Px(100.0), Px(30.0), Px(20.0));
    assert!(up.0 < 0.0);

    let down = drag_autoscroll_delta_y(Px(100.0), Px(30.0), Px(80.0));
    assert!(down.0 > 0.0);

    let capped = drag_autoscroll_delta_y(Px(100.0), Px(30.0), Px(10_000.0));
    assert!((capped.0 - 3.0).abs() < 1e-6);
}

#[test]
fn drag_autoscroll_delta_y_handles_zero_sizes() {
    assert_eq!(
        drag_autoscroll_delta_y(Px(0.0), Px(100.0), Px(20.0)),
        Px(0.0)
    );
    assert_eq!(
        drag_autoscroll_delta_y(Px(20.0), Px(0.0), Px(20.0)),
        Px(0.0)
    );
}

#[test]
fn display_row_for_pointer_y_clamps_outside_viewport() {
    let bounds = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(100.0), Px(60.0)),
    );
    assert_eq!(
        display_row_for_pointer_y(bounds, Px(10.0), Px(5.0), 5),
        Some(0)
    );
    assert_eq!(
        display_row_for_pointer_y(bounds, Px(10.0), Px(25.0), 5),
        Some(0)
    );
    assert_eq!(
        display_row_for_pointer_y(bounds, Px(10.0), Px(95.0), 5),
        Some(4)
    );
}

#[test]
fn display_row_for_pointer_y_rejects_invalid_inputs() {
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
    assert_eq!(
        display_row_for_pointer_y(bounds, Px(10.0), Px(1.0), 0),
        None
    );
    assert_eq!(display_row_for_pointer_y(bounds, Px(0.0), Px(1.0), 2), None);
}
