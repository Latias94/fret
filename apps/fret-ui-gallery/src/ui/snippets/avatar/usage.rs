pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret_core::ImageId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let image: Option<ImageId> = None;

    shadcn::Avatar::new(Vec::<AnyElement>::new())
        .children([
            shadcn::AvatarImage::maybe(image).into_element(cx),
            shadcn::AvatarFallback::new("CN")
                .when_image_missing(image)
                .delay_ms(120)
                .into_element(cx),
        ])
        .size(shadcn::AvatarSize::Default)
        .into_element(cx)
        .test_id("ui-gallery-avatar-usage")
}
// endregion: example
