pub const SOURCE: &str = include_str!("conversation_demo.rs");

// region: example
use fret_core::{Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::SemanticsProps;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, Justify, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::{AnyElement, ElementContext, UiHost};
use std::sync::Arc;

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
        let messages = cx
            .app
            .models_mut()
            .insert(Arc::<[ui_ai::AiMessage]>::from([]));
        let prompt = cx.app.models_mut().insert(String::new());
        let next_id = cx.app.models_mut().insert(4u64);
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

    let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

    let transcript: AnyElement = if messages_empty {
        ui_ai::ConversationEmptyState::new("Start a conversation")
            .description("Type a message below to begin chatting.")
            .test_id("ui-ai-conversation-demo-empty")
            .into_element(cx)
    } else {
        ui_ai::AiConversationTranscript::from_arc(messages.clone())
            .content_revision(revision)
            .scroll_handle(scroll_handle.clone())
            .stick_to_bottom(true)
            .test_id_message_prefix("ui-ai-conversation-demo-msg-")
            .debug_root_test_id("ui-ai-conversation-demo-transcript-root")
            .debug_row_test_id_prefix("ui-ai-conversation-demo-row-")
            .into_element(cx)
    };

    let scroll_button = ui_ai::ConversationScrollButton::new(scroll_handle.clone())
        .test_id("ui-ai-conversation-demo-scroll-bottom")
        .into_element(cx);

    let download_button = ui_ai::ConversationDownload::new("Download conversation")
        .disabled(messages_empty)
        .on_activate(download.clone())
        .test_id("ui-ai-conversation-demo-download")
        .into_element(cx);

    let transcript_stack_layout = cx.with_theme(|theme| {
        decl_style::layout_style(
            theme,
            LayoutRefinement::default()
                .w_full()
                .flex_1()
                .min_h_0()
                .relative(),
        )
    });
    let transcript_stack = cx.stack_props(
        fret_ui::element::StackProps {
            layout: transcript_stack_layout,
        },
        move |_cx| vec![transcript, download_button, scroll_button],
    );

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

    let prompt_row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify(Justify::Center),
        move |_cx| vec![prompt_input],
    );

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().h_full())
            .gap(Space::N4),
        move |_cx| vec![transcript_stack, prompt_row, instrumentation],
    );

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
