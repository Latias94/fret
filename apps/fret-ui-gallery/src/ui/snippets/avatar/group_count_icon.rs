pub const SOURCE: &str = include_str!("group_count_icon.rs");

// region: example
use super::demo_image;
use fret::{UiChild, UiCx};
use fret_core::{ImageId, Px};
use fret_ui::Theme;
use fret_ui_kit::{ColorRef, IntoUiElement};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    name: &'static str,
    size: Px,
    fg: ColorRef,
) -> impl IntoUiElement<H> + use<H> {
    icon::icon_with(
        cx,
        fret_icons::IconId::new_static(name),
        Some(size),
        Some(fg),
    )
}

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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar_image = demo_image(cx);

    let avatars = (0..2)
        .map(|idx| {
            avatar_with_image(
                cx,
                avatar_image,
                shadcn::AvatarSize::Default,
                ["CN", "ML"][idx],
            )
            .into_element(cx)
        })
        .collect::<Vec<_>>();

    let muted_fg = Theme::global(&*cx.app).color_token("muted-foreground");
    let fg = ColorRef::Color(muted_fg);
    let count =
        shadcn::AvatarGroupCount::new([icon(cx, "lucide.plus", Px(18.0), fg).into_element(cx)])
            .into_element(cx);

    shadcn::AvatarGroup::new(avatars.into_iter().chain([count]).collect::<Vec<_>>())
        .size(shadcn::AvatarSize::Default)
        .into_element(cx)
        .test_id("ui-gallery-avatar-group-count-icon")
}
// endregion: example
