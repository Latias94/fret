use super::*;

#[test]
fn rect_from_points_orders_corners() {
    let rect = rect_from_points(
        Point::new(Px(30.0), Px(10.0)),
        Point::new(Px(5.0), Px(25.0)),
    );
    assert_eq!(
        rect,
        Rect::new(Point::new(Px(5.0), Px(10.0)), Size::new(Px(25.0), Px(15.0)))
    );
}

#[test]
fn rects_intersect_counts_touching_edges_as_intersection() {
    let a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
    let b = Rect::new(Point::new(Px(10.0), Px(5.0)), Size::new(Px(4.0), Px(6.0)));
    assert!(rects_intersect(a, b));
}
