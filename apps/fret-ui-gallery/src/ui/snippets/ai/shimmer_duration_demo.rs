pub const SOURCE: &str = include_str!("shimmer_duration_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let item = |cx: &mut AppComponentCx<'_>, label: &'static str, secs: f32, text: &'static str| {
        ui::v_stack(move |cx| {
            vec![
                decl_text::text_control_readout(cx, label),
                ui_ai::Shimmer::new(text).duration(secs).into_element(cx),
            ]
        })
        .gap(Space::N3)
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
