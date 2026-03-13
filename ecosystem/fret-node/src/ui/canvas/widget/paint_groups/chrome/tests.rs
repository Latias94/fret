use super::*;

#[test]
fn group_chrome_scales_inverse_with_zoom() {
    assert_eq!(group_corner_radius(2.0), Px(5.0));
    assert_eq!(group_border_width(4.0), Px(0.25));
    assert_eq!(group_padding(5.0), 2.0);
}

#[test]
fn group_title_max_width_clamps_at_zero() {
    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(12.0), Px(20.0)));
    assert_eq!(group_title_max_width(rect, 10.0), 0.0);

    let rect = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(80.0), Px(20.0)));
    assert_eq!(group_title_max_width(rect, 10.0), 60.0);
}
