pub const SOURCE: &str = include_str!("with_badge.rs");

// region: example
use super::demo_image;
use fret::{UiChild, UiCx};
use fret_core::window::ColorScheme;
use fret_ui::Invalidation;
use fret_ui_kit::{ColorRef, IntoUiElement};
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

fn avatar_with_badge<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Option<fret_core::ImageId>,
    size: shadcn::AvatarSize,
    badge: shadcn::AvatarBadge,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let image = shadcn::AvatarImage::maybe(avatar_image).into_element(cx);
    let fallback = shadcn::AvatarFallback::new("CN")
        .when_image_missing(avatar_image)
        .delay_ms(120)
        .into_element(cx);
    let badge = badge.into_element(cx);

    shadcn::Avatar::new([image, fallback, badge])
        .size(size)
        .into_element(cx)
        .test_id(test_id)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar_image = demo_image(cx);
    let scheme = cx.environment_color_scheme(Invalidation::Paint);
    let badge_bg = match scheme.unwrap_or(ColorScheme::Light) {
        ColorScheme::Dark => fret_ui_kit::colors::linear_from_hex_rgb(0x16_65_34),
        ColorScheme::Light => fret_ui_kit::colors::linear_from_hex_rgb(0x16_A3_4A),
    };

    wrap_row(|cx| {
        vec![
            avatar_with_badge(
                cx,
                avatar_image,
                shadcn::AvatarSize::Default,
                shadcn::AvatarBadge::new()
                    .refine_style(ChromeRefinement::default().bg(ColorRef::Color(badge_bg))),
                "ui-gallery-avatar-badge-default",
            )
            .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-avatar-badge")
}
// endregion: example
