pub const SOURCE: &str = include_str!("open_in_chat_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let query: Arc<str> = Arc::from("How do I implement a focus trap?");

    let menu = ui_ai::OpenIn::new(query)
        .trigger(ui_ai::OpenInTrigger::new().test_id("ui-ai-open-in-chat-demo-trigger"))
        .into_element_with_entries(cx, move |cx| {
            vec![
                ui_ai::OpenInChatGpt::new()
                    .test_id("ui-ai-open-in-chat-demo-item-chatgpt")
                    .into_entry(cx),
                ui_ai::OpenInClaude::new()
                    .test_id("ui-ai-open-in-chat-demo-item-claude")
                    .into_entry(cx),
                ui_ai::OpenInT3::new()
                    .test_id("ui-ai-open-in-chat-demo-item-t3")
                    .into_entry(cx),
                ui_ai::OpenInScira::new()
                    .test_id("ui-ai-open-in-chat-demo-item-scira")
                    .into_entry(cx),
                ui_ai::OpenInv0::new()
                    .test_id("ui-ai-open-in-chat-demo-item-v0")
                    .into_entry(cx),
                ui_ai::OpenInCursor::new()
                    .test_id("ui-ai-open-in-chat-demo-item-cursor")
                    .into_entry(cx),
            ]
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("OpenIn (AI Elements)"),
            cx.text("Selecting a provider emits Effect::OpenUrl (URLs match upstream)."),
            menu,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
