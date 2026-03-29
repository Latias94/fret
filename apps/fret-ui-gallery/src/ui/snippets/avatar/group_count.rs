pub const SOURCE: &str = include_str!("group_count.rs");

// region: example
use super::demo_image;
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

fn group_with_count<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Option<ImageId>,
    size: shadcn::AvatarSize,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let avatars = (0..3)
        .map(|idx| {
            avatar_with_image(cx, avatar_image, size, ["CN", "ML", "ER"][idx]).into_element(cx)
        })
        .collect::<Vec<_>>();

    let count = shadcn::AvatarGroupCount::empty()
        .children([ui::text("+3").font_medium().nowrap().into_element(cx)])
        .into_element(cx);

    shadcn::AvatarGroup::empty()
        .children(avatars.into_iter().chain([count]).collect::<Vec<_>>())
        .size(size)
        .into_element(cx)
        .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar_image = demo_image(cx);

    ui::v_flex(move |cx| {
        vec![
            group_with_count(
                cx,
                avatar_image,
                shadcn::AvatarSize::Default,
                "ui-gallery-avatar-group-count-default",
            )
            .into_element(cx),
        ]
    })
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-avatar-group-count")
}
// endregion: example
