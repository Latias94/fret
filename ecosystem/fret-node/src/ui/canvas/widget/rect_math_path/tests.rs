use super::*;

#[test]
fn path_bounds_rect_includes_close_back_to_subpath_start() {
    let bounds = path_bounds_rect(&[
        PathCommand::MoveTo(Point::new(Px(10.0), Px(20.0))),
        PathCommand::LineTo(Point::new(Px(30.0), Px(25.0))),
        PathCommand::Close,
    ])
    .expect("expected bounds");

    assert_eq!(
        bounds,
        Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(20.0), Px(5.0)))
    );
}
