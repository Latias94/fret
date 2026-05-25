use fret_core::scene::{ImageSamplingHint, UvRect};
use fret_core::{ImageId, Px, Size, ViewportFit};
use fret_ui::element::Length;

use super::{image_props_for_item, normalize_opacity, sanitize_item_size, uv_rect_is_valid};

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

#[test]
fn image_props_fill_the_interactive_item_box() {
    let props = image_props_for_item(
        ImageId::default(),
        ViewportFit::Contain,
        ImageSamplingHint::Nearest,
        0.5,
        Some(UvRect::FULL),
    );

    assert_eq!(props.layout.size.width, Length::Fill);
    assert_eq!(props.layout.size.height, Length::Fill);
    assert_eq!(props.fit, ViewportFit::Contain);
    assert_eq!(props.sampling, ImageSamplingHint::Nearest);
    assert_eq!(props.opacity, 0.5);
    assert_eq!(props.uv, Some(UvRect::FULL));
}
