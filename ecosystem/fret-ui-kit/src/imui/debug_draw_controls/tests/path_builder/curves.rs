use super::*;

#[test]
fn debug_draw_path_builder_appends_bezier_curve_samples() {
    let mut list = ImUiDebugDrawList::default();
    let start = Point::new(Px(0.0), Px(0.0));
    let quad_mid = Point::new(Px(10.0), Px(5.0));
    let quad_end = Point::new(Px(20.0), Px(0.0));
    let cubic_mid = Point::new(Px(30.0), Px(5.0));
    let cubic_end = Point::new(Px(40.0), Px(10.0));

    list.path(|path| {
        path.line_to(start)
            .bezier_quadratic_curve_to(Point::new(Px(10.0), Px(10.0)), quad_end, 2)
            .bezier_cubic_curve_to(
                Point::new(Px(30.0), Px(0.0)),
                Point::new(Px(30.0), Px(10.0)),
                cubic_end,
                2,
            );
        assert_eq!(path.point_count(), 5);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
    });

    let DebugDrawCommand::Linear(DebugDrawLinearCommand::Polyline { points, .. }) =
        &list.commands[0]
    else {
        panic!("path Bezier helpers should record a sampled polyline command");
    };
    assert_eq!(
        &**points,
        &[start, quad_mid, quad_end, cubic_mid, cubic_end]
    );
}

#[test]
fn debug_draw_path_builder_bezier_helpers_require_a_start_point_and_default_segments() {
    let mut list = ImUiDebugDrawList::default();
    let start = Point::new(Px(0.0), Px(0.0));
    let ctrl = Point::new(Px(10.0), Px(10.0));
    let end = Point::new(Px(20.0), Px(0.0));

    list.path(|path| {
        path.bezier_quadratic_curve_to(ctrl, end, 2);
        assert!(path.is_empty());

        path.line_to(start).bezier_quadratic_curve_to(ctrl, end, 0);
        assert_eq!(path.point_count(), DEFAULT_PATH_BEZIER_SEGMENTS + 1);
        path.clear();

        path.bezier_cubic_curve_to(ctrl, ctrl, end, 2);
        assert!(path.is_empty());
    });

    assert_eq!(list.command_count(), 0);
}
