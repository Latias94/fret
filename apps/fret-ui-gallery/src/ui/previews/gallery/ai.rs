use super::super::super::*;

pub(in crate::ui) fn preview_ai_transcript_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;

    let variable_height = std::env::var_os("FRET_UI_GALLERY_AI_TRANSCRIPT_VARIABLE_HEIGHT")
        .filter(|v| !v.is_empty())
        .is_some();
    let message_count = std::env::var("FRET_UI_GALLERY_AI_TRANSCRIPT_LEN")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5_000);
    let append_batch: usize = 100;

    #[derive(Default)]
    struct TranscriptModels {
        messages: Option<Model<Arc<[ui_ai::ConversationMessage]>>>,
    }

    let message_text = |i: u64| -> Arc<str> {
        if variable_height && i % 7 == 0 {
            Arc::<str>::from(format!(
                "Message {i}\nDetails: seed={} tokens={} latency={}ms",
                (i * 31) % 97,
                16 + (i % 64),
                10 + (i % 120)
            ))
        } else {
            Arc::<str>::from(format!("Message {i}: hello world"))
        }
    };

    let messages_model = cx.with_state(TranscriptModels::default, |st| st.messages.clone());
    let messages_model = match messages_model {
        Some(model) => model,
        None => {
            let mut out: Vec<ui_ai::ConversationMessage> = Vec::with_capacity(message_count);
            for i in 0..message_count as u64 {
                let role = match i % 4 {
                    0 => ui_ai::MessageRole::User,
                    1 => ui_ai::MessageRole::Assistant,
                    2 => ui_ai::MessageRole::Tool,
                    _ => ui_ai::MessageRole::System,
                };
                out.push(ui_ai::ConversationMessage::new(i, role, message_text(i)));
            }

            let out: Arc<[ui_ai::ConversationMessage]> = Arc::from(out);
            let model = cx.app.models_mut().insert(out);
            cx.with_state(TranscriptModels::default, |st| {
                st.messages = Some(model.clone())
            });
            model
        }
    };
    let messages = cx
        .get_model_cloned(&messages_model, Invalidation::Layout)
        .unwrap_or_else(|| Arc::from([]));

    let append_messages_on_activate: OnActivate = {
        let messages_model = messages_model.clone();
        Arc::new(move |host, acx, _reason| {
            let existing = host
                .models_mut()
                .get_cloned(&messages_model)
                .unwrap_or_else(|| Arc::from([]));
            let start = existing.len() as u64;

            let mut out: Vec<ui_ai::ConversationMessage> = existing.iter().cloned().collect();
            out.reserve(append_batch);
            for i in start..start + append_batch as u64 {
                let role = match i % 4 {
                    0 => ui_ai::MessageRole::User,
                    1 => ui_ai::MessageRole::Assistant,
                    2 => ui_ai::MessageRole::Tool,
                    _ => ui_ai::MessageRole::System,
                };
                let text = if variable_height && i % 5 == 0 {
                    Arc::<str>::from(format!("Appended {i}\n(extra line)"))
                } else {
                    Arc::<str>::from(format!("Appended {i}"))
                };
                out.push(ui_ai::ConversationMessage::new(i, role, text));
            }

            let out: Arc<[ui_ai::ConversationMessage]> = Arc::from(out);
            let _ = host.models_mut().update(&messages_model, |v| *v = out);
            host.request_redraw(acx.window);
        })
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline harness for long AI transcripts (scrolling + virtualization + caching)."),
                cx.text("Use scripted wheel-scroll to validate view-cache reuse stability and stale-paint safety."),
                fret_ui_shadcn::Button::new(format!("Append {append_batch} messages"))
                    .test_id("ui-gallery-ai-transcript-append")
                    .on_activate(append_messages_on_activate)
                    .into_element(cx),
            ]
        },
    );

    let transcript =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());
            let revision = messages.len().min(u64::MAX as usize) as u64;

            let transcript = ui_ai::ConversationTranscript::from_arc(messages.clone())
                .content_revision(revision)
                .scroll_handle(scroll_handle.clone())
                .stick_to_bottom(false)
                .show_scroll_to_bottom_button(false)
                .debug_root_test_id("ui-gallery-ai-transcript-root")
                .debug_row_test_id_prefix("ui-gallery-ai-transcript-row-")
                .into_element(cx);

            let scroll_button = ui_ai::ConversationScrollButton::new(scroll_handle)
                .test_id("ui-gallery-ai-transcript-scroll-bottom")
                .into_element(cx);

            let layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default().w_full().h_full().relative(),
            );

            vec![
                cx.stack_props(fret_ui::element::StackProps { layout }, |_cx| {
                    vec![transcript, scroll_button]
                }),
            ]
        });

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![
        header,
        cx.container(container_props, |_cx| vec![transcript]),
    ]
}

pub(in crate::ui) fn preview_ai_chat_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui::action::OnActivate;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Debug, Clone)]
    struct PendingReply {
        assistant_id: u64,
        chunks: Arc<[Arc<str>]>,
        next_chunk: usize,
        markdown: Arc<str>,
        tool_call_running: ui_ai::ToolCall,
        tool_call_final: ui_ai::ToolCall,
        sources: Arc<[ui_ai::SourceItem]>,
        citations: Arc<[ui_ai::CitationItem]>,
    }

    #[derive(Default)]
    struct ChatModels {
        prompt: Option<Model<String>>,
        messages: Option<Model<Arc<[ui_ai::AiMessage]>>>,
        loading: Option<Model<bool>>,
        pending: Option<Model<Option<PendingReply>>>,
        next_id: Option<Model<u64>>,
        content_revision: Option<Model<u64>>,
        exported_md_len: Option<Model<Option<usize>>>,
    }

    let prompt = cx.with_state(ChatModels::default, |st| st.prompt.clone());
    let prompt = match prompt {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(ChatModels::default, |st| st.prompt = Some(model.clone()));
            model
        }
    };

    let messages = cx.with_state(ChatModels::default, |st| st.messages.clone());
    let messages = match messages {
        Some(model) => model,
        None => {
            let sources: Arc<[ui_ai::SourceItem]> = Arc::from(vec![
                ui_ai::SourceItem::new("src-0", "Example source A")
                    .url("https://example.com/a")
                    .excerpt("A short excerpt used for truncation and wrapping tests."),
                ui_ai::SourceItem::new("src-1", "Example source B")
                    .url("https://example.com/b")
                    .excerpt("Another excerpt: this should wrap and remain readable."),
            ]);

            let citations: Arc<[ui_ai::CitationItem]> = Arc::from(vec![
                ui_ai::CitationItem::new("src-0", "[1]"),
                ui_ai::CitationItem::from_arc(
                    Arc::from(vec![Arc::<str>::from("src-0"), Arc::<str>::from("src-1")]),
                    "[2]",
                ),
            ]);

            let tool_call = ui_ai::ToolCall::new("toolcall-seed-0", "search")
                .state(ui_ai::ToolCallState::InputAvailable)
                .input(ui_ai::ToolCallPayload::Json(serde_json::json!({
                    "query": "seeded tool call",
                    "k": 3
                })));

            let initial: Arc<[ui_ai::AiMessage]> = Arc::from(vec![
                ui_ai::AiMessage::new(
                    1,
                    ui_ai::MessageRole::User,
                    [ui_ai::MessagePart::Text(Arc::<str>::from("Hello!"))],
                ),
                ui_ai::AiMessage::new(
                    2,
                    ui_ai::MessageRole::Assistant,
                    [ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::new(
                        Arc::<str>::from(
                            "This is a small demo for `PromptInput` + transcript append.\n\nIt also exercises tool calls + sources blocks.\n\n```rust\nfn demo() {\n    println!(\"hello from code fence\");\n}\n```",
                        ),
                    ))],
                ),
                ui_ai::AiMessage::new(
                    3,
                    ui_ai::MessageRole::User,
                    [ui_ai::MessagePart::Text(Arc::<str>::from(
                        "Show me seeded tools + sources + citations.",
                    ))],
                ),
                ui_ai::AiMessage::new(
                    4,
                    ui_ai::MessageRole::Assistant,
                    [
                        ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::streaming(
                            Arc::<str>::from(""),
                        )),
                        ui_ai::MessagePart::ToolCall(tool_call),
                        ui_ai::MessagePart::Sources(sources),
                        ui_ai::MessagePart::Citations(citations),
                    ],
                ),
            ]);
            let model = cx.app.models_mut().insert(initial);
            cx.with_state(ChatModels::default, |st| st.messages = Some(model.clone()));
            model
        }
    };

    let loading = cx.with_state(ChatModels::default, |st| st.loading.clone());
    let loading = match loading {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ChatModels::default, |st| st.loading = Some(model.clone()));
            model
        }
    };

    let pending = cx.with_state(ChatModels::default, |st| st.pending.clone());
    let pending = match pending {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<PendingReply>);
            cx.with_state(ChatModels::default, |st| st.pending = Some(model.clone()));
            model
        }
    };

    let next_id = cx.with_state(ChatModels::default, |st| st.next_id.clone());
    let next_id = match next_id {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(5u64);
            cx.with_state(ChatModels::default, |st| st.next_id = Some(model.clone()));
            model
        }
    };

    let content_revision = cx.with_state(ChatModels::default, |st| st.content_revision.clone());
    let content_revision = match content_revision {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(0u64);
            cx.with_state(ChatModels::default, |st| {
                st.content_revision = Some(model.clone())
            });
            model
        }
    };

    let exported_md_len = cx.with_state(ChatModels::default, |st| st.exported_md_len.clone());
    let exported_md_len = match exported_md_len {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<usize>);
            cx.with_state(ChatModels::default, |st| {
                st.exported_md_len = Some(model.clone())
            });
            model
        }
    };

    let prompt_non_empty = cx
        .get_model_cloned(&prompt, Invalidation::Paint)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let prompt_non_empty_marker = prompt_non_empty.then(|| {
        cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-gallery-ai-chat-prompt-nonempty")),
                ..Default::default()
            },
            |cx| {
                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: fret_ui::element::LayoutStyle {
                            size: fret_ui::element::SizeStyle {
                                width: fret_ui::element::Length::Px(Px(0.0)),
                                height: fret_ui::element::Length::Px(Px(0.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                )]
            },
        )
    });

    let loading_value = cx
        .get_model_copied(&loading, Invalidation::Paint)
        .unwrap_or(false);
    let pending_value = cx
        .get_model_cloned(&pending, Invalidation::Paint)
        .unwrap_or(None);

    if loading_value {
        if let Some(pending_state) = pending_value {
            if pending_state.next_chunk < pending_state.chunks.len() {
                cx.request_frame();

                if let Some(chunk) = pending_state.chunks.get(pending_state.next_chunk).cloned() {
                    let new_markdown =
                        Arc::<str>::from(format!("{}{}", pending_state.markdown, chunk));

                    let _ = cx.app.models_mut().update(&pending, |v| {
                        if let Some(p) = v {
                            p.markdown = new_markdown.clone();
                            p.next_chunk = p.next_chunk.saturating_add(1);
                        }
                    });

                    let assistant_id = pending_state.assistant_id;
                    let tool_call_running = pending_state.tool_call_running.clone();
                    let sources = pending_state.sources.clone();
                    let citations = pending_state.citations.clone();

                    let _ = cx.app.models_mut().update(&messages, |list| {
                        let mut vec = list.as_ref().to_vec();
                        if let Some(msg) = vec.iter_mut().find(|m| m.id == assistant_id) {
                            msg.parts = Arc::from(vec![
                                ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::streaming(
                                    new_markdown.clone(),
                                )),
                                ui_ai::MessagePart::ToolCall(tool_call_running),
                                ui_ai::MessagePart::Sources(sources),
                                ui_ai::MessagePart::Citations(citations),
                            ]);
                        }
                        *list = vec.into();
                    });
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&content_revision, |v| *v = v.saturating_add(1));
                } else {
                    let _ = cx.app.models_mut().update(&pending, |v| *v = None);
                    let _ = cx.app.models_mut().update(&loading, |v| *v = false);
                }
            } else {
                let assistant_id = pending_state.assistant_id;
                let markdown = pending_state.markdown.clone();
                let tool_call_final = pending_state.tool_call_final.clone();
                let sources = pending_state.sources.clone();
                let citations = pending_state.citations.clone();

                let _ = cx.app.models_mut().update(&messages, |list| {
                    let mut vec = list.as_ref().to_vec();
                    if let Some(msg) = vec.iter_mut().find(|m| m.id == assistant_id) {
                        msg.parts = Arc::from(vec![
                            ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::new(markdown)),
                            ui_ai::MessagePart::ToolCall(tool_call_final),
                            ui_ai::MessagePart::Sources(sources),
                            ui_ai::MessagePart::Citations(citations),
                        ]);
                    }
                    *list = vec.into();
                });
                let _ = cx
                    .app
                    .models_mut()
                    .update(&content_revision, |v| *v = v.saturating_add(1));

                let _ = cx.app.models_mut().update(&pending, |v| *v = None);
                let _ = cx.app.models_mut().update(&loading, |v| *v = false);
            }
        }
    }

    let send: OnActivate = Arc::new({
        let prompt = prompt.clone();
        let messages = messages.clone();
        let pending = pending.clone();
        let loading = loading.clone();
        let next_id = next_id.clone();
        let content_revision = content_revision.clone();
        move |host, _action_cx, _reason| {
            fn chunk_for_demo(text: &str, chars_per_chunk: usize) -> Arc<[Arc<str>]> {
                let mut out = Vec::new();
                let mut buf = String::new();
                let mut count = 0usize;

                for ch in text.chars() {
                    buf.push(ch);
                    count = count.saturating_add(1);
                    if count >= chars_per_chunk {
                        out.push(Arc::<str>::from(std::mem::take(&mut buf)));
                        count = 0;
                    }
                }

                if !buf.is_empty() {
                    out.push(Arc::<str>::from(buf));
                }

                out.into()
            }

            let text = host.models_mut().read(&prompt, Clone::clone).ok();
            let Some(text) = text else { return };
            let text = text.trim().to_string();
            if text.is_empty() {
                return;
            }

            let user_id = host
                .models_mut()
                .update(&next_id, |v| {
                    let id = *v;
                    *v = v.saturating_add(1);
                    id
                })
                .ok()
                .unwrap_or(0);
            let assistant_id = host
                .models_mut()
                .update(&next_id, |v| {
                    let id = *v;
                    *v = v.saturating_add(1);
                    id
                })
                .ok()
                .unwrap_or(0);

            let tool_call = ui_ai::ToolCall::new("toolcall-0", "search")
                .state(ui_ai::ToolCallState::InputAvailable)
                .input(ui_ai::ToolCallPayload::Json(serde_json::json!({
                    "query": text,
                    "k": 3
                })));

            let sources: Arc<[ui_ai::SourceItem]> = Arc::from(vec![
                ui_ai::SourceItem::new("src-0", "Example source A")
                    .url("https://example.com/a")
                    .excerpt("A short excerpt used for truncation and wrapping tests."),
                ui_ai::SourceItem::new("src-1", "Example source B")
                    .url("https://example.com/b")
                    .excerpt("Another excerpt: this should wrap and remain readable."),
            ]);

            let citations: Arc<[ui_ai::CitationItem]> = Arc::from(vec![
                ui_ai::CitationItem::new("src-0", "[1]"),
                ui_ai::CitationItem::from_arc(
                    Arc::from(vec![Arc::<str>::from("src-0"), Arc::<str>::from("src-1")]),
                    "[2]",
                ),
            ]);

            let reply = format!(
                "Echo: **{text}**\n\nThis reply is streamed via append-only updates.\n\n```rust\nfn streamed_demo() {{\n    println!(\"{text}\");\n}}\n"
            );
            let chunks = chunk_for_demo(&reply, 12);

            let tool_call_final = tool_call
                .clone()
                .state(ui_ai::ToolCallState::OutputAvailable)
                .output(ui_ai::ToolCallPayload::Json(serde_json::json!({
                    "results": [
                        {"title": "A", "score": 0.9},
                        {"title": "B", "score": 0.8}
                    ]
                })));

            let _ = host.models_mut().update(&messages, |list| {
                let mut vec = list.as_ref().to_vec();
                vec.push(ui_ai::AiMessage::new(
                    user_id,
                    ui_ai::MessageRole::User,
                    [ui_ai::MessagePart::Text(Arc::<str>::from(text))],
                ));
                vec.push(ui_ai::AiMessage::new(
                    assistant_id,
                    ui_ai::MessageRole::Assistant,
                    [
                        ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::streaming(
                            Arc::<str>::from(""),
                        )),
                        ui_ai::MessagePart::ToolCall(tool_call.clone()),
                        ui_ai::MessagePart::Sources(sources.clone()),
                        ui_ai::MessagePart::Citations(citations.clone()),
                    ],
                ));
                *list = vec.into();
            });
            let _ = host
                .models_mut()
                .update(&content_revision, |v| *v = v.saturating_add(1));

            let _ = host.models_mut().update(&pending, |v| {
                *v = Some(PendingReply {
                    assistant_id,
                    chunks,
                    next_chunk: 0,
                    markdown: Arc::<str>::from(""),
                    tool_call_running: tool_call,
                    tool_call_final,
                    sources,
                    citations,
                })
            });
            let _ = host.models_mut().update(&loading, |v| *v = true);
        }
    });

    let stop: OnActivate = Arc::new({
        let messages = messages.clone();
        let pending = pending.clone();
        let loading = loading.clone();
        let content_revision = content_revision.clone();
        move |host, _action_cx, _reason| {
            let assistant_id = host
                .models_mut()
                .read(&pending, |v| v.as_ref().map(|p| p.assistant_id))
                .ok()
                .flatten();

            let _ = host.models_mut().update(&pending, |v| *v = None);
            let _ = host.models_mut().update(&loading, |v| *v = false);

            let Some(assistant_id) = assistant_id else {
                return;
            };
            let _ = host.models_mut().update(&messages, |list| {
                let vec: Vec<_> = list
                    .iter()
                    .cloned()
                    .filter(|m| m.id != assistant_id)
                    .collect();
                *list = vec.into();
            });
            let _ = host
                .models_mut()
                .update(&content_revision, |v| *v = v.saturating_add(1));
        }
    });

    let export_markdown: OnActivate = Arc::new({
        let messages = messages.clone();
        let exported_md_len = exported_md_len.clone();
        move |host, _action_cx, _reason| {
            let messages = host.models_mut().read(&messages, Clone::clone).ok();
            let Some(messages) = messages else {
                return;
            };

            let md = ui_ai::messages_to_markdown(messages.as_ref());
            let _ = host
                .models_mut()
                .update(&exported_md_len, |v| *v = Some(md.len()));
        }
    });

    let start_streaming: OnActivate = Arc::new({
        let messages = messages.clone();
        let pending = pending.clone();
        let loading = loading.clone();
        let content_revision = content_revision.clone();
        move |host, _action_cx, _reason| {
            fn chunk_for_demo(text: &str, chars_per_chunk: usize) -> Arc<[Arc<str>]> {
                let mut out = Vec::new();
                let mut buf = String::new();
                let mut count = 0usize;

                for ch in text.chars() {
                    buf.push(ch);
                    count = count.saturating_add(1);
                    if count >= chars_per_chunk {
                        out.push(Arc::<str>::from(std::mem::take(&mut buf)));
                        count = 0;
                    }
                }

                if !buf.is_empty() {
                    out.push(Arc::<str>::from(buf));
                }

                out.into()
            }

            let sources: Arc<[ui_ai::SourceItem]> = Arc::from(vec![
                ui_ai::SourceItem::new("src-0", "Example source A")
                    .url("https://example.com/a")
                    .excerpt("A short excerpt used for truncation and wrapping tests."),
                ui_ai::SourceItem::new("src-1", "Example source B")
                    .url("https://example.com/b")
                    .excerpt("Another excerpt: this should wrap and remain readable."),
            ]);

            let citations: Arc<[ui_ai::CitationItem]> = Arc::from(vec![
                ui_ai::CitationItem::new("src-0", "[1]"),
                ui_ai::CitationItem::from_arc(
                    Arc::from(vec![Arc::<str>::from("src-0"), Arc::<str>::from("src-1")]),
                    "[2]",
                ),
            ]);

            let tool_call_running = ui_ai::ToolCall::new("toolcall-seed-0", "search")
                .state(ui_ai::ToolCallState::InputAvailable)
                .input(ui_ai::ToolCallPayload::Json(serde_json::json!({
                    "query": "seeded tool call",
                    "k": 3
                })));

            let tool_call_final = tool_call_running
                .clone()
                .state(ui_ai::ToolCallState::OutputAvailable)
                .output(ui_ai::ToolCallPayload::Json(serde_json::json!({
                    "results": [
                        {"title": "A", "score": 0.9},
                        {"title": "B", "score": 0.8}
                    ]
                })));

            let reply = "This assistant message is streamed in append-only chunks.\n\n```rust\nfn streamed_demo() {\n    println!(\"hello from stream\");\n}\n```\n";
            let chunks = chunk_for_demo(reply, 12);

            let assistant_id = 4u64;

            let _ = host.models_mut().update(&messages, |list| {
                let mut vec = list.as_ref().to_vec();
                if let Some(msg) = vec.iter_mut().find(|m| m.id == assistant_id) {
                    msg.parts = Arc::from(vec![
                        ui_ai::MessagePart::Markdown(ui_ai::MarkdownPart::streaming(
                            Arc::<str>::from(""),
                        )),
                        ui_ai::MessagePart::ToolCall(tool_call_running.clone()),
                        ui_ai::MessagePart::Sources(sources.clone()),
                        ui_ai::MessagePart::Citations(citations.clone()),
                    ]);
                }
                *list = vec.into();
            });

            let _ = host.models_mut().update(&pending, |v| {
                *v = Some(PendingReply {
                    assistant_id,
                    chunks,
                    next_chunk: 0,
                    markdown: Arc::<str>::from(""),
                    tool_call_running,
                    tool_call_final,
                    sources,
                    citations,
                })
            });

            let _ = host.models_mut().update(&loading, |v| *v = true);
            let _ = host
                .models_mut()
                .update(&content_revision, |v| *v = v.saturating_add(1));
        }
    });

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: interactive demo for PromptInput + transcript append."),
                cx.text("Send triggers a short \"loading\" window where Stop is available."),
                shadcn::Button::new("Start streaming (seeded)")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-ai-chat-start-stream")
                    .on_activate(start_streaming.clone())
                    .into_element(cx),
            ]
        },
    );

    let actions_demo = {
        let copy = ui_ai::MessageAction::new("Copy")
            .tooltip("Copy")
            .test_id("ui-gallery-ai-chat-action-copy")
            .children([shadcn::icon::icon(
                cx,
                fret_icons::IconId::new_static("lucide.copy"),
            )])
            .into_element(cx);

        ui_ai::MessageActions::new([copy])
            .test_id("ui-gallery-ai-chat-actions")
            .into_element(cx)
    };

    let chat = ui_ai::AiChat::new(messages.clone(), prompt)
        .loading_model(loading.clone())
        .content_revision_model(content_revision.clone())
        .on_send(send)
        .on_stop(stop)
        .show_download(true)
        .on_download(export_markdown)
        .download_test_id("ui-gallery-ai-chat-download")
        .message_test_id_prefix("ui-ai-msg-")
        .transcript_root_test_id("ui-gallery-ai-chat-transcript-root")
        .transcript_row_test_id_prefix("ui-gallery-ai-chat-transcript-row-")
        .scroll_button_test_id("ui-gallery-ai-chat-scroll-bottom")
        .prompt_root_test_id("ui-gallery-ai-chat-prompt-root")
        .prompt_textarea_test_id("ui-gallery-ai-chat-prompt-textarea")
        .prompt_send_test_id("ui-gallery-ai-chat-prompt-send")
        .prompt_stop_test_id("ui-gallery-ai-chat-prompt-stop")
        .transcript_container_layout(LayoutRefinement::default().w_full().h_px(Px(360.0)))
        .into_element(cx);

    let exported_value = cx
        .get_model_cloned(&exported_md_len, Invalidation::Paint)
        .unwrap_or(None);
    let exported = exported_value.map(|len| {
        cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(Arc::<str>::from("ui-gallery-ai-chat-exported-md-len")),
                ..Default::default()
            },
            move |cx| vec![cx.text(format!("Exported markdown: {len} chars"))],
        )
    });

    vec![
        header,
        actions_demo,
        chat,
        prompt_non_empty_marker.unwrap_or_else(|| cx.text("")),
        exported.unwrap_or_else(|| cx.text("")),
    ]
}

pub(in crate::ui) fn preview_ai_file_tree_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::collections::HashSet;
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::action::ActionCx;
    use fret_ui::element::SemanticsProps;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Default)]
    struct FileTreeModels {
        expanded: Option<Model<HashSet<Arc<str>>>>,
        selected: Option<Model<Option<Arc<str>>>>,
    }

    let expanded = cx.with_state(FileTreeModels::default, |st| st.expanded.clone());
    let expanded = match expanded {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(HashSet::<Arc<str>>::new());
            cx.with_state(FileTreeModels::default, |st| {
                st.expanded = Some(model.clone())
            });
            model
        }
    };

    let selected = cx.with_state(FileTreeModels::default, |st| st.selected.clone());
    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(FileTreeModels::default, |st| {
                st.selected = Some(model.clone())
            });
            model
        }
    };

    let selected_value = cx.watch_model(&selected).layout().cloned().flatten();

    let tree = ui_ai::FileTree::new([
        ui_ai::FileTreeFolder::new("src", "src")
            .test_id("ui-ai-file-tree-folder-src")
            .children([
                ui_ai::FileTreeFile::new("src/lib.rs", "lib.rs")
                    .test_id("ui-ai-file-tree-file-lib")
                    .into(),
                ui_ai::FileTreeFile::new("src/main.rs", "main.rs")
                    .test_id("ui-ai-file-tree-file-main")
                    .into(),
            ])
            .into(),
        ui_ai::FileTreeFile::new("Cargo.toml", "Cargo.toml")
            .test_id("ui-ai-file-tree-file-cargo-toml")
            .into(),
        ui_ai::FileTreeFolder::new("tests", "tests")
            .test_id("ui-ai-file-tree-folder-tests")
            .child(
                ui_ai::FileTreeFile::new("tests/file_tree.rs", "file_tree.rs")
                    .test_id("ui-ai-file-tree-file-tests-file-tree"),
            )
            .into(),
    ])
    .expanded_paths(expanded.clone())
    .selected_path(selected_value.clone())
    .on_select(Arc::new({
        let selected = selected.clone();
        move |host, _action_cx: ActionCx, path| {
            let _ = host.models_mut().update(&selected, |v| *v = Some(path));
        }
    }))
    .test_id_root("ui-ai-file-tree-root")
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let selected_label_text = selected_value
        .as_deref()
        .map(|s| format!("Selected: {s}"))
        .unwrap_or_else(|| "Selected: <none>".to_string());

    let selected_label = cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Text,
            test_id: Some(Arc::<str>::from("ui-ai-file-tree-selected-label")),
            ..Default::default()
        },
        move |cx| vec![cx.text(selected_label_text)],
    );

    let selected_marker = (selected_value.as_deref() == Some("src/lib.rs")).then(|| {
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Generic,
                test_id: Some(Arc::<str>::from("ui-ai-file-tree-selected-marker")),
                ..Default::default()
            },
            move |_cx| vec![],
        )
    });

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3),
        move |cx| {
            vec![
                cx.text("FileTree (AI Elements)"),
                tree,
                selected_label,
                selected_marker.unwrap_or_else(|| cx.text("")),
            ]
        },
    )]
}
