pub const SOURCE: &str = include_str!("colors.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Color;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let color_spinner = |cx: &mut AppComponentCx<'_>, hex: u32, test_id: &'static str| {
        shadcn::Spinner::new()
            .refine_layout(LayoutRefinement::default().w_px(Px(24.0)).h_px(Px(24.0)))
            .color(ColorRef::Color(Color::from_srgb_hex_rgb(hex)))
            .into_element(cx)
            .test_id(test_id)
    };

    ui::h_flex(|cx| {
        vec![
            color_spinner(cx, 0xef4444, "ui-gallery-spinner-color-red"),
            color_spinner(cx, 0x22c55e, "ui-gallery-spinner-color-green"),
            color_spinner(cx, 0x3b82f6, "ui-gallery-spinner-color-blue"),
            color_spinner(cx, 0xeab308, "ui-gallery-spinner-color-yellow"),
            color_spinner(cx, 0xa855f7, "ui-gallery-spinner-color-purple"),
        ]
    })
    .gap(Space::N6)
    .items_center()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-spinner-color")
}
// endregion: example
