pub const SOURCE: &str = include_str!("context_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let usage = ui_ai::ContextUsage {
        input_tokens: Some(12_345),
        output_tokens: Some(6_789),
        reasoning_tokens: Some(321),
        cached_input_tokens: Some(1_024),
        input_cost_usd: Some(0.12),
        output_cost_usd: Some(0.08),
        reasoning_cost_usd: Some(0.01),
        cached_cost_usd: Some(0.00),
        total_cost_usd: Some(0.21),
    };

    let context = ui_ai::Context::new(19_134, 100_000)
        .model_id("openai:gpt-4.1-mini")
        .usage(usage)
        .test_id_root("ui-ai-context-demo-root")
        .into_element_with_children(cx, |cx| {
            let trigger = ui_ai::ContextTrigger::default()
                .test_id("ui-ai-context-demo-trigger")
                .into_element(cx);

            let content = ui_ai::ContextContent::new([
                ui_ai::ContextContentHeader::default()
                    .test_id("ui-ai-context-demo-header")
                    .into_element(cx),
                ui_ai::ContextContentBody::new([
                    ui_ai::ContextInputUsage::default()
                        .test_id("ui-ai-context-demo-usage-input")
                        .into_element(cx),
                    ui_ai::ContextOutputUsage::default()
                        .test_id("ui-ai-context-demo-usage-output")
                        .into_element(cx),
                    ui_ai::ContextReasoningUsage::default()
                        .test_id("ui-ai-context-demo-usage-reasoning")
                        .into_element(cx),
                    ui_ai::ContextCacheUsage::default()
                        .test_id("ui-ai-context-demo-usage-cache")
                        .into_element(cx),
                ])
                .test_id("ui-ai-context-demo-body")
                .into_element(cx),
                ui_ai::ContextContentFooter::default()
                    .test_id("ui-ai-context-demo-footer")
                    .into_element(cx),
            ])
            .test_id("ui-ai-context-demo-content")
            .into_element(cx);

            (trigger, content)
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Context (AI Elements)"),
            cx.text("Hover to inspect model usage + token budget."),
            context,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
