pub const SOURCE: &str = include_str!("colors.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Color;
use fret_core::window::ColorScheme;
use fret_ui::Invalidation;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    fret_ui_kit::ui::h_flex(children)
        .gap(Space::N2)
        .wrap()
        .w_full()
        .justify_center()
        .items_center()
}

fn bg_fg_for_scheme(
    light_bg: u32,
    light_fg: u32,
    dark_bg: u32,
    dark_fg: u32,
    scheme: ColorScheme,
) -> (Color, Color) {
    match scheme {
        ColorScheme::Dark => (
            fret_ui_kit::colors::linear_from_hex_rgb(dark_bg),
            fret_ui_kit::colors::linear_from_hex_rgb(dark_fg),
        ),
        ColorScheme::Light => (
            fret_ui_kit::colors::linear_from_hex_rgb(light_bg),
            fret_ui_kit::colors::linear_from_hex_rgb(light_fg),
        ),
    }
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    // Upstream: `apps/v4/examples/radix/badge-colors.tsx`.
    let scheme = cx.environment_color_scheme(Invalidation::Paint);
    let scheme = scheme.unwrap_or(ColorScheme::Light);

    let (blue_bg, blue_fg) =
        bg_fg_for_scheme(0xEF_F6_FF, 0x1D_4E_D8, 0x17_25_54, 0x93_C5_FD, scheme);
    let (green_bg, green_fg) =
        bg_fg_for_scheme(0xF0_FD_F4, 0x15_80_3D, 0x05_2E_16, 0x86_EF_AC, scheme);
    let (sky_bg, sky_fg) = bg_fg_for_scheme(0xF0_F9_FF, 0x03_69_A1, 0x08_2F_49, 0x7D_D3_FC, scheme);
    let (purple_bg, purple_fg) =
        bg_fg_for_scheme(0xFA_F5_FF, 0x7E_22_CE, 0x3B_07_64, 0xD8_B4_FE, scheme);
    let (red_bg, red_fg) = bg_fg_for_scheme(0xFE_F2_F2, 0xB9_1C_1C, 0x45_0A_0A, 0xFD_A4_AF, scheme);

    row(|cx| {
        vec![
            shadcn::Badge::new("Blue")
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(blue_bg))
                        .text_color(ColorRef::Color(blue_fg)),
                )
                .test_id("ui-gallery-badge-colors-blue")
                .into_element(cx),
            shadcn::Badge::new("Green")
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(green_bg))
                        .text_color(ColorRef::Color(green_fg)),
                )
                .test_id("ui-gallery-badge-colors-green")
                .into_element(cx),
            shadcn::Badge::new("Sky")
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(sky_bg))
                        .text_color(ColorRef::Color(sky_fg)),
                )
                .test_id("ui-gallery-badge-colors-sky")
                .into_element(cx),
            shadcn::Badge::new("Purple")
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(purple_bg))
                        .text_color(ColorRef::Color(purple_fg)),
                )
                .test_id("ui-gallery-badge-colors-purple")
                .into_element(cx),
            shadcn::Badge::new("Red")
                .refine_style(
                    ChromeRefinement::default()
                        .bg(ColorRef::Color(red_bg))
                        .text_color(ColorRef::Color(red_fg)),
                )
                .test_id("ui-gallery-badge-colors-red")
                .into_element(cx),
        ]
    })
    .into_element(cx)
    .test_id("ui-gallery-badge-colors")
}
// endregion: example
