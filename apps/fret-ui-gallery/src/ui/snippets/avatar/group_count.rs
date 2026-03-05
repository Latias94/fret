pub const SOURCE: &str = include_str!("group_count.rs");

// region: example
use fret_core::{ImageId, Px};
use fret_ui::Theme;
use fret_ui_kit::ColorRef;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    name: &'static str,
    size: Px,
    fg: ColorRef,
) -> AnyElement {
    shadcn::icon::icon_with(
        cx,
        fret_icons::IconId::new_static(name),
        Some(size),
        Some(fg),
    )
}

fn avatar_with_image<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
    size: shadcn::AvatarSize,
    fallback_text: &'static str,
) -> AnyElement {
    let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
    let fallback = shadcn::AvatarFallback::new(fallback_text)
        .when_image_missing_model(avatar_image)
        .delay_ms(120)
        .into_element(cx);
    shadcn::Avatar::new([image, fallback])
        .size(size)
        .into_element(cx)
}

fn group_with_count<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
    size: shadcn::AvatarSize,
    test_id: &'static str,
) -> AnyElement {
    let avatars = (0..3)
        .map(|idx| avatar_with_image(cx, avatar_image.clone(), size, ["CN", "ML", "ER"][idx]))
        .collect::<Vec<_>>();

    let count =
        shadcn::AvatarGroupCount::new([ui::text("+3").font_medium().nowrap().into_element(cx)])
            .into_element(cx);

    shadcn::AvatarGroup::new(avatars.into_iter().chain([count]).collect::<Vec<_>>())
        .size(size)
        .into_element(cx)
        .test_id(test_id)
}

fn group_with_icon_count<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
    size: shadcn::AvatarSize,
    test_id: &'static str,
) -> AnyElement {
    let avatars = (0..2)
        .map(|idx| avatar_with_image(cx, avatar_image.clone(), size, ["CN", "ML"][idx]))
        .collect::<Vec<_>>();

    let muted_fg = Theme::global(&*cx.app).color_token("muted-foreground");
    let fg = ColorRef::Color(muted_fg);
    let count =
        shadcn::AvatarGroupCount::new([icon(cx, "lucide.plus", Px(18.0), fg)]).into_element(cx);

    shadcn::AvatarGroup::new(avatars.into_iter().chain([count]).collect::<Vec<_>>())
        .size(size)
        .into_element(cx)
        .test_id(test_id)
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<ImageId>>,
) -> AnyElement {
    ui::v_flex(move |cx| {
        vec![
            group_with_count(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Sm,
                "ui-gallery-avatar-group-count-sm",
            ),
            group_with_count(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Default,
                "ui-gallery-avatar-group-count-default",
            ),
            group_with_count(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Lg,
                "ui-gallery-avatar-group-count-lg",
            ),
            group_with_icon_count(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Default,
                "ui-gallery-avatar-group-count-icon",
            ),
        ]
    })
    .gap(Space::N4)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-avatar-group-count")
}
// endregion: example
