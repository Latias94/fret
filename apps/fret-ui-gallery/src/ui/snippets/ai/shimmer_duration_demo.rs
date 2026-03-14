pub const SOURCE: &str = include_str!("shimmer_duration_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let item = |cx: &mut UiCx<'_>, label: &'static str, secs: f32, text: &'static str| {
            ui::v_stack(move |cx| {
                vec![
                    shadcn::Badge::new(label)
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                    ui_ai::Shimmer::new(Arc::<str>::from(text))
                        .duration_secs(secs)
                        .into_element(cx),
                ]
            })
            .gap(Space::N2)
            .items_center()
            .into_element(cx)
    };

    ui::v_flex(move |cx| {
        vec![
            item(cx, "Fast (1 second)", 1.0, "Loading quickly..."),
            item(cx, "Default (2 seconds)", 2.0, "Loading at normal speed..."),
            item(cx, "Slow (4 seconds)", 4.0, "Loading slowly..."),
            item(cx, "Very Slow (6 seconds)", 6.0, "Loading very slowly..."),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N6)
    .into_element(cx)
    .test_id("ui-ai-shimmer-duration-root")
}
// endregion: example
