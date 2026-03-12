pub const SOURCE: &str = include_str!("custom_colors.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let warn_bg = ColorRef::Color(fret_ui_kit::colors::linear_from_hex_rgb(0xFF_FA_EB));
    let warn_border = ColorRef::Color(fret_ui_kit::colors::linear_from_hex_rgb(0xFA_D9_73));

    shadcn::Alert::new([
        fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.triangle-alert")),
        shadcn::AlertTitle::new("Your subscription expires in 3 days").into_element(cx),
        shadcn::AlertDescription::new(
            "Renew now to avoid service interruption or upgrade to a paid plan.",
        )
        .into_element(cx),
    ])
    .refine_style(
        ChromeRefinement::default()
            .bg(warn_bg)
            .border_color(warn_border),
    )
    .refine_layout(LayoutRefinement::default().max_w(Px(520.0)))
    .into_element(cx)
    .test_id("ui-gallery-alert-colors")
}
// endregion: example
