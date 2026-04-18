pub const SOURCE: &str = include_str!("sizes.rs");

// region: example
use super::demo_image;
use fret::{AppComponentCx, UiChild};
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
    avatar_image: Option<ImageId>,
    size: shadcn::AvatarSize,
    fallback_text: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    shadcn::avatar_sized(cx, size, |cx| {
        let image = shadcn::AvatarImage::maybe(avatar_image).into_element(cx);
        let fallback = shadcn::AvatarFallback::new(fallback_text)
            .when_image_missing(avatar_image)
            .delay_ms(120)
            .into_element(cx);
        [image, fallback]
    })
    .into_element(cx)
    .test_id(test_id)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let avatar_image = demo_image(cx);

    wrap_row(|cx| {
        vec![
            avatar_with_image(
                cx,
                avatar_image,
                shadcn::AvatarSize::Sm,
                "CN",
                "ui-gallery-avatar-sizes-sm",
            )
            .into_element(cx),
            avatar_with_image(
                cx,
                avatar_image,
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
