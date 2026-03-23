pub const SOURCE: &str = include_str!("shimmer_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{FontId, FontWeight, Px, TextStyle};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let text = ui_ai::Shimmer::new("This text has a shimmer effect")
        .test_id("ui-ai-shimmer-root")
        .into_element(cx);

    let heading_style = TextStyle {
        font: FontId::ui(),
        size: Px(36.0),
        weight: FontWeight::BOLD,
        line_height: Some(Px(40.0)),
        ..Default::default()
    };
    let heading = ui_ai::Shimmer::new("Large Heading")
        .text_style(heading_style)
        .into_element(cx);

    let slow = ui_ai::Shimmer::new("Slower shimmer with wider spread")
        .duration(3.0)
        .spread(3.0)
        .into_element(cx);

    ui::v_flex(move |_cx| vec![text, heading, slow])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .items_center()
        .into_element(cx)
}
// endregion: example
