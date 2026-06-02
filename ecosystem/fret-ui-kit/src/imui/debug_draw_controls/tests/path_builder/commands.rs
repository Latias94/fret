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
