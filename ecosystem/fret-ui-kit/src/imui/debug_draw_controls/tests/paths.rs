use super::*;

#[test]
fn rect_path_closes_clockwise_edges() {
    let path = rect_path(Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(30.0), Px(40.0)),
    ));

    assert_eq!(
        path,
        [
            PathCommand::MoveTo(Point::new(Px(10.0), Px(20.0))),
            PathCommand::LineTo(Point::new(Px(40.0), Px(20.0))),
            PathCommand::LineTo(Point::new(Px(40.0), Px(60.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(60.0))),
            PathCommand::Close,
        ]
    );
}

#[test]
fn polyline_path_requires_enough_points_and_closes_when_requested() {
    assert!(polyline_path(&[Point::new(Px(0.0), Px(0.0))], false).is_none());
    assert!(
        polyline_path(
            &[Point::new(Px(0.0), Px(0.0)), Point::new(Px(1.0), Px(1.0))],
            true,
        )
        .is_none()
    );

    let path = polyline_path(
        &[
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(10.0), Px(0.0)),
            Point::new(Px(10.0), Px(10.0)),
        ],
        true,
    )
    .unwrap();

    assert_eq!(
        path,
        vec![
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(10.0))),
            PathCommand::Close,
        ]
    );
}

#[test]
fn convex_poly_fill_path_requires_three_points_and_closes() {
    assert!(convex_poly_fill_path(&[Point::new(Px(0.0), Px(0.0))]).is_none());
    assert!(
        convex_poly_fill_path(&[Point::new(Px(0.0), Px(0.0)), Point::new(Px(10.0), Px(0.0)),])
            .is_none()
    );

    let path = convex_poly_fill_path(&[
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(10.0), Px(0.0)),
        Point::new(Px(12.0), Px(8.0)),
        Point::new(Px(2.0), Px(10.0)),
    ])
    .unwrap();

    assert_eq!(
        path,
        vec![
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(12.0), Px(8.0))),
            PathCommand::LineTo(Point::new(Px(2.0), Px(10.0))),
            PathCommand::Close,
        ]
    );
}

#[test]
fn concave_poly_fill_path_requires_three_points_and_closes() {
    assert!(concave_poly_fill_path(&[Point::new(Px(0.0), Px(0.0))]).is_none());
    assert!(
        concave_poly_fill_path(&[Point::new(Px(0.0), Px(0.0)), Point::new(Px(10.0), Px(0.0)),])
            .is_none()
    );

    let path = concave_poly_fill_path(&[
        Point::new(Px(0.0), Px(0.0)),
        Point::new(Px(18.0), Px(0.0)),
        Point::new(Px(10.0), Px(8.0)),
        Point::new(Px(18.0), Px(16.0)),
        Point::new(Px(0.0), Px(16.0)),
    ])
    .unwrap();

    assert_eq!(
        path,
        vec![
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(18.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(8.0))),
            PathCommand::LineTo(Point::new(Px(18.0), Px(16.0))),
            PathCommand::LineTo(Point::new(Px(0.0), Px(16.0))),
            PathCommand::Close,
        ]
    );
}

#[test]
fn triangle_path_closes_and_degenerate_triangles_are_detected() {
    let p1 = Point::new(Px(0.0), Px(0.0));
    let p2 = Point::new(Px(10.0), Px(0.0));
    let p3 = Point::new(Px(5.0), Px(8.0));

    assert_eq!(
        triangle_path(p1, p2, p3),
        [
            PathCommand::MoveTo(p1),
            PathCommand::LineTo(p2),
            PathCommand::LineTo(p3),
            PathCommand::Close,
        ]
    );
    assert!(!triangle_is_degenerate(p1, p2, p3));
    assert!(triangle_is_degenerate(p1, Point::new(Px(5.0), Px(0.0)), p2));
}

#[test]
fn quad_path_closes_four_ordered_points() {
    let p1 = Point::new(Px(0.0), Px(0.0));
    let p2 = Point::new(Px(10.0), Px(2.0));
    let p3 = Point::new(Px(12.0), Px(12.0));
    let p4 = Point::new(Px(2.0), Px(10.0));

    assert_eq!(
        quad_path(p1, p2, p3, p4),
        [
            PathCommand::MoveTo(p1),
            PathCommand::LineTo(p2),
            PathCommand::LineTo(p3),
            PathCommand::LineTo(p4),
            PathCommand::Close,
        ]
    );
}

#[test]
fn circle_path_uses_four_cubic_arcs_and_closes() {
    let path = circle_path(Point::new(Px(10.0), Px(20.0)), Px(8.0));

    assert_eq!(path.len(), 6);
    assert_eq!(path[0], PathCommand::MoveTo(Point::new(Px(18.0), Px(20.0))));
    assert!(matches!(path[1], PathCommand::CubicTo { .. }));
    assert!(matches!(path[2], PathCommand::CubicTo { .. }));
    assert!(matches!(path[3], PathCommand::CubicTo { .. }));
    assert!(matches!(path[4], PathCommand::CubicTo { .. }));
    assert_eq!(path[5], PathCommand::Close);
}

#[test]
fn ngon_path_requires_three_segments_and_positive_radius() {
    assert!(ngon_path(Point::new(Px(0.0), Px(0.0)), Px(8.0), 2).is_none());
    assert!(ngon_path(Point::new(Px(0.0), Px(0.0)), Px(0.0), 3).is_none());

    let path = ngon_path(Point::new(Px(10.0), Px(20.0)), Px(8.0), 4).unwrap();

    assert_eq!(path.len(), 5);
    assert_eq!(path[0], PathCommand::MoveTo(Point::new(Px(18.0), Px(20.0))));
    assert!(matches!(path[1], PathCommand::LineTo(_)));
    assert!(matches!(path[2], PathCommand::LineTo(_)));
    assert!(matches!(path[3], PathCommand::LineTo(_)));
    assert_eq!(path[4], PathCommand::Close);
}

#[test]
fn ellipse_path_defaults_segments_and_supports_rotation() {
    assert!(
        ellipse_path(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(8.0), Px(4.0)),
            0.0,
            2
        )
        .is_none()
    );
    assert!(
        ellipse_path(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(0.0), Px(4.0)),
            0.0,
            4
        )
        .is_none()
    );
    assert!(
        ellipse_path(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(8.0), Px(4.0)),
            f32::NAN,
            4,
        )
        .is_none()
    );

    let default_path = ellipse_path(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(8.0), Px(4.0)),
        0.0,
        0,
    )
    .unwrap();
    assert_eq!(default_path.len(), DEFAULT_ELLIPSE_SEGMENTS + 1);
    assert_eq!(
        default_path[0],
        PathCommand::MoveTo(Point::new(Px(18.0), Px(20.0)))
    );
    assert_eq!(default_path[DEFAULT_ELLIPSE_SEGMENTS], PathCommand::Close);

    let rotated_path = ellipse_path(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(8.0), Px(4.0)),
        std::f32::consts::FRAC_PI_2,
        4,
    )
    .unwrap();
    let PathCommand::MoveTo(point) = &rotated_path[0] else {
        panic!("rotated ellipse should start with a MoveTo");
    };
    assert!((point.x.0 - 10.0).abs() <= 0.000_1);
    assert!((point.y.0 - 28.0).abs() <= 0.000_1);
    assert_eq!(rotated_path[4], PathCommand::Close);
}

#[test]
fn bezier_paths_use_native_quad_and_cubic_commands() {
    let from = Point::new(Px(0.0), Px(0.0));
    let ctrl = Point::new(Px(10.0), Px(20.0));
    let ctrl1 = Point::new(Px(8.0), Px(16.0));
    let ctrl2 = Point::new(Px(18.0), Px(-6.0));
    let to = Point::new(Px(24.0), Px(0.0));

    assert_eq!(
        bezier_quadratic_path(from, ctrl, to),
        [PathCommand::MoveTo(from), PathCommand::QuadTo { ctrl, to }]
    );
    assert_eq!(
        bezier_cubic_path(from, ctrl1, ctrl2, to),
        [
            PathCommand::MoveTo(from),
            PathCommand::CubicTo { ctrl1, ctrl2, to },
        ]
    );
}
