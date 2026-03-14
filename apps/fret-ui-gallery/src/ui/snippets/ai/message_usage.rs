pub const SOURCE: &str = include_str!("message_usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::Invalidation;
use fret_ui::action::OnActivate;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, Justify, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

fn seed_messages() -> Arc<[ui_ai::AiMessage]> {
    Arc::from(
        vec![
            ui_ai::AiMessage::new(
                1,
                ui_ai::MessageRole::User,
                [ui_ai::MessagePart::Text(Arc::<str>::from(
                    "Can you summarize the current Message parity audit?",
                ))],
            ),
            ui_ai::AiMessage::new(
                2,
                ui_ai::MessageRole::Assistant,
                [ui_ai::MessagePart::Text(Arc::<str>::from(
                    "Sure — here is the short version:\n\n- user messages stay right-aligned and bubbled\n- assistant messages remain full-width flow content\n- `MessageActions` stays a sibling under the latest assistant reply\n\n```rust\nui_ai::Message::new(role, [content, actions])\n```",
                ))],
            ),
        ]
        .into_boxed_slice(),
    )
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let messages_model = cx.local_model_keyed("messages", seed_messages);
    let prompt_model = cx.local_model_keyed("prompt", || {
        String::from("Add a small follow-up question...")
    });
    let next_id_model = cx.local_model_keyed("next_id", || {
        (seed_messages().len() as u64).saturating_add(1)
    });
    let last_action_model = cx.local_model_keyed("last_action", || None::<Arc<str>>);

    let messages = cx
        .get_model_cloned(&messages_model, Invalidation::Layout)
        .unwrap_or_else(seed_messages);
    let revision = messages.len().min(u64::MAX as usize) as u64;
    let last_assistant_index = messages
        .iter()
        .rposition(|message| message.role == ui_ai::MessageRole::Assistant);
    let last_action = cx
        .get_model_cloned(&last_action_model, Invalidation::Paint)
        .flatten()
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("<none>"));

    let set_action = |label: &'static str| -> OnActivate {
        let last_action_model = last_action_model.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action_model, |value| {
                *value = Some(Arc::<str>::from(label));
            });
            host.request_redraw(action_cx.window);
        })
    };

    let on_send: OnActivate = Arc::new({
        let messages_model = messages_model.clone();
        let prompt_model = prompt_model.clone();
        let next_id_model = next_id_model.clone();
        move |host, action_cx, _reason| {
            let prompt = host
                .models_mut()
                .get_cloned(&prompt_model)
                .unwrap_or_default();
            let prompt_trimmed = prompt.trim();
            if prompt_trimmed.is_empty() {
                return;
            }

            let user_id = host.models_mut().get_copied(&next_id_model).unwrap_or(1);
            let assistant_id = user_id.saturating_add(1);
            let _ = host.models_mut().update(&next_id_model, |value| {
                *value = assistant_id.saturating_add(1);
            });

            let mut out: Vec<ui_ai::AiMessage> = host
                .models_mut()
                .get_cloned(&messages_model)
                .unwrap_or_else(seed_messages)
                .iter()
                .cloned()
                .collect();
            out.push(ui_ai::AiMessage::new(
                user_id,
                ui_ai::MessageRole::User,
                [ui_ai::MessagePart::Text(Arc::<str>::from(prompt_trimmed))],
            ));
            out.push(ui_ai::AiMessage::new(
                assistant_id,
                ui_ai::MessageRole::Assistant,
                [ui_ai::MessagePart::Text(Arc::<str>::from(format!(
                    "Thanks — here is a docs-aligned follow-up for `{prompt_trimmed}`:\n\n- keep `MessageContent` focused on content chrome\n- keep `MessageActions` as separate children of `Message`\n- attach retry/copy actions only to the latest assistant reply",
                )))],
            ));
            let _ = host.models_mut().update(&messages_model, |messages| {
                *messages = Arc::from(out.into_boxed_slice());
            });
            host.request_redraw(action_cx.window);
        }
    });

    let conversation = ui_ai::Conversation::new([])
        .content_revision(revision)
        .stick_to_bottom(true)
        .test_id("ui-ai-message-usage-conversation")
        .refine_layout(LayoutRefinement::default().w_full().flex_1().min_h_0())
        .into_element_with_children(cx, |cx| {
            let content_children: Vec<AnyElement> = messages
                .iter()
                .enumerate()
                .map(|(index, message)| {
                    let response_parts: Vec<AnyElement> = message
                        .parts
                        .iter()
                        .filter_map(|part| match part {
                            ui_ai::MessagePart::Text(text) => match message.role {
                                ui_ai::MessageRole::Assistant => Some(
                                    ui_ai::MessageResponse::new(text.clone())
                                        .streaming(false)
                                        .test_id_prefix(Arc::<str>::from(format!(
                                            "ui-ai-message-usage-msg-{index}-response-"
                                        )))
                                        .into_element(cx),
                                ),
                                _ => Some(cx.text(text.clone())),
                            },
                            _ => None,
                        })
                        .collect();

                    let content = ui_ai::MessageContent::new(message.role, response_parts)
                        .test_id(format!("ui-ai-message-usage-msg-{index}-content"))
                        .into_element(cx);

                    let mut children = vec![content];
                    if Some(index) == last_assistant_index {
                        let actions = ui_ai::MessageActions::new([
                            ui_ai::MessageAction::new("Retry")
                                .tooltip("Retry")
                                .icon(fret_icons::ids::ui::LOADER)
                                .test_id("ui-ai-message-usage-action-retry")
                                .on_activate(set_action("assistant.retry"))
                                .into_element(cx),
                            ui_ai::MessageAction::new("Copy")
                                .tooltip("Copy")
                                .icon(fret_icons::ids::ui::COPY)
                                .test_id("ui-ai-message-usage-action-copy")
                                .on_activate(set_action("assistant.copy"))
                                .into_element(cx),
                        ])
                        .justify(Justify::Start)
                        .test_id("ui-ai-message-usage-actions")
                        .into_element(cx);
                        children.push(actions);
                    }

                    ui_ai::Message::new(message.role, children)
                        .test_id(format!("ui-ai-message-usage-msg-{index}"))
                        .into_element(cx)
                })
                .collect();

            vec![
                ui_ai::ConversationContent::new(content_children).into_element(cx),
                ui_ai::ConversationScrollButton::default()
                    .test_id("ui-ai-message-usage-scroll-bottom")
                    .into_element(cx),
            ]
        });

    let prompt_input = ui_ai::PromptInput::new(prompt_model)
        .on_send(on_send)
        .test_id_root("ui-ai-message-usage-prompt")
        .test_id_textarea("ui-ai-message-usage-prompt-textarea")
        .test_id_send("ui-ai-message-usage-prompt-send")
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    let prompt_row = ui::h_flex(move |_cx| vec![prompt_input])
        .layout(LayoutRefinement::default().w_full())
        .justify(Justify::Center)
        .into_element(cx);

    let body = ui::v_flex(move |_cx| vec![conversation, prompt_row])
        .layout(LayoutRefinement::default().w_full().h_full())
        .gap(Space::N4)
        .into_element(cx);

    let props = cx.with_theme(|theme| {
        let chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .p(Space::N6);
        decl_style::container_props(
            theme,
            chrome,
            LayoutRefinement::default()
                .w_full()
                .h_px(Px(600.0))
                .min_w_0()
                .min_h_0(),
        )
    });
    let frame = cx.container(props, move |_cx| vec![body]);

    let marker = cx
        .text(format!("last_action={last_action}"))
        .test_id("ui-ai-message-usage-last-action");

    ui::v_flex(move |cx| {
        vec![
            cx.text("Message usage (AI Elements)")
                .test_id("ui-ai-message-usage-title"),
            cx.text(
                "Docs-aligned composition: Conversation + Message + MessageActions + PromptInput.",
            ),
            marker,
            frame,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
