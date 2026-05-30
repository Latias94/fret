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
