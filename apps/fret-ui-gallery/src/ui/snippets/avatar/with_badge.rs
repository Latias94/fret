pub const SOURCE: &str = include_str!("with_badge.rs");

// region: example
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_kit::ColorRef;
use fret_ui_shadcn::{self as shadcn, prelude::*};

fn wrap_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N4)
        .wrap()
        .w_full()
        .items_center()
        .into_element(cx)
}

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

fn avatar_with_badge<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<fret_core::ImageId>>,
    size: shadcn::AvatarSize,
    badge: shadcn::AvatarBadge,
    test_id: &'static str,
) -> AnyElement {
    let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
    let fallback = shadcn::AvatarFallback::new("CN")
        .when_image_missing_model(avatar_image)
        .delay_ms(120)
        .into_element(cx);
    let badge = badge.into_element(cx);

    shadcn::Avatar::new([image, fallback, badge])
        .size(size)
        .into_element(cx)
        .test_id(test_id)
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<fret_core::ImageId>>,
) -> AnyElement {
    let destructive = Theme::global(&*cx.app).color_token("destructive");
    let primary_fg = Theme::global(&*cx.app).color_token("primary-foreground");

    let dot_row = wrap_row(cx, |cx| {
        let custom_badge = shadcn::AvatarBadge::new()
            .refine_style(ChromeRefinement::default().bg(ColorRef::Color(destructive)));

        vec![
            avatar_with_badge(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Sm,
                shadcn::AvatarBadge::new(),
                "ui-gallery-avatar-badge-sm",
            ),
            avatar_with_badge(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Default,
                custom_badge,
                "ui-gallery-avatar-badge-default",
            ),
            avatar_with_badge(
                cx,
                avatar_image.clone(),
                shadcn::AvatarSize::Lg,
                shadcn::AvatarBadge::new(),
                "ui-gallery-avatar-badge-lg",
            ),
        ]
    })
    .test_id("ui-gallery-avatar-badge-dot-row");

    let icon_row = wrap_row(cx, |cx| {
        let fg = ColorRef::Color(primary_fg);
        let badge = shadcn::AvatarBadge::new().children([icon(cx, "lucide.plus", Px(10.0), fg)]);

        vec![avatar_with_badge(
            cx,
            avatar_image.clone(),
            shadcn::AvatarSize::Default,
            badge,
            "ui-gallery-avatar-badge-icon",
        )]
    })
    .test_id("ui-gallery-avatar-badge-icon-row");

    ui::v_flex(|_cx| vec![dot_row, icon_row])
        .gap(Space::N4)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
        .test_id("ui-gallery-avatar-badge")
}
// endregion: example
