use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_open_in_chat_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

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

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("OpenIn (AI Elements)"),
                cx.text("Selecting a provider emits Effect::OpenUrl (URLs match upstream)."),
                menu,
            ]
        },
    )]
}
