use super::*;

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
