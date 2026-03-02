pub const SOURCE: &str = include_str!("shimmer_demo.rs");

// region: example
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let shimmer = ui_ai::Shimmer::new(Arc::<str>::from("Streaming"))
        .test_id("ui-ai-shimmer-root")
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .h_px(Px(48.0))
                .min_w_0(),
        )
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Shimmer (AI Elements)"),
                cx.text("Animated shimmer used as a lightweight streaming indicator."),
                shimmer,
            ]
        },
    )
}
// endregion: example

