pub const SOURCE: &str = include_str!("sizes.rs");

// region: example
use fret_core::ImageId;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(cx, children)
        .gap(Space::N4)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}

fn avatar_with_image<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
    size: shadcn::AvatarSize,
    fallback_text: &'static str,
    test_id: &'static str,
) -> AnyElement {
    let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
    let fallback = shadcn::AvatarFallback::new(fallback_text)
        .when_image_missing_model(avatar_image)
        .delay_ms(120)
        .into_element(cx);

    shadcn::Avatar::new([image, fallback])
        .size(size)
        .into_element(cx)
        .test_id(test_id)
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
) -> AnyElement {
    wrap_row(cx, |cx| {
        vec![
            avatar_with_image(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Sm,
                "CN",
                "ui-gallery-avatar-sizes-sm",
            ),
            avatar_with_image(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Default,
                "CN",
                "ui-gallery-avatar-sizes-default",
            ),
            avatar_with_image(
                cx,
                avatar_image,
                shadcn::AvatarSize::Lg,
                "CN",
                "ui-gallery-avatar-sizes-lg",
            ),
        ]
    })
    .test_id("ui-gallery-avatar-sizes")
}
// endregion: example
