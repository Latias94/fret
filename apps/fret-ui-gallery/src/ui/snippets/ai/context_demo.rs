pub const SOURCE: &str = include_str!("context_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let usage = ui_ai::ContextUsage {
        prompt_tokens: Some(12_345),
        completion_tokens: Some(6_789),
        total_tokens: Some(19_134),
    };

    let context = ui_ai::Context::new(19_134, 100_000)
        .model_id("gpt-4.1-mini")
        .usage(usage)
        .test_id_trigger("ui-ai-context-demo-trigger")
        .test_id_content("ui-ai-context-demo-content")
        .into_element(cx);

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Context (AI Elements)"),
                cx.text("Hover to inspect model usage + token budget."),
                context,
            ]
        },
    )
}
// endregion: example

