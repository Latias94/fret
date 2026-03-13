use super::*;

fn point(x: f32, y: f32) -> Point {
    Point::new(Px(x), Px(y))
}

#[test]
fn resolve_activation_point_prefers_port_center() {
    let port_center = Some(point(10.0, 20.0));
    let last_pos = Some(point(30.0, 40.0));
    let last_bounds = Some(Rect::new(
        point(100.0, 200.0),
        fret_core::Size::new(Px(50.0), Px(60.0)),
    ));

    let resolved = resolve_activation_point(port_center, last_pos, last_bounds);

    assert_eq!(resolved, point(10.0, 20.0));
}

#[test]
fn resolve_activation_point_falls_back_to_last_pos() {
    let last_pos = Some(point(30.0, 40.0));
    let last_bounds = Some(Rect::new(
        point(100.0, 200.0),
        fret_core::Size::new(Px(50.0), Px(60.0)),
    ));

    let resolved = resolve_activation_point(None, last_pos, last_bounds);

    assert_eq!(resolved, point(30.0, 40.0));
}

#[test]
fn resolve_activation_point_falls_back_to_bounds_center() {
    let last_bounds = Some(Rect::new(
        point(100.0, 200.0),
        fret_core::Size::new(Px(50.0), Px(60.0)),
    ));

    let resolved = resolve_activation_point(None, None, last_bounds);

    assert_eq!(resolved, point(125.0, 230.0));
}
