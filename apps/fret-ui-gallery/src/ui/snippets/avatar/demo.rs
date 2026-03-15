pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use crate::ui::snippets::avatar::demo_image;
use fret::{UiChild, UiCx};
use fret_core::ImageId;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn avatar_with_image<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Option<ImageId>,
    size: shadcn::AvatarSize,
    fallback_text: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let image = shadcn::AvatarImage::maybe(avatar_image).into_element(cx);
    let fallback = shadcn::AvatarFallback::new(fallback_text)
        .when_image_missing(avatar_image)
        .delay_ms(120)
        .into_element(cx);

    shadcn::Avatar::new([image, fallback])
        .size(size)
        .into_element(cx)
}

fn avatar_with_badge<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Option<ImageId>,
    fallback_text: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let image = shadcn::AvatarImage::maybe(avatar_image).into_element(cx);
    let fallback = shadcn::AvatarFallback::new(fallback_text)
        .when_image_missing(avatar_image)
        .delay_ms(120)
        .into_element(cx);
    let badge = shadcn::AvatarBadge::new().into_element(cx);

    shadcn::Avatar::new([image, fallback, badge])
        .size(shadcn::AvatarSize::Default)
        .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar_image = demo_image(cx);

    let group = {
        let avatars = ["CN", "ML", "ER"]
            .into_iter()
            .map(|fallback| {
                avatar_with_image(cx, avatar_image, shadcn::AvatarSize::Default, fallback)
                    .into_element(cx)
            })
            .collect::<Vec<_>>();
        let count =
            shadcn::AvatarGroupCount::new([ui::text("+3").font_medium().nowrap().into_element(cx)])
                .into_element(cx);

        shadcn::AvatarGroup::new(avatars.into_iter().chain([count]).collect::<Vec<_>>())
            .size(shadcn::AvatarSize::Default)
            .into_element(cx)
            .test_id("ui-gallery-avatar-demo-group")
    };

    fret_ui_kit::ui::h_flex(|cx| {
        vec![
            avatar_with_image(cx, avatar_image, shadcn::AvatarSize::Default, "CN")
                .into_element(cx)
                .test_id("ui-gallery-avatar-demo-basic"),
            avatar_with_badge(cx, avatar_image, "ER")
                .into_element(cx)
                .test_id("ui-gallery-avatar-demo-badge"),
            group,
        ]
    })
    .gap(Space::N12)
    .wrap()
    .w_full()
    .items_center()
    .into_element(cx)
    .test_id("ui-gallery-avatar-demo")
}
// endregion: example
