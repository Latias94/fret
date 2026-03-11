pub const SOURCE: &str = include_str!("badge_icon.rs");

// region: example
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_kit::ColorRef;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

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
    fret_ui_shadcn::icon::icon_with(
        cx,
        fret_icons::IconId::new_static(name),
        Some(size),
        Some(fg),
    )
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    avatar_image: Model<Option<fret_core::ImageId>>,
) -> AnyElement {
    wrap_row(cx, |cx| {
        let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
        let fallback = shadcn::AvatarFallback::new("CN")
            .when_image_missing_model(avatar_image)
            .delay_ms(120)
            .into_element(cx);

        let primary_fg = Theme::global(&*cx.app).color_token("primary-foreground");
        let fg = ColorRef::Color(primary_fg);
        let badge = shadcn::AvatarBadge::new().children([icon(cx, "lucide.plus", Px(10.0), fg)]);
        let badge = badge.into_element(cx);

        vec![
            shadcn::Avatar::new([image, fallback, badge])
                .size(shadcn::AvatarSize::Default)
                .into_element(cx)
                .test_id("ui-gallery-avatar-badge-icon"),
        ]
    })
    .test_id("ui-gallery-avatar-badge-icon-row")
}
// endregion: example
