pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use super::demo_image;
use fret::{UiChild, UiCx};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar_image = demo_image(cx);

    fret_ui_kit::ui::h_flex(|cx| {
        vec![with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
            let image = shadcn::AvatarImage::maybe(avatar_image).into_element(cx);
            let fallback = shadcn::AvatarFallback::new("CN")
                .when_image_missing(avatar_image)
                .delay_ms(120)
                .into_element(cx);
            shadcn::Avatar::new([image, fallback])
                .size(shadcn::AvatarSize::Default)
                .into_element(cx)
                .test_id("ui-gallery-avatar-rtl")
        })]
    })
    .gap(Space::N4)
    .wrap()
    .w_full()
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-avatar-rtl-row")
}
// endregion: example
