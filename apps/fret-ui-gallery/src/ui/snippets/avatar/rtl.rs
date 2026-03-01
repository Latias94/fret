// region: example
use fret_core::ImageId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(cx, |cx| {
        vec![fret_ui_kit::primitives::direction::with_direction_provider(
            cx,
            fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
            |cx| {
                let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
                let fallback = shadcn::AvatarFallback::new("CN")
                    .when_image_missing_model(avatar_image.clone())
                    .delay_ms(120)
                    .into_element(cx);
                shadcn::Avatar::new([image, fallback])
                    .size(shadcn::AvatarSize::Default)
                    .into_element(cx)
                    .test_id("ui-gallery-avatar-rtl")
            },
        )]
    })
    .gap(Space::N4)
    .wrap()
    .w_full()
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-avatar-rtl-row")
}
// endregion: example

