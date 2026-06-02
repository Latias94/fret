use super::*;

#[test]
fn image_item_helpers_sanitize_size_opacity_and_uv() {
    let size = sanitize_item_size(Size::new(Px(-8.0), Px(f32::NAN)));
    assert_eq!(size, Size::new(Px(0.0), Px(0.0)));

    assert_eq!(normalize_opacity(-1.0), 0.0);
    assert_eq!(normalize_opacity(2.0), 1.0);
    assert_eq!(normalize_opacity(f32::NAN), 1.0);

    assert!(uv_rect_is_valid(UvRect::FULL));
    assert!(!uv_rect_is_valid(UvRect {
        u0: 0.8,
        v0: 0.0,
        u1: 0.2,
        v1: 1.0,
    }));
}
