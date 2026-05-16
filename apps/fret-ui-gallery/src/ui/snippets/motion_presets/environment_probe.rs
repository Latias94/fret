pub const SOURCE: &str = include_str!("environment_probe.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::ColorScheme;
use fret_ui::Invalidation;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn color_scheme_label(value: Option<ColorScheme>) -> &'static str {
    match value {
        Some(ColorScheme::Light) => "light",
        Some(ColorScheme::Dark) => "dark",
        None => "unknown",
    }
}

fn option_bool_label(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn option_text_scale_label(value: Option<f32>) -> String {
    value
        .map(|value| format!("{:.2}", value))
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let color_scheme = cx.environment_color_scheme(Invalidation::Paint);
    let prefers_reduced_motion = cx.environment_prefers_reduced_motion(Invalidation::Paint);
    let text_scale_factor = cx.environment_text_scale_factor(Invalidation::Layout);

    let color_scheme_label = color_scheme_label(color_scheme);
    let motion_label = option_bool_label(prefers_reduced_motion);
    let text_scale_label = option_text_scale_label(text_scale_factor);

    let probe_body = ui::h_flex(move |cx| {
        vec![
            shadcn::Badge::new(format!("scheme: {color_scheme_label}"))
                .variant(shadcn::BadgeVariant::Outline)
                .test_id(format!(
                    "ui-gallery-motion-presets-environment-probe-color-scheme-{color_scheme_label}"
                ))
                .into_element(cx),
            shadcn::Badge::new(format!("reduced: {motion_label}"))
                .variant(shadcn::BadgeVariant::Outline)
                .test_id(format!(
                    "ui-gallery-motion-presets-environment-probe-reduced-motion-{motion_label}"
                ))
                .into_element(cx),
            shadcn::Badge::new(format!("text scale: {text_scale_label}"))
                .variant(shadcn::BadgeVariant::Outline)
                .test_id(format!(
                    "ui-gallery-motion-presets-environment-probe-text-scale-{text_scale_label}"
                ))
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .items_center()
    .gap(Space::N2)
    .wrap()
    .into_element(cx);

    shadcn::card(|cx| {
        ui::children![
            cx;
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_title("Environment probe"),
                    shadcn::card_description(
                        "Runtime window environment values observed through ElementContext queries.",
                    ),
                ]
            }),
            shadcn::card_content(|cx| ui::children![cx; probe_body]),
        ]
    })
    .refine_layout(LayoutRefinement::default().w_full().max_w(Px(760.0)).min_w_0())
    .into_element(cx)
    .test_id("ui-gallery-motion-presets-environment-probe")
}
// endregion: example
