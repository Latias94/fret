pub const SOURCE: &str = include_str!("reasoning_demo.rs");

// region: example
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let streaming = cx.local_model_keyed("streaming", || false);

    let is_streaming = cx
        .get_model_copied(&streaming, Invalidation::Layout)
        .unwrap_or(false);

    let start = shadcn::Button::new("Start streaming")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .test_id("ui-ai-reasoning-start-streaming")
        .on_activate(Arc::new({
            let streaming = streaming.clone();
            move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&streaming, |v| *v = true);
                host.notify(action_cx);
            }
        }))
        .into_element(cx);

    let stop = shadcn::Button::new("Stop streaming")
        .variant(shadcn::ButtonVariant::Secondary)
        .size(shadcn::ButtonSize::Sm)
        .test_id("ui-ai-reasoning-stop-streaming")
        .on_activate(Arc::new({
            let streaming = streaming.clone();
            move |host, action_cx, _reason| {
                let _ = host.models_mut().update(&streaming, |v| *v = false);
                host.notify(action_cx);
            }
        }))
        .into_element(cx);

    // Mirror the upstream docs pattern: consolidate multiple reasoning parts into one panel.
    // Source reference: `repo-ref/ai-elements/apps/docs/content/components/(chatbot)/reasoning.mdx`.
    let reasoning_text = [
        "Reasoning content is markdown.",
        "- Opens automatically when streaming starts.",
        "- Auto-closes shortly after streaming ends.",
    ]
    .join("\n\n");

    let reasoning = ui_ai::Reasoning::new(is_streaming)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .test_id_root("ui-ai-reasoning-root")
        .into_element(
            cx,
            |cx| ui_ai::ReasoningTrigger::new().into_element(cx),
            |cx| {
                ui_ai::ReasoningContent::new(reasoning_text)
                    .test_id("ui-ai-reasoning-content")
                    .into_element(cx)
            },
        );

    let controls = ui::h_row(move |_cx| vec![start, stop])
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text("Reasoning (AI Elements)"),
            cx.text("Start streaming to auto-open; stop to auto-close."),
            controls,
            reasoning,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
