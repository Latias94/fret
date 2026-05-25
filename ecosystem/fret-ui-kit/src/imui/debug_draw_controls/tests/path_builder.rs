use super::*;

#[test]
fn debug_draw_path_builder_records_stroke_and_fill_commands() {
    let mut list = ImUiDebugDrawList::default();
    let p0 = Point::new(Px(0.0), Px(0.0));
    let p1 = Point::new(Px(12.0), Px(0.0));
    let p2 = Point::new(Px(12.0), Px(10.0));
    let p3 = Point::new(Px(0.0), Px(10.0));

    list.path(|path| {
        assert!(path.is_empty());
        path.line_to(p0)
            .line_to_merge_duplicate(p0)
            .line_to_merge_duplicate(p1)
            .line_to(p2);
        assert_eq!(path.point_count(), 3);

        path.stroke_with_style(
            Color::from_srgb_hex_rgb(0xff_aa_00),
            DebugDrawStrokeStyle::new(Px(2.0)).with_join(StrokeJoinV1::Round),
            true,
        );
        assert!(path.is_empty());

        path.line_to(p0).line_to(p1).line_to(p2).line_to(p3);
        path.fill_convex(Color::from_srgb_hex_rgb(0x22_c5_5e));
    });

    assert_eq!(list.command_count(), 2);
    let DebugDrawCommand::Polyline {
        points,
        style,
        closed,
        ..
    } = &list.commands[0]
    else {
        panic!("path stroke should record a polyline command");
    };
    assert_eq!(&**points, &[p0, p1, p2]);
    assert_eq!(style.width, Px(2.0));
    assert_eq!(style.join, StrokeJoinV1::Round);
    assert!(*closed);

    let DebugDrawCommand::ConvexPolyFilled { points, .. } = &list.commands[1] else {
        panic!("path fill should record a convex fill command");
    };
    assert_eq!(&**points, &[p0, p1, p2, p3]);
}

#[test]
fn debug_draw_path_builder_records_concave_fill_command() {
    let mut list = ImUiDebugDrawList::default();
    let points = [
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(18.0), Px(0.0)),
        Point::new(Px(10.0), Px(8.0)),
        Point::new(Px(18.0), Px(16.0)),
        Point::new(Px(0.0), Px(16.0)),
    ];

    list.path(|path| {
        path.line_to(points[0])
            .line_to(points[1])
            .line_to(points[2])
            .line_to(points[3])
            .line_to(points[4]);
        path.fill_concave(Color::from_srgb_hex_rgb(0xff_ff_ff));
        assert!(path.is_empty());

        path.line_to(points[0]).line_to(points[1]);
        path.fill_concave(Color::from_srgb_hex_rgb(0xff_ff_ff));
        assert!(path.is_empty());
    });

    assert_eq!(list.command_count(), 1);
    let DebugDrawCommand::ConcavePolyFilled {
        points: recorded, ..
    } = &list.commands[0]
    else {
        panic!("path concave fill should record a dedicated command");
    };
    assert_eq!(&**recorded, &points);
}

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

    let DebugDrawCommand::Polyline { points, .. } = &list.commands[0] else {
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

#[test]
fn debug_draw_path_builder_appends_arc_samples() {
    let mut list = ImUiDebugDrawList::default();
    let center = Point::new(Px(10.0), Px(20.0));

    list.path(|path| {
        path.arc_to(center, Px(8.0), 0.0, std::f32::consts::PI, 2);
        assert_eq!(path.point_count(), 3);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
    });

    let DebugDrawCommand::Polyline { points, .. } = &list.commands[0] else {
        panic!("path arc helper should record a sampled polyline command");
    };
    assert_eq!(points.len(), 3);
    assert_point_near(points[0], Point::new(Px(18.0), Px(20.0)));
    assert_point_near(points[1], Point::new(Px(10.0), Px(28.0)));
    assert_point_near(points[2], Point::new(Px(2.0), Px(20.0)));
}

#[test]
fn debug_draw_path_builder_arc_helpers_handle_fast_default_and_degenerate_inputs() {
    let mut list = ImUiDebugDrawList::default();
    let center = Point::new(Px(10.0), Px(20.0));

    list.path(|path| {
        path.arc_to(center, Px(0.25), 0.0, std::f32::consts::PI, 4);
        assert_eq!(path.point_count(), 1);
        assert_eq!(path.clear().point_count(), 0);

        path.arc_to(center, Px(8.0), f32::NAN, std::f32::consts::PI, 4);
        path.arc_to(center, Px(0.0), 0.0, std::f32::consts::PI, 4);
        assert!(path.is_empty());

        path.arc_to(center, Px(8.0), 0.0, std::f32::consts::FRAC_PI_2, 0);
        assert_eq!(path.point_count(), DEFAULT_PATH_ARC_SEGMENTS + 1);
        path.clear();

        path.arc_to_fast(center, Px(8.0), 0, 3);
        assert_eq!(path.point_count(), 4);
        path.clear();

        path.arc_to_fast(center, Px(8.0), 3, 0);
        assert_eq!(path.point_count(), 4);
        path.clear();
    });

    assert_eq!(list.command_count(), 0);
}

#[test]
fn debug_draw_path_builder_appends_elliptical_arc_samples() {
    let mut list = ImUiDebugDrawList::default();
    let center = Point::new(Px(10.0), Px(20.0));

    list.path(|path| {
        path.elliptical_arc_to(
            center,
            Size::new(Px(8.0), Px(4.0)),
            0.0,
            0.0,
            std::f32::consts::PI,
            2,
        );
        assert_eq!(path.point_count(), 3);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
    });

    let DebugDrawCommand::Polyline { points, .. } = &list.commands[0] else {
        panic!("path elliptical arc helper should record a sampled polyline command");
    };
    assert_eq!(points.len(), 3);
    assert_point_near(points[0], Point::new(Px(18.0), Px(20.0)));
    assert_point_near(points[1], Point::new(Px(10.0), Px(24.0)));
    assert_point_near(points[2], Point::new(Px(2.0), Px(20.0)));
}

#[test]
fn debug_draw_path_builder_elliptical_arc_handles_rotation_default_and_invalid_inputs() {
    let mut list = ImUiDebugDrawList::default();
    let center = Point::new(Px(10.0), Px(20.0));

    list.path(|path| {
        path.elliptical_arc_to(
            center,
            Size::new(Px(8.0), Px(4.0)),
            std::f32::consts::FRAC_PI_2,
            0.0,
            std::f32::consts::PI,
            2,
        );
        assert_eq!(path.point_count(), 3);
        assert_point_near(path.points[0], Point::new(Px(10.0), Px(28.0)));
        assert_point_near(path.points[1], Point::new(Px(6.0), Px(20.0)));
        assert_point_near(path.points[2], Point::new(Px(10.0), Px(12.0)));
        path.clear();

        path.elliptical_arc_to(
            center,
            Size::new(Px(8.0), Px(4.0)),
            0.0,
            0.0,
            std::f32::consts::FRAC_PI_2,
            0,
        );
        assert_eq!(path.point_count(), DEFAULT_PATH_ELLIPTICAL_ARC_SEGMENTS + 1);
        path.clear();

        path.elliptical_arc_to(
            center,
            Size::new(Px(0.0), Px(4.0)),
            0.0,
            0.0,
            std::f32::consts::PI,
            2,
        );
        path.elliptical_arc_to(
            center,
            Size::new(Px(8.0), Px(4.0)),
            f32::NAN,
            0.0,
            std::f32::consts::PI,
            2,
        );
        path.elliptical_arc_to(
            center,
            Size::new(Px(8.0), Px(4.0)),
            0.0,
            f32::NAN,
            std::f32::consts::PI,
            2,
        );
        assert!(path.is_empty());
    });

    assert_eq!(list.command_count(), 0);
}

#[test]
fn debug_draw_path_builder_clears_invalid_finished_paths_without_recording() {
    let mut list = ImUiDebugDrawList::default();
    let p0 = Point::new(Px(0.0), Px(0.0));
    let p1 = Point::new(Px(8.0), Px(0.0));

    list.path(|path| {
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
        assert!(path.is_empty());

        path.line_to(p0);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), false);
        assert!(path.is_empty());

        path.line_to(p0).line_to(p1);
        path.stroke(Color::from_srgb_hex_rgb(0xff_ff_ff), Px(1.0), true);
        assert!(path.is_empty());

        path.line_to(p0).line_to(p1);
        path.fill_convex(Color::from_srgb_hex_rgb(0xff_ff_ff));
        assert!(path.is_empty());

        path.line_to(p0).line_to(p1);
        assert_eq!(path.point_count(), 2);
        path.clear();
        assert!(path.is_empty());
    });

    assert_eq!(list.command_count(), 0);
}
