pub const SOURCE: &str = include_str!("context_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_kit::{LayoutRefinement, ui};

fn centered<B>(cx: &mut UiCx<'_>, body: B) -> impl UiChild + use<B>
where
    B: UiChild,
{
    ui::h_flex(move |cx| [body.into_element(cx)])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
        .into_element(cx)
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let usage = ui_ai::ContextUsage {
        input_tokens: Some(32_000),
        output_tokens: Some(8_000),
        reasoning_tokens: Some(512),
        cached_input_tokens: Some(2_048),
        ..Default::default()
    };

    let context = ui_ai::Context::new(42_560, 128_000)
        .model_id("openai:gpt-5")
        .usage(usage)
        .test_id_root("ui-ai-context-demo-root")
        .children([
            ui_ai::ContextTrigger::default()
                .test_id("ui-ai-context-demo-trigger")
                .into_element(cx),
            ui_ai::ContextContent::new([
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
            .into_element(cx),
        ])
        .into_element(cx);

    centered(cx, context)
}
// endregion: example
