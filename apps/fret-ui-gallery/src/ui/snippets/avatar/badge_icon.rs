pub const SOURCE: &str = include_str!("badge_icon.rs");

// region: example
use super::demo_image;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::Theme;
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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let avatar_image = demo_image(cx);

    wrap_row(|cx| {
        let image = shadcn::AvatarImage::maybe(avatar_image).into_element(cx);
        let fallback = shadcn::AvatarFallback::new("CN")
            .when_image_missing(avatar_image)
            .delay_ms(120)
            .into_element(cx);

        let primary_fg = Theme::global(&*cx.app).color_token("primary-foreground");
        let fg = ColorRef::Color(primary_fg);
        let badge = shadcn::AvatarBadge::new()
            .children([icon(cx, "lucide.plus", Px(10.0), fg).into_element(cx)]);
        let badge = badge.into_element(cx);

        vec![
            shadcn::Avatar::new([image, fallback, badge])
                .size(shadcn::AvatarSize::Default)
                .into_element(cx)
                .test_id("ui-gallery-avatar-badge-icon"),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-avatar-badge-icon-row")
}
// endregion: example
