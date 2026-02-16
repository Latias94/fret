use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_conversation_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui::action::OnActivate;
    use fret_ui::scroll::VirtualListScrollHandle;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

    #[derive(Default)]
    struct DemoModels {
        messages: Option<Model<Arc<[ui_ai::ConversationMessage]>>>,
        next_id: Option<Model<u64>>,
        exported_len: Option<Model<Option<usize>>>,
    }

    let needs_init = cx.with_state(DemoModels::default, |st| {
        st.messages.is_none() || st.next_id.is_none() || st.exported_len.is_none()
    });
    if needs_init {
        let initial: Arc<[ui_ai::ConversationMessage]> = Arc::from([
            ui_ai::ConversationMessage::new(
                1,
                ui_ai::MessageRole::User,
                "Hello! This is the Conversation surface.",
            ),
            ui_ai::ConversationMessage::new(
                2,
                ui_ai::MessageRole::Assistant,
                "Messages are virtualized and stick-to-bottom when eligible.",
            ),
            ui_ai::ConversationMessage::new(
                3,
                ui_ai::MessageRole::Assistant,
                "Scroll up to see the scroll-to-bottom button appear.",
            ),
        ]);

        let messages = cx.app.models_mut().insert(initial);
        let next_id = cx.app.models_mut().insert(4u64);
        let exported_len = cx.app.models_mut().insert(None::<usize>);

        cx.with_state(DemoModels::default, move |st| {
            st.messages = Some(messages.clone());
            st.next_id = Some(next_id.clone());
            st.exported_len = Some(exported_len.clone());
        });
    }

    let (messages_model, next_id_model, exported_len_model) =
        cx.with_state(DemoModels::default, |st| {
            (
                st.messages.clone().expect("messages"),
                st.next_id.clone().expect("next_id"),
                st.exported_len.clone().expect("exported_len"),
            )
        });

    let messages = cx
        .get_model_cloned(&messages_model, Invalidation::Layout)
        .unwrap_or_else(|| Arc::from([]));
    let revision = messages.len().min(u64::MAX as usize) as u64;
    let messages_empty = messages.is_empty();

    let append_user: OnActivate = Arc::new({
        let messages_model = messages_model.clone();
        let next_id_model = next_id_model.clone();
        move |host, acx, _reason| {
            let id = host.models_mut().get_copied(&next_id_model).unwrap_or(1);
            let _ = host
                .models_mut()
                .update(&next_id_model, |v| *v = v.saturating_add(1));

            let existing = host
                .models_mut()
                .get_cloned(&messages_model)
                .unwrap_or_else(|| Arc::from([]));
            let mut out: Vec<ui_ai::ConversationMessage> = existing.iter().cloned().collect();
            out.push(ui_ai::ConversationMessage::new(
                id,
                ui_ai::MessageRole::User,
                format!("User message #{id}"),
            ));
            let out: Arc<[ui_ai::ConversationMessage]> = out.into();
            let _ = host.models_mut().update(&messages_model, |v| *v = out);
            host.request_redraw(acx.window);
        }
    });

    let append_assistant: OnActivate = Arc::new({
        let messages_model = messages_model.clone();
        let next_id_model = next_id_model.clone();
        move |host, acx, _reason| {
            let id = host.models_mut().get_copied(&next_id_model).unwrap_or(1);
            let _ = host
                .models_mut()
                .update(&next_id_model, |v| *v = v.saturating_add(1));

            let existing = host
                .models_mut()
                .get_cloned(&messages_model)
                .unwrap_or_else(|| Arc::from([]));
            let mut out: Vec<ui_ai::ConversationMessage> = existing.iter().cloned().collect();
            out.push(ui_ai::ConversationMessage::new(
                id,
                ui_ai::MessageRole::Assistant,
                format!("Assistant message #{id}: appended content for scroll + virtualization."),
            ));
            let out: Arc<[ui_ai::ConversationMessage]> = out.into();
            let _ = host.models_mut().update(&messages_model, |v| *v = out);
            host.request_redraw(acx.window);
        }
    });

    let clear: OnActivate = Arc::new({
        let messages_model = messages_model.clone();
        move |host, acx, _reason| {
            let _ = host
                .models_mut()
                .update(&messages_model, |v| *v = Arc::from([]));
            host.request_redraw(acx.window);
        }
    });

    let download: OnActivate = Arc::new({
        let messages_model = messages_model.clone();
        let exported_len_model = exported_len_model.clone();
        move |host, acx, _reason| {
            let count = host
                .models_mut()
                .get_cloned(&messages_model)
                .map(|v| v.len())
                .unwrap_or(0);
            let _ = host
                .models_mut()
                .update(&exported_len_model, |v| *v = Some(count));
            host.request_redraw(acx.window);
        }
    });

    let exported_marker = cx
        .get_model_cloned(&exported_len_model, Invalidation::Paint)
        .flatten()
        .map(|len| cx.text(format!("exported_len={len}")))
        .unwrap_or_else(|| cx.text("exported_len=<none>"))
        .test_id("ui-ai-conversation-demo-exported-len");

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        move |cx| {
            vec![
                cx.text("Conversation (AI Elements): transcript + scroll-to-bottom affordance."),
                exported_marker,
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N2),
                    move |cx| {
                        vec![
                            fret_ui_shadcn::Button::new("Append user")
                                .test_id("ui-ai-conversation-demo-append-user")
                                .on_activate(append_user.clone())
                                .into_element(cx),
                            fret_ui_shadcn::Button::new("Append assistant")
                                .test_id("ui-ai-conversation-demo-append-assistant")
                                .on_activate(append_assistant.clone())
                                .into_element(cx),
                            fret_ui_shadcn::Button::new("Clear")
                                .test_id("ui-ai-conversation-demo-clear")
                                .on_activate(clear.clone())
                                .into_element(cx),
                            ui_ai::ConversationDownload::new("Download")
                                .disabled(messages_empty)
                                .on_activate(download.clone())
                                .test_id("ui-ai-conversation-demo-download")
                                .into_element(cx),
                        ]
                    },
                ),
            ]
        },
    );

    let body = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

        let content: AnyElement = if messages.is_empty() {
            ui_ai::ConversationEmptyState::new("Start a conversation")
                .description("Type a message below to begin chatting (demo state).")
                .test_id("ui-ai-conversation-demo-empty")
                .into_element(cx)
        } else {
            ui_ai::ConversationTranscript::from_arc(messages.clone())
                .content_revision(revision)
                .scroll_handle(scroll_handle.clone())
                .stick_to_bottom(true)
                .show_scroll_to_bottom_button(false)
                .debug_root_test_id("ui-ai-conversation-demo-transcript-root")
                .debug_row_test_id_prefix("ui-ai-conversation-demo-row-")
                .into_element(cx)
        };

        let scroll_button = ui_ai::ConversationScrollButton::new(scroll_handle)
            .test_id("ui-ai-conversation-demo-scroll-bottom")
            .into_element(cx);

        let layout = decl_style::layout_style(
            theme,
            LayoutRefinement::default().w_full().h_full().relative(),
        );

        vec![
            cx.stack_props(fret_ui::element::StackProps { layout }, |_cx| {
                vec![content, scroll_button]
            }),
        ]
    });

    let chrome = ChromeRefinement::default()
        .rounded(Radius::Lg)
        .border_1()
        .border_color(ColorRef::Color(theme.color_token("border")));
    let props = decl_style::container_props(
        theme,
        chrome,
        LayoutRefinement::default()
            .w_full()
            .h_px(Px(560.0))
            .min_w_0()
            .min_h_0(),
    );

    vec![header, cx.container(props, move |_cx| vec![body])]
}
