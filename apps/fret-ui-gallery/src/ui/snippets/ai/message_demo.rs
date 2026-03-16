pub const SOURCE: &str = include_str!("message_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{Justify, LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let last_action_model = cx.local_model_keyed("last_action", || None::<Arc<str>>);
    let last_action = cx
        .get_model_cloned(&last_action_model, Invalidation::Paint)
        .flatten()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "<none>".to_string());

    let marker = cx
        .text(format!("last_action={last_action}"))
        .test_id("ui-ai-message-demo-last-action");

    let set_action = |label: &'static str| {
        let last_action_model = last_action_model.clone();
        move |host, acx| {
            let _ = host.models_mut().update(&last_action_model, |v| {
                *v = Some(Arc::<str>::from(label));
            });
            host.request_redraw(acx.window);
        }
    };

    let assistant_actions = ui_ai::MessageActions::new([
        ui_ai::MessageAction::new("Copy")
            .tooltip("Copy message")
            .icon(fret_icons::ids::ui::COPY)
            .test_id("ui-ai-message-demo-assistant-action-copy")
            .on_activate(cx.actions().listen(set_action("assistant.copy")))
            .into_element(cx),
        ui_ai::MessageAction::new("Regenerate")
            .tooltip("Regenerate")
            .icon(fret_icons::ids::ui::LOADER)
            .test_id("ui-ai-message-demo-assistant-action-regenerate")
            .on_activate(cx.actions().listen(set_action("assistant.regenerate")))
            .into_element(cx),
    ])
    .justify(Justify::Start)
    .test_id("ui-ai-message-demo-assistant-actions")
    .into_element(cx);

    let user_actions = ui_ai::MessageActions::new([ui_ai::MessageAction::new("Edit")
        .tooltip("Edit message")
        .icon(fret_icons::ids::ui::FILE)
        .test_id("ui-ai-message-demo-user-action-edit")
        .on_activate(cx.actions().listen(set_action("user.edit")))
        .into_element(cx)])
    .justify(Justify::End)
    .test_id("ui-ai-message-demo-user-actions")
    .into_element(cx);

    let assistant = ui_ai::Message::new(
        ui_ai::MessageRole::Assistant,
        [
            ui_ai::MessageContent::new(
                ui_ai::MessageRole::Assistant,
                [ui_ai::MessageResponse::new(Arc::<str>::from(
                    "**Assistant** messages default to a full-width flow (no bubble).\n\n\
Compose actions/toolbar slots as separate children.\n\n\
```rust\n\
fn streamed_demo() {\n\
    println!(\"hello from message response\");\n\
}\n\
```\n",
                ))
                .streaming(false)
                .test_id_prefix("ui-ai-message-demo-assistant-response-")
                .into_element(cx)],
            )
            .test_id("ui-ai-message-demo-assistant-content")
            .into_element(cx),
            assistant_actions,
        ],
    )
    .test_id("ui-ai-message-demo-assistant")
    .into_element(cx);

    let user = ui_ai::Message::new(
        ui_ai::MessageRole::User,
        [
            ui_ai::MessageContent::new(
                ui_ai::MessageRole::User,
                [
                    cx.text("User messages render as a bubble aligned to the right."),
                    cx.text("Bubble chrome is controlled by theme tokens."),
                ],
            )
            .test_id("ui-ai-message-demo-user-content")
            .into_element(cx),
            user_actions,
        ],
    )
    .test_id("ui-ai-message-demo-user")
    .into_element(cx);

    let title = cx.text("Message (AI Elements): alignment + bubble + actions + markdown response.");

    ui::v_flex(move |_cx| vec![title, marker, assistant, user])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .into_element(cx)
}
// endregion: example
