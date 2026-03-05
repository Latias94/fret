pub const SOURCE: &str = include_str!("shimmer_demo.rs");

// region: example
use fret_core::{FontId, FontWeight, Px, TextStyle};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let text = ui_ai::Shimmer::new(Arc::<str>::from("This text has a shimmer effect"))
        .test_id("ui-ai-shimmer-root")
        .into_element(cx);

    let heading_style = TextStyle {
        font: FontId::ui(),
        size: Px(36.0),
        weight: FontWeight::BOLD,
        line_height: Some(Px(40.0)),
        ..Default::default()
    };
    let heading = ui_ai::Shimmer::new(Arc::<str>::from("Large Heading"))
        .text_style(heading_style)
        .into_element(cx);

    let slow = ui_ai::Shimmer::new(Arc::<str>::from("Slower shimmer with wider spread"))
        .duration_secs(3.0)
        .spread(3.0)
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4)
            .items_center(),
        move |_cx| vec![text, heading, slow],
    )
}
// endregion: example
