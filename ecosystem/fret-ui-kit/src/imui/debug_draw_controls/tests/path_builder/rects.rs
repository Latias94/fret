use super::*;

#[test]
fn debug_draw_path_builder_appends_rect_points() {
    let mut list = ImUiDebugDrawList::default();
    let rect = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(20.0), Px(10.0)),
    );

    list.path(|path| {
        path.rect(rect);
        assert_eq!(path.point_count(), 4);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), true);
    });

    let DebugDrawCommand::Polyline { points, closed, .. } = &list.commands[0] else {
        panic!("path rect helper should record a closed polyline command");
    };
    assert!(*closed);
    assert_eq!(points.len(), 4);
    assert_eq!(
        &**points,
        &[
            Point::new(Px(10.0), Px(20.0)),
            Point::new(Px(30.0), Px(20.0)),
            Point::new(Px(30.0), Px(30.0)),
            Point::new(Px(10.0), Px(30.0)),
        ]
    );
}

#[test]
fn debug_draw_path_builder_appends_rounded_rect_corner_samples() {
    let mut list = ImUiDebugDrawList::default();
    let rect = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(20.0), Px(10.0)),
    );

    list.path(|path| {
        path.rect_with_rounding(
            rect,
            Px(4.0),
            DebugDrawRoundCorners::TOP_LEFT | DebugDrawRoundCorners::BOTTOM_RIGHT,
        );
        assert_eq!(path.point_count(), 10);
        path.fill_convex(Color::from_srgb_hex_rgb(0xff_ff_ff));
    });

    let DebugDrawCommand::ConvexPolyFilled { points, .. } = &list.commands[0] else {
        panic!("rounded path rect helper should record sampled convex fill points");
    };
    assert_eq!(points.len(), 10);
    assert_point_near(points[0], Point::new(Px(10.0), Px(24.0)));
    assert_point_near(points[3], Point::new(Px(14.0), Px(20.0)));
    assert_point_near(points[4], Point::new(Px(30.0), Px(20.0)));
    assert_point_near(points[5], Point::new(Px(30.0), Px(26.0)));
    assert_point_near(points[8], Point::new(Px(26.0), Px(30.0)));
    assert_point_near(points[9], Point::new(Px(10.0), Px(30.0)));
}

#[test]
fn debug_draw_path_builder_rect_rounding_clamps_and_handles_invalid_inputs() {
    let mut list = ImUiDebugDrawList::default();
    let rect = Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(12.0), Px(8.0)));

    list.path(|path| {
        path.rect_with_rounding(rect, Px(50.0), DebugDrawRoundCorners::ALL);
        assert_eq!(path.point_count(), 16);
        assert_point_near(path.points[0], Point::new(Px(10.0), Px(23.0)));
        path.clear();

        path.rect_with_rounding(rect, Px(4.0), DebugDrawRoundCorners::NONE);
        assert_eq!(path.point_count(), 4);
        assert_eq!(path.points[0], Point::new(Px(10.0), Px(20.0)));
        assert_eq!(path.points[2], Point::new(Px(22.0), Px(28.0)));
        path.clear();

        path.rect_with_rounding(
            Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(0.0), Px(8.0))),
            Px(4.0),
            DebugDrawRoundCorners::ALL,
        );
        path.rect_with_rounding(rect, Px(f32::NAN), DebugDrawRoundCorners::ALL);
        assert!(path.is_empty());
    });

    assert_eq!(list.command_count(), 0);
}
