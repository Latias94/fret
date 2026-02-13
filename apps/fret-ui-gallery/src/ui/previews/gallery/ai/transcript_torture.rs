use super::super::super::super::*;

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
