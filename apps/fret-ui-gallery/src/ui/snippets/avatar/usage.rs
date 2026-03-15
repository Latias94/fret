pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::ImageId;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
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
