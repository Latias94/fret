use super::*;

#[test]
fn rect_contains_includes_rect_edges() {
    let rect = Rect::new(
        Point::new(Px(10.0), Px(20.0)),
        Size::new(Px(30.0), Px(40.0)),
    );
    assert!(rect_contains(rect, Point::new(Px(10.0), Px(20.0))));
    assert!(rect_contains(rect, Point::new(Px(40.0), Px(60.0))));
    assert!(!rect_contains(rect, Point::new(Px(40.1), Px(60.0))));
}
