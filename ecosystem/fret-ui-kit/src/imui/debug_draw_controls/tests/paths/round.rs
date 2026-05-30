use super::*;

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
