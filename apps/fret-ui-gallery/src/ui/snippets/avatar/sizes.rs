pub const SOURCE: &str = include_str!("sizes.rs");

// region: example
use fret_core::ImageId;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn wrap_row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N4)
        .wrap()
        .w_full()
        .items_center()
}

fn avatar_with_image<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
    size: shadcn::AvatarSize,
    fallback_text: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::avatar_sized(cx, size, |cx| {
        let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
        let fallback = shadcn::AvatarFallback::new(fallback_text)
            .when_image_missing_model(avatar_image)
            .delay_ms(120)
            .into_element(cx);
        [image, fallback]
    })
    .into_element(cx)
    .test_id(test_id)
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
) -> AnyElement {
    wrap_row(|cx| {
        vec![
            avatar_with_image(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Sm,
                "CN",
                "ui-gallery-avatar-sizes-sm",
            )
            .into_element(cx),
            avatar_with_image(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Default,
                "CN",
                "ui-gallery-avatar-sizes-default",
            )
            .into_element(cx),
            avatar_with_image(
                cx,
                avatar_image,
                shadcn::AvatarSize::Lg,
                "CN",
                "ui-gallery-avatar-sizes-lg",
            )
            .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-avatar-sizes")
}
// endregion: example
