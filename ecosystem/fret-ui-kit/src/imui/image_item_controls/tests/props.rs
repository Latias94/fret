use super::*;

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
