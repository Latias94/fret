pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use super::demo_image;
use fret::{AppComponentCx, UiChild};
use fret_ui_shadcn::facade as shadcn;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let avatar_image = demo_image(cx);
    let image = shadcn::AvatarImage::maybe(avatar_image).into_element(cx);
    let fallback = shadcn::AvatarFallback::new("CN")
        .when_image_missing(avatar_image)
        .delay_ms(120)
        .into_element(cx);

    shadcn::Avatar::new([image, fallback])
        .size(shadcn::AvatarSize::Default)
        .into_element(cx)
        .test_id("ui-gallery-avatar-basic")
}
// endregion: example
