pub const SOURCE: &str = include_str!("context_default.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::LayoutRefinement;
use fret_ui_shadcn::prelude::*;

fn centered<H: UiHost>(cx: &mut ElementContext<'_, H>, body: AnyElement) -> AnyElement {
    ui::h_flex(move |_cx| [body])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
        .into_element(cx)
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
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
        .test_id_root("ui-ai-context-default-root")
        .into_element(cx);

    centered(cx, context)
}
// endregion: example
