pub const SOURCE: &str = include_str!("conversation_demo.rs");

// region: example
use fret_core::{Px, SemanticsRole};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::SemanticsProps;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, Justify, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::{AnyElement, ElementContext, UiHost};
use std::sync::Arc;

const DIAG_SEED_MESSAGES_ENV: &str = "FRET_UI_GALLERY_AI_CONVERSATION_SEED_MESSAGES";

fn diag_seed_message_count() -> usize {
    std::env::var(DIAG_SEED_MESSAGES_ENV)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
}

fn build_seeded_messages(count: usize) -> Arc<[ui_ai::AiMessage]> {
    let seeded: Vec<ui_ai::AiMessage> = (0..count)
        .map(|index| {
            let id = index as u64 + 1;
            let role = if index % 2 == 0 {
                ui_ai::MessageRole::User
            } else {
                ui_ai::MessageRole::Assistant
            };
            let text = if matches!(role, ui_ai::MessageRole::User) {
                format!(
                    "Seeded user message {id}: summarize the state of the conversation and keep the latest action items visible."
                )
            } else {
                format!(
                    "Seeded assistant message {id}: here is a longer response for diagnostics.\n\n- bullet 1\n- bullet 2\n- bullet 3\n- bullet 4\n- bullet 5\n- bullet 6"
                )
            };
            ui_ai::AiMessage::new(id, role, [ui_ai::MessagePart::Text(Arc::<str>::from(text))])
        })
        .collect();
    Arc::from(seeded.into_boxed_slice())
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    #[derive(Default)]
    struct DemoModels {
        messages: Option<Model<Arc<[ui_ai::AiMessage]>>>,
        prompt: Option<Model<String>>,
        next_id: Option<Model<u64>>,
        exported_md_len: Option<Model<u64>>,
    }

    let needs_init = cx.with_state(DemoModels::default, |st| {
        st.messages.is_none()
            || st.prompt.is_none()
            || st.next_id.is_none()
            || st.exported_md_len.is_none()
    });
    if needs_init {
        let initial_messages = build_seeded_messages(diag_seed_message_count());
        let next_id_value = (initial_messages.len() as u64).max(3) + 1;
        let messages = cx.app.models_mut().insert(initial_messages);
        let prompt = cx.app.models_mut().insert(String::new());
        let next_id = cx.app.models_mut().insert(next_id_value);
        let exported_md_len = cx.app.models_mut().insert(0u64);

        cx.with_state(DemoModels::default, move |st| {
            st.messages = Some(messages.clone());
            st.prompt = Some(prompt.clone());
            st.next_id = Some(next_id.clone());
            st.exported_md_len = Some(exported_md_len.clone());
        });
    }

    let (messages_model, prompt_model, next_id_model, exported_md_len_model) =
        cx.with_state(DemoModels::default, |st| {
            (
                st.messages.clone().expect("messages"),
                st.prompt.clone().expect("prompt"),
                st.next_id.clone().expect("next_id"),
                st.exported_md_len.clone().expect("exported_md_len"),
            )
        });

    let messages = cx
        .app
        .models_mut()
        .get_cloned(&messages_model)
        .unwrap_or_default();
    let revision = messages.len().min(u64::MAX as usize) as u64;
    let messages_empty = messages.is_empty();

    let on_send: OnActivate = Arc::new({
        let messages_model = messages_model.clone();
        let prompt_model = prompt_model.clone();
        let next_id_model = next_id_model.clone();
        move |host, acx, _reason| {
            let prompt = host
                .models_mut()
                .get_cloned(&prompt_model)
                .unwrap_or_default();
            let prompt_trimmed = prompt.trim();
            if prompt_trimmed.is_empty() {
                return;
            }

            let id = host.models_mut().get_copied(&next_id_model).unwrap_or(1);
            let _ = host.models_mut().update(&next_id_model, |v| {
                *v = v.saturating_add(1);
            });

            let existing = host
                .models_mut()
                .get_cloned(&messages_model)
                .unwrap_or_default();
            let mut out: Vec<ui_ai::AiMessage> = existing.iter().cloned().collect();
            out.push(ui_ai::AiMessage::new(
                id,
                ui_ai::MessageRole::User,
                [ui_ai::MessagePart::Text(Arc::<str>::from(prompt_trimmed))],
            ));
            let out: Arc<[ui_ai::AiMessage]> = Arc::from(out.into_boxed_slice());
            let _ = host.models_mut().update(&messages_model, |v| *v = out);
            let _ = host.models_mut().update(&prompt_model, |v| v.clear());
            host.request_redraw(acx.window);
        }
    });

    let download: OnActivate = Arc::new({
        let messages_model = messages_model.clone();
        let exported_md_len_model = exported_md_len_model.clone();
        move |host, acx, _reason| {
            let messages = host
                .models_mut()
                .get_cloned(&messages_model)
                .unwrap_or_default();
            let md = ui_ai::messages_to_markdown(&messages);
            let _ = host.models_mut().update(&exported_md_len_model, |v| {
                *v = md.len().min(u64::MAX as usize) as u64;
            });
            host.request_redraw(acx.window);
        }
    });

    let conversation = ui_ai::Conversation::new([])
        .content_revision(revision)
        .stick_to_bottom(true)
        .test_id("ui-ai-conversation-demo-transcript-root")
        .refine_layout(LayoutRefinement::default().w_full().flex_1().min_h_0())
        .into_element_with_children(cx, |cx| {
            let content_children: Vec<AnyElement> = if messages_empty {
                vec![ui_ai::ConversationEmptyState::new("Start a conversation")
                    .description("Type a message below to begin chatting.")
                    .icon(icon::icon(cx, IconId::new_static("lucide.message-square")))
                    .test_id("ui-ai-conversation-demo-empty")
                    .into_element(cx)]
            } else {
                messages
                    .iter()
                    .enumerate()
                    .map(|(index, message)| {
                        let content_parts: Vec<AnyElement> = message
                            .parts
                            .iter()
                            .filter_map(|part| match part {
                                ui_ai::MessagePart::Text(text) => Some(
                                    ui_ai::MessageResponse::new(text.clone())
                                        .streaming(false)
                                        .test_id_prefix(Arc::<str>::from(format!(
                                            "ui-ai-conversation-demo-msg-{index}-response-"
                                        )))
                                        .into_element(cx),
                                ),
                                _ => None,
                            })
                            .collect();

                        ui_ai::Message::new(
                            message.role,
                            [ui_ai::MessageContent::new(message.role, content_parts)
                                .test_id(format!("ui-ai-conversation-demo-msg-{index}"))
                                .into_element(cx)],
                        )
                        .into_element(cx)
                    })
                    .collect()
            };

            vec![
                ui_ai::ConversationContent::new(content_children).into_element(cx),
                ui_ai::ConversationDownload::new("Download conversation")
                    .disabled(messages_empty)
                    .on_activate(download.clone())
                    .test_id("ui-ai-conversation-demo-download")
                    .into_element(cx),
                ui_ai::ConversationScrollButton::default()
                    .test_id("ui-ai-conversation-demo-scroll-bottom")
                    .into_element(cx),
            ]
        });

    let exported_md_len = cx
        .app
        .models_mut()
        .get_copied(&exported_md_len_model)
        .unwrap_or(0);
    let instrumentation = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Text,
            test_id: Some(Arc::<str>::from("ui-ai-conversation-demo-exported-md-len")),
            numeric_value: Some(exported_md_len as f64),
            ..Default::default()
        },
        |_cx| Vec::<AnyElement>::new(),
    );
    let instrumentation = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Text,
            test_id: Some(Arc::<str>::from("ui-ai-conversation-demo-messages-len")),
            numeric_value: Some(messages.len() as f64),
            ..Default::default()
        },
        move |_cx| vec![instrumentation],
    );

    let prompt_input = ui_ai::PromptInput::new(prompt_model)
        .on_send(on_send)
        .test_id_root("ui-ai-conversation-demo-prompt-root")
        .test_id_textarea("ui-ai-conversation-demo-prompt-textarea")
        .test_id_send("ui-ai-conversation-demo-prompt-send")
        .into_element(cx);

    let prompt_row = ui::h_flex(move |_cx| vec![prompt_input])
        .layout(LayoutRefinement::default().w_full())
        .justify(Justify::Center)
        .into_element(cx);

    let body = ui::v_flex(move |_cx| vec![conversation, prompt_row, instrumentation])
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

    cx.container(props, move |_cx| vec![body])
}
// endregion: example
