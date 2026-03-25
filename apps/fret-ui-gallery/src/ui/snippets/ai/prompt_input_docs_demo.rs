pub const SOURCE: &str = include_str!("prompt_input_docs_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_core::{ImageColorSpace, ImageId, Px};
use fret_icons::IconId;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::{Arc, OnceLock};

mod act {
    fret::actions!([ToggleSearch = "ui-gallery.ai.prompt_input_docs.toggle_search.v1"]);
}

#[derive(Clone, Copy)]
struct DemoModel {
    id: &'static str,
    name: &'static str,
}

const MODELS: &[DemoModel] = &[
    DemoModel {
        id: "gpt-4o",
        name: "GPT-4o",
    },
    DemoModel {
        id: "claude-opus-4-20250514",
        name: "Claude 4 Opus",
    },
];

fn screenshot_preview_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;

            let mut r = (18.0 + 76.0 * (1.0 - fy) + 118.0 * fx).min(255.0);
            let mut g = (22.0 + 42.0 * fy + 56.0 * (1.0 - fx)).min(255.0);
            let mut b = (36.0 + 82.0 * fy + 96.0 * (1.0 - fx)).min(255.0);

            if x < 8 || y < 8 || x + 8 >= width || y + 8 >= height {
                r = 232.0;
                g = 236.0;
                b = 241.0;
            } else if y > height / 5 && y < (height / 5) + 18 {
                r = 245.0;
                g = 196.0;
                b = 92.0;
            } else if x > width / 6 && x < (width * 5) / 6 && y > height / 3 && y < (height * 4) / 5
            {
                r = (r + 18.0).min(255.0);
                g = (g + 18.0).min(255.0);
                b = (b + 24.0).min(255.0);
            }

            out[idx] = r as u8;
            out[idx + 1] = g as u8;
            out[idx + 2] = b as u8;
            out[idx + 3] = 255;
        }
    }

    out
}

fn screenshot_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            320,
            200,
            screenshot_preview_rgba8(320, 200),
            ImageColorSpace::Srgb,
        )
    })
}

fn screenshot_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(screenshot_source()).image
}

fn model_name(model_id: &str) -> Arc<str> {
    MODELS
        .iter()
        .find(|model| model.id == model_id)
        .map(|model| Arc::<str>::from(model.name))
        .unwrap_or_else(|| Arc::<str>::from(model_id.to_owned()))
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let messages = cx.local_model_keyed("messages", Vec::<ui_ai::AiMessage>::new);
    let next_message_id = cx.local_model_keyed("next_message_id", || 1u64);
    let text = cx.local_model_keyed("text", String::new);
    let attachments = cx.local_model_keyed("attachments", Vec::<ui_ai::AttachmentData>::new);
    let use_web_search = cx.local_model_keyed("use_web_search", || false);
    let model_value = cx.local_model_keyed("model_value", || Some(Arc::<str>::from(MODELS[0].id)));
    let model_open = cx.local_model_keyed("model_open", || false);

    let transcript_messages = cx
        .get_model_cloned(&messages, Invalidation::Layout)
        .unwrap_or_default();
    let transcript_revision = transcript_messages.len().min(u64::MAX as usize) as u64;
    let searching = cx
        .get_model_cloned(&use_web_search, Invalidation::Layout)
        .unwrap_or(false);
    let screenshot_preview = screenshot_image_id(cx);

    cx.actions().models::<act::ToggleSearch>({
        let use_web_search = use_web_search.clone();
        move |models| {
            models
                .update(&use_web_search, |value| *value = !*value)
                .is_ok()
        }
    });

    let on_submit: ui_ai::OnPromptInputSubmit = Arc::new({
        let messages = messages.clone();
        let next_message_id = next_message_id.clone();
        let model_value = model_value.clone();
        let use_web_search = use_web_search.clone();
        move |host, action_cx, message, _reason| {
            let trimmed = message.text.trim();
            if trimmed.is_empty() && message.files.is_empty() {
                return;
            }

            let selected_model = host
                .models_mut()
                .read(&model_value, Clone::clone)
                .ok()
                .flatten()
                .unwrap_or_else(|| Arc::<str>::from(MODELS[0].id));
            let selected_model_name = model_name(selected_model.as_ref());
            let web_search = host
                .models_mut()
                .read(&use_web_search, |value| *value)
                .ok()
                .unwrap_or(false);

            let user_text = if trimmed.is_empty() {
                Arc::<str>::from("Sent with attachments")
            } else {
                Arc::<str>::from(trimmed.to_owned())
            };

            let attachment_summary = if message.files.is_empty() {
                Arc::<str>::from("None")
            } else {
                Arc::<str>::from(
                    message
                        .files
                        .iter()
                        .map(|item| ui_ai::get_attachment_label(item).to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            };

            let user_id = host
                .models_mut()
                .read(&next_message_id, |value| *value)
                .ok()
                .unwrap_or(1);
            let assistant_id = user_id.saturating_add(1);
            let _ = host.models_mut().update(&next_message_id, |value| {
                *value = assistant_id.saturating_add(1);
            });

            let assistant_text = Arc::<str>::from(format!(
                "Model: {selected_model_name}\nWeb search: {}\nAttachments: {attachment_summary}",
                if web_search { "enabled" } else { "disabled" }
            ));

            let _ = host.models_mut().update(&messages, |items| {
                items.push(ui_ai::AiMessage::new(
                    user_id,
                    ui_ai::MessageRole::User,
                    [ui_ai::MessagePart::Text(user_text)],
                ));
                items.push(ui_ai::AiMessage::new(
                    assistant_id,
                    ui_ai::MessageRole::Assistant,
                    [ui_ai::MessagePart::Text(assistant_text)],
                ));
            });

            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }
    });

    let on_add_attachments: fret_ui::action::OnActivate = Arc::new({
        let attachments = attachments.clone();
        move |host, action_cx, _reason| {
            let file = ui_ai::AttachmentFileData::new("docs-attachment")
                .filename("design-brief.pdf")
                .media_type("application/pdf")
                .size_bytes(42_000);
            let item = ui_ai::AttachmentData::File(file);
            let _ = host.models_mut().update(&attachments, |items| {
                if items
                    .iter()
                    .any(|existing| existing.id().as_ref() == "docs-attachment")
                {
                    return;
                }
                items.push(item.clone());
            });
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }
    });

    let on_add_screenshot: fret_ui::action::OnActivate = Arc::new({
        let attachments = attachments.clone();
        move |host, action_cx, _reason| {
            let mut file = ui_ai::AttachmentFileData::new("docs-screenshot")
                .filename("screenshot-2026-03-22.png")
                .media_type("image/png")
                .size_bytes(184_000);
            if let Some(preview) = screenshot_preview {
                file = file.preview_image(preview);
            }
            let item = ui_ai::AttachmentData::File(file);
            let _ = host.models_mut().update(&attachments, |items| {
                if items
                    .iter()
                    .any(|existing| existing.id().as_ref() == "docs-screenshot")
                {
                    return;
                }
                items.push(item.clone());
            });
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }
    });

    let conversation = ui_ai::Conversation::new([])
        .content_revision(transcript_revision)
        .stick_to_bottom(true)
        .test_id("ui-gallery-ai-prompt-input-docs-conversation")
        .refine_layout(LayoutRefinement::default().w_full().flex_1().min_h_0())
        .into_element_with_children(cx, move |cx| {
            let content_children =
                if transcript_messages.is_empty() {
                    vec![ui_ai::ConversationEmptyState::new("Start a conversation")
                    .description(
                        "Submit a prompt below to preview the docs-shaped PromptInput flow.",
                    )
                    .icon(decl_icon::icon(cx, IconId::new("lucide.message-square")))
                    .test_id("ui-gallery-ai-prompt-input-docs-empty")
                    .into_element(cx)]
                } else {
                    transcript_messages
                        .iter()
                        .enumerate()
                        .map(|(index, message)| {
                            let content_parts = message
                                .parts
                                .iter()
                                .filter_map(|part| match part {
                                    ui_ai::MessagePart::Text(text) => Some(
                                        ui_ai::MessageResponse::new(text.clone())
                                            .streaming(false)
                                            .into_element(cx),
                                    ),
                                    _ => None,
                                })
                                .collect::<Vec<_>>();

                            ui_ai::Message::new(
                                message.role,
                                [ui_ai::MessageContent::new(message.role, content_parts)
                                    .test_id(format!("ui-gallery-ai-prompt-input-docs-msg-{index}"))
                                    .into_element(cx)],
                            )
                            .into_element(cx)
                        })
                        .collect()
                };

            vec![
                ui_ai::ConversationContent::new(content_children).into_element(cx),
                ui_ai::ConversationScrollButton::default()
                    .test_id("ui-gallery-ai-prompt-input-docs-scroll-bottom")
                    .into_element(cx),
            ]
        });

    let body = {
        let prompt = ui_ai::PromptInputProvider::new()
            .text_model(text.clone())
            .attachments_model(attachments.clone())
            .into_element_with_children(cx, move |cx, controller| {
                let menu = ui_ai::PromptInputActionMenu::new(
                    ui_ai::PromptInputActionMenuContent::new([])
                        .add_attachments(
                            ui_ai::PromptInputActionAddAttachments::new()
                                .test_id("ui-gallery-ai-prompt-input-docs-add-attachments-item"),
                        )
                        .add_screenshot(
                            ui_ai::PromptInputActionAddScreenshot::new()
                                .test_id("ui-gallery-ai-prompt-input-docs-add-screenshot-item"),
                        ),
                )
                .trigger(
                    ui_ai::PromptInputActionMenuTrigger::new()
                        .test_id("ui-gallery-ai-prompt-input-docs-action-menu-trigger"),
                );

                let search_btn = ui_ai::PromptInputButton::new("Search")
                    .children([
                        decl_icon::icon(cx, IconId::new("lucide.globe")),
                        ui::text("Search").into_element(cx),
                    ])
                    .tooltip(
                        ui_ai::PromptInputButtonTooltip::new("Search the web")
                            .shortcut("⌘K")
                            .panel_test_id("ui-gallery-ai-prompt-input-docs-search-tooltip-panel"),
                    )
                    .variant(if searching {
                        shadcn::ButtonVariant::Default
                    } else {
                        shadcn::ButtonVariant::Ghost
                    })
                    .test_id("ui-gallery-ai-prompt-input-docs-search")
                    .action(act::ToggleSearch)
                    .into_element(cx);

                let select = ui_ai::PromptInputSelect::new(model_value.clone(), model_open.clone())
                    .trigger_test_id("ui-gallery-ai-prompt-input-docs-model-trigger")
                    .on_value_change({
                        let model_value = model_value.clone();
                        move |host, _action_cx, value| {
                            let _ = host
                                .models_mut()
                                .update(&model_value, |current| *current = Some(value));
                        }
                    })
                    .trigger(ui_ai::PromptInputSelectTrigger::new())
                    .value(ui_ai::PromptInputSelectValue::new().placeholder("Model"))
                    .content(ui_ai::PromptInputSelectContent::new())
                    .entries([
                        ui_ai::PromptInputSelectItem::new("gpt-4o", "GPT-4o"),
                        ui_ai::PromptInputSelectItem::new(
                            "claude-opus-4-20250514",
                            "Claude 4 Opus",
                        ),
                    ])
                    .into_element(cx);

                let input = ui_ai::PromptInput::new(controller.text)
                    .on_submit(on_submit)
                    .on_add_attachments(on_add_attachments)
                    .on_add_screenshot(on_add_screenshot)
                    .test_id_root("ui-gallery-ai-prompt-input-docs")
                    .test_id_send("ui-gallery-ai-prompt-input-docs-send")
                    .test_id_stop("ui-gallery-ai-prompt-input-docs-stop")
                    .children([
                        ui_ai::PromptInputPart::from(ui_ai::PromptInputHeader::new([
                            ui_ai::PromptInputAttachmentsRow::new(),
                        ])),
                        ui_ai::PromptInputPart::from(ui_ai::PromptInputBody::new([
                            ui_ai::PromptInputTextarea::new()
                                .placeholder("What would you like to know?")
                                .test_id("ui-gallery-ai-prompt-input-docs-textarea"),
                        ])),
                        ui_ai::PromptInputPart::from(ui_ai::PromptInputFooter::new(
                            [ui_ai::PromptInputTools::empty()
                                .child(menu)
                                .child(search_btn)
                                .child(select)],
                            [ui_ai::PromptInputSubmit::new()
                                .refine_layout(LayoutRefinement::default().ml_auto())],
                        )),
                    ])
                    .into_element(cx);

                vec![input]
            });

        ui::v_flex(move |_cx| vec![conversation, prompt])
            .layout(LayoutRefinement::default().w_full().h_full().min_h_0())
            .gap(Space::N4)
            .into_element(cx)
    };

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
                .h_px(Px(560.0))
                .min_w_0()
                .min_h_0(),
        )
    });

    let frame = cx.container(props, move |_cx| vec![body]);

    ui::v_flex(move |cx| {
        vec![
            cx.text("Prompt Input (AI Elements)"),
            cx.text(
                "Docs-aligned chat example: transcript + prompt composer, add attachments/screenshot actions, model picker, and upstream-like onSubmit(message).",
            ),
            frame,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
