pub const SOURCE: &str = include_str!("context_default.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui_ai as ui_ai;
use fret_ui_kit::{LayoutRefinement, ui};
use fret_ui_shadcn::prelude::*;

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
        .test_id_root("ui-ai-context-default-root")
        .into_element(cx);

    centered(cx, context)
}
// endregion: example
