use super::*;

#[test]
fn edge_center_canvas_matches_bezier_math() {
    let from = Point::new(Px(0.0), Px(0.0));
    let to = Point::new(Px(100.0), Px(0.0));
    let zoom = 1.0;
    let (c1, c2) = wire_ctrl_points(from, to, zoom);
    let expected = cubic_bezier(from, c1, c2, to, 0.5);
    let got = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_center_canvas(
        EdgeRouteKind::Bezier,
        from,
        to,
        zoom,
    );
    assert!((got.x.0 - expected.x.0).abs() <= 1.0e-6);
    assert!((got.y.0 - expected.y.0).abs() <= 1.0e-6);
}

#[test]
fn edge_center_canvas_step_uses_mid_x_and_mid_y() {
    let from = Point::new(Px(10.0), Px(50.0));
    let to = Point::new(Px(110.0), Px(150.0));
    let got = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_center_canvas(
        EdgeRouteKind::Step,
        from,
        to,
        1.0,
    );
    assert!((got.x.0 - 60.0).abs() <= 1.0e-6);
    assert!((got.y.0 - 100.0).abs() <= 1.0e-6);
}
