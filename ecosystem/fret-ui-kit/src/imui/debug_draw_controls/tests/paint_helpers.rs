use super::*;

#[test]
fn image_overlay_helpers_sanitize_opacity_and_uv_rects() {
    assert_eq!(normalized_opacity(-1.0), 0.0);
    assert_eq!(normalized_opacity(2.0), 1.0);
    assert_eq!(normalized_opacity(f32::NAN), 1.0);

    assert!(uv_rect_is_valid(UvRect::FULL));
    assert!(!uv_rect_is_valid(UvRect {
        u0: 0.5,
        v0: 0.0,
        u1: 0.25,
        v1: 1.0,
    }));
}

#[test]
fn rounded_image_helpers_follow_imgui_path_rect_corner_rules() {
    let rect = Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(12.0), Px(8.0)));

    let all = rounded_rect_corner_radii(rect, Px(50.0), DebugDrawRoundCorners::ALL);
    assert_eq!(all, Corners::all(Px(3.0)));
    assert!(corner_radii_are_visible(all));

    let diagonal = rounded_rect_corner_radii(
        rect,
        Px(50.0),
        DebugDrawRoundCorners::TOP_LEFT | DebugDrawRoundCorners::BOTTOM_RIGHT,
    );
    assert_eq!(diagonal.top_left, Px(7.0));
    assert_eq!(diagonal.top_right, Px(0.0));
    assert_eq!(diagonal.bottom_right, Px(7.0));
    assert_eq!(diagonal.bottom_left, Px(0.0));

    assert_eq!(
        rounded_rect_corner_radii(rect, Px(4.0), DebugDrawRoundCorners::NONE),
        Corners::all(Px(0.0))
    );
    assert_eq!(
        rounded_rect_corner_radii(rect, Px(f32::NAN), DebugDrawRoundCorners::ALL),
        Corners::all(Px(0.0))
    );
}
