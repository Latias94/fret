use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};

use crate::elements::attachments::AttachmentData;
use crate::elements::conversation::CONVERSATION_MANAGED_CONTENT_SLOT_KEY;
use crate::elements::{
    AiConversationTranscript, Conversation, ConversationContent, ConversationDownload,
    ConversationEmptyState, ConversationScrollButton, PromptInput,
};
use crate::model::AiMessage;

#[derive(Clone)]
struct AiChatState {
    handle: VirtualListScrollHandle,
}

impl Default for AiChatState {
    fn default() -> Self {
        Self {
            handle: VirtualListScrollHandle::new(),
        }
    }
}

/// A ready-to-compose chat surface built from `fret-ui-ai` parts.
///
/// This is a convenience wrapper that composes:
///
/// - `AiConversationTranscript` (parts-based transcript)
/// - `ConversationScrollButton` (overlay “scroll to bottom” affordance)
/// - `PromptInput` (prompt composer)
/// - optional `ConversationEmptyState` and `ConversationDownload` (app-owned effects)
///
/// Effects (clipboard/file IO/network) remain app-owned; this component only emits intents via
/// action hooks (`OnActivate`).
pub struct AiChat {
    messages: Model<Arc<[AiMessage]>>,
    prompt: Model<String>,
    loading_model: Option<Model<bool>>,
    content_revision_model: Option<Model<u64>>,
    disabled: bool,
    stick_to_bottom: bool,
    on_send: Option<OnActivate>,
    on_stop: Option<OnActivate>,
    on_download: Option<OnActivate>,
    on_add_attachments: Option<OnActivate>,
    show_download: bool,
    empty_state: Option<ConversationEmptyState>,
    message_test_id_prefix: Option<Arc<str>>,
    transcript_root_test_id: Option<Arc<str>>,
    transcript_row_test_id_prefix: Option<Arc<str>>,
    scroll_button_test_id: Option<Arc<str>>,
    download_test_id: Option<Arc<str>>,
    prompt_root_test_id: Option<Arc<str>>,
    prompt_textarea_test_id: Option<Arc<str>>,
    prompt_send_test_id: Option<Arc<str>>,
    prompt_stop_test_id: Option<Arc<str>>,
    prompt_attachments_model: Option<Model<Vec<AttachmentData>>>,
    prompt_attachments_test_id: Option<Arc<str>>,
    prompt_add_attachments_test_id: Option<Arc<str>>,
    root_test_id: Option<Arc<str>>,
    scroll_handle: Option<VirtualListScrollHandle>,
    root_layout: LayoutRefinement,
    transcript_container_layout: LayoutRefinement,
}

impl std::fmt::Debug for AiChat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiChat")
            .field("messages", &"<model>")
            .field("prompt", &"<model>")
            .field("has_loading_model", &self.loading_model.is_some())
            .field(
                "has_content_revision_model",
                &self.content_revision_model.is_some(),
            )
            .field("disabled", &self.disabled)
            .field("stick_to_bottom", &self.stick_to_bottom)
            .field("has_on_send", &self.on_send.is_some())
            .field("has_on_stop", &self.on_stop.is_some())
            .field("has_on_download", &self.on_download.is_some())
            .field("has_on_add_attachments", &self.on_add_attachments.is_some())
            .field("show_download", &self.show_download)
            .field("has_empty_state", &self.empty_state.is_some())
            .field(
                "message_test_id_prefix",
                &self.message_test_id_prefix.as_deref(),
            )
            .field(
                "transcript_root_test_id",
                &self.transcript_root_test_id.as_deref(),
            )
            .field(
                "transcript_row_test_id_prefix",
                &self.transcript_row_test_id_prefix.as_deref(),
            )
            .field(
                "scroll_button_test_id",
                &self.scroll_button_test_id.as_deref(),
            )
            .field("download_test_id", &self.download_test_id.as_deref())
            .field("prompt_root_test_id", &self.prompt_root_test_id.as_deref())
            .field(
                "prompt_textarea_test_id",
                &self.prompt_textarea_test_id.as_deref(),
            )
            .field("prompt_send_test_id", &self.prompt_send_test_id.as_deref())
            .field("prompt_stop_test_id", &self.prompt_stop_test_id.as_deref())
            .field(
                "has_prompt_attachments_model",
                &self.prompt_attachments_model.is_some(),
            )
            .field(
                "prompt_attachments_test_id",
                &self.prompt_attachments_test_id.as_deref(),
            )
            .field(
                "prompt_add_attachments_test_id",
                &self.prompt_add_attachments_test_id.as_deref(),
            )
            .field("root_test_id", &self.root_test_id.as_deref())
            .field("has_scroll_handle", &self.scroll_handle.is_some())
            .field("root_layout", &self.root_layout)
            .field(
                "transcript_container_layout",
                &self.transcript_container_layout,
            )
            .finish()
    }
}

impl AiChat {
    pub fn new(messages: Model<Arc<[AiMessage]>>, prompt: Model<String>) -> Self {
        Self {
            messages,
            prompt,
            loading_model: None,
            content_revision_model: None,
            disabled: false,
            stick_to_bottom: true,
            on_send: None,
            on_stop: None,
            on_download: None,
            on_add_attachments: None,
            show_download: false,
            empty_state: None,
            message_test_id_prefix: None,
            transcript_root_test_id: None,
            transcript_row_test_id_prefix: None,
            scroll_button_test_id: None,
            download_test_id: None,
            prompt_root_test_id: None,
            prompt_textarea_test_id: None,
            prompt_send_test_id: None,
            prompt_stop_test_id: None,
            prompt_attachments_model: None,
            prompt_attachments_test_id: None,
            prompt_add_attachments_test_id: None,
            root_test_id: None,
            scroll_handle: None,
            root_layout: LayoutRefinement::default(),
            transcript_container_layout: LayoutRefinement::default().w_full().flex_1().min_h_0(),
        }
    }

    pub fn loading_model(mut self, model: Model<bool>) -> Self {
        self.loading_model = Some(model);
        self
    }

    /// Revision marker used by the transcript to decide when “new content arrived” for stick-to-bottom.
    ///
    /// Recommended: update this for streaming assistant output (append chunks) as well as for new
    /// message insertion, so the scroll position remains pinned at the bottom when appropriate.
    pub fn content_revision_model(mut self, model: Model<u64>) -> Self {
        self.content_revision_model = Some(model);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn stick_to_bottom(mut self, stick: bool) -> Self {
        self.stick_to_bottom = stick;
        self
    }

    pub fn on_send(mut self, on_send: OnActivate) -> Self {
        self.on_send = Some(on_send);
        self
    }

    pub fn on_stop(mut self, on_stop: OnActivate) -> Self {
        self.on_stop = Some(on_stop);
        self
    }

    pub fn on_add_attachments(mut self, on_add_attachments: OnActivate) -> Self {
        self.on_add_attachments = Some(on_add_attachments);
        self
    }

    pub fn show_download(mut self, show: bool) -> Self {
        self.show_download = show;
        self
    }

    pub fn on_download(mut self, on_download: OnActivate) -> Self {
        self.on_download = Some(on_download);
        self
    }

    pub fn empty_state(mut self, empty_state: ConversationEmptyState) -> Self {
        self.empty_state = Some(empty_state);
        self
    }

    /// Optional `test_id` prefix for per-message parts selectors (see `AiConversationTranscript`).
    pub fn message_test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.message_test_id_prefix = Some(prefix.into());
        self
    }

    pub fn transcript_root_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.transcript_root_test_id = Some(id.into());
        self
    }

    pub fn transcript_row_test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.transcript_row_test_id_prefix = Some(prefix.into());
        self
    }

    pub fn scroll_button_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.scroll_button_test_id = Some(id.into());
        self
    }

    pub fn download_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.download_test_id = Some(id.into());
        self
    }

    pub fn prompt_root_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.prompt_root_test_id = Some(id.into());
        self
    }

    pub fn prompt_textarea_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.prompt_textarea_test_id = Some(id.into());
        self
    }

    pub fn prompt_send_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.prompt_send_test_id = Some(id.into());
        self
    }

    pub fn prompt_stop_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.prompt_stop_test_id = Some(id.into());
        self
    }

    pub fn prompt_attachments_model(mut self, model: Model<Vec<AttachmentData>>) -> Self {
        self.prompt_attachments_model = Some(model);
        self
    }

    pub fn prompt_attachments_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.prompt_attachments_test_id = Some(id.into());
        self
    }

    pub fn prompt_add_attachments_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.prompt_add_attachments_test_id = Some(id.into());
        self
    }

    pub fn root_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.root_test_id = Some(id.into());
        self
    }

    pub fn scroll_handle(mut self, handle: VirtualListScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.root_layout = self.root_layout.merge(layout);
        self
    }

    pub fn transcript_container_layout(mut self, layout: LayoutRefinement) -> Self {
        self.transcript_container_layout = self.transcript_container_layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let messages_value = cx
            .get_model_cloned(&self.messages, Invalidation::Paint)
            .unwrap_or_default();
        let is_empty = messages_value.is_empty();

        let loading = self
            .loading_model
            .as_ref()
            .and_then(|m| cx.get_model_copied(m, Invalidation::Paint))
            .unwrap_or(false);

        let revision = self
            .content_revision_model
            .as_ref()
            .and_then(|m| cx.get_model_copied(m, Invalidation::Paint))
            .unwrap_or(0);
        let conversation_revision = if revision == 0 {
            messages_value.len().min(u64::MAX as usize) as u64
        } else {
            revision
        };

        let provided_handle = self.scroll_handle;
        let handle = cx.root_state(AiChatState::default, |st| {
            if let Some(handle) = provided_handle.clone() {
                st.handle = handle;
            }
            st.handle.clone()
        });

        let mut transcript_container = Conversation::new([])
            .scroll_handle(handle.base_handle().clone())
            .content_revision(conversation_revision)
            .stick_to_bottom(self.stick_to_bottom)
            .show_scroll_to_bottom_button(false)
            .refine_layout(self.transcript_container_layout);
        if let Some(id) = self.transcript_root_test_id.clone() {
            transcript_container = transcript_container.test_id(id);
        }
        let transcript_container = transcript_container.into_element_with_children(cx, |cx| {
            let mut children = Vec::new();

            if is_empty {
                let empty_state = self.empty_state.unwrap_or_else(|| {
                    ConversationEmptyState::new("Start a conversation")
                        .description("Messages will appear here as you send prompts.")
                });
                children.push(
                    ConversationContent::new([empty_state.into_element(cx)])
                        .content_padding(Space::N0)
                        .content_gap(Space::N0)
                        .into_element(cx),
                );
            } else {
                let mut transcript = AiConversationTranscript::from_arc(messages_value.clone())
                    .scroll_handle(handle.clone())
                    .stick_to_bottom(self.stick_to_bottom)
                    .tail_padding(Px(360.0));

                if revision != 0 {
                    transcript = transcript.content_revision(revision);
                }
                if let Some(prefix) = self.message_test_id_prefix.clone() {
                    transcript = transcript.test_id_message_prefix(prefix);
                }
                if let Some(prefix) = self.transcript_row_test_id_prefix.clone() {
                    transcript = transcript.debug_row_test_id_prefix(prefix);
                }

                children.push(
                    transcript
                        .into_element(cx)
                        .key_context(CONVERSATION_MANAGED_CONTENT_SLOT_KEY),
                );
            }

            if self.show_download {
                let mut download = ConversationDownload::new("Export Markdown")
                    .show_label(true)
                    .disabled(self.disabled);
                if let Some(on_download) = self.on_download.clone() {
                    download = download.on_activate(on_download);
                }
                if let Some(id) = self.download_test_id.clone() {
                    download = download.test_id(id);
                }
                children.push(download.into_element(cx));
            }

            if !is_empty {
                let mut scroll = ConversationScrollButton::default();
                if let Some(id) = self.scroll_button_test_id.clone() {
                    scroll = scroll.test_id(id);
                }
                children.push(scroll.into_element(cx));
            }

            children
        });

        let mut prompt = PromptInput::new(self.prompt)
            .disabled(self.disabled)
            .loading(loading)
            .clear_on_send(true);
        if let Some(on_send) = self.on_send.clone() {
            prompt = prompt.on_send(on_send);
        }
        if let Some(on_stop) = self.on_stop.clone() {
            prompt = prompt.on_stop(on_stop);
        }
        if let Some(on_add) = self.on_add_attachments.clone() {
            prompt = prompt.on_add_attachments(on_add);
        }
        if let Some(id) = self.prompt_root_test_id.clone() {
            prompt = prompt.test_id_root(id);
        }
        if let Some(id) = self.prompt_textarea_test_id.clone() {
            prompt = prompt.test_id_textarea(id);
        }
        if let Some(id) = self.prompt_send_test_id.clone() {
            prompt = prompt.test_id_send(id);
        }
        if let Some(id) = self.prompt_stop_test_id.clone() {
            prompt = prompt.test_id_stop(id);
        }
        if let Some(model) = self.prompt_attachments_model.clone() {
            prompt = prompt.attachments(model);
        }
        if let Some(id) = self.prompt_attachments_test_id.clone() {
            prompt = prompt.test_id_attachments(id);
        }
        if let Some(id) = self.prompt_add_attachments_test_id.clone() {
            prompt = prompt.test_id_add_attachments(id);
        }

        let prompt = prompt.into_element(cx);

        let footer = ui::v_stack(|_cx| vec![prompt])
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2)
            .into_element(cx);

        let root = ui::v_stack(|_cx| vec![transcript_container, footer])
            .layout(
                LayoutRefinement::default()
                    .w_full()
                    .h_full()
                    .merge(self.root_layout),
            )
            .gap(Space::N2)
            .into_element(cx);

        let Some(test_id) = self.root_test_id else {
            return root;
        };

        cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![root],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{ElementKind, TextProps};

    use crate::model::{MessagePart, MessageRole};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(480.0), Px(320.0)),
        )
    }

    fn has_test_id(element: &AnyElement, expected: &str) -> bool {
        if element
            .semantics_decoration
            .as_ref()
            .and_then(|d| d.test_id.as_deref())
            == Some(expected)
        {
            return true;
        }

        if matches!(
            &element.kind,
            ElementKind::Semantics(props) if props.test_id.as_deref() == Some(expected)
        ) {
            return true;
        }

        element
            .children
            .iter()
            .any(|child| has_test_id(child, expected))
    }

    fn has_text(element: &AnyElement, expected: &str) -> bool {
        match &element.kind {
            ElementKind::Text(TextProps { text, .. }) if text.as_ref() == expected => true,
            _ => element
                .children
                .iter()
                .any(|child| has_text(child, expected)),
        }
    }

    fn count_wheel_regions(element: &AnyElement) -> usize {
        let self_count = usize::from(matches!(&element.kind, ElementKind::WheelRegion(_)));
        self_count
            + element
                .children
                .iter()
                .map(count_wheel_regions)
                .sum::<usize>()
    }

    #[test]
    fn ai_chat_reuses_conversation_without_nested_scroll_area() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let messages = app.models_mut().insert(
            vec![AiMessage::new(
                1,
                MessageRole::Assistant,
                [MessagePart::Text(Arc::<str>::from("Hello from assistant"))],
            )]
            .into(),
        );
        let prompt = app.models_mut().insert(String::new());
        let handle = VirtualListScrollHandle::new();
        handle
            .base_handle()
            .set_viewport_size(Size::new(Px(320.0), Px(120.0)));
        handle
            .base_handle()
            .set_content_size(Size::new(Px(320.0), Px(720.0)));

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                AiChat::new(messages.clone(), prompt.clone())
                    .scroll_handle(handle.clone())
                    .stick_to_bottom(false)
                    .show_download(true)
                    .transcript_root_test_id("chat-transcript")
                    .scroll_button_test_id("chat-scroll")
                    .download_test_id("chat-download")
                    .into_element(cx)
            });

        assert!(has_test_id(&element, "chat-transcript"));
        assert!(has_test_id(&element, "chat-scroll.chrome"));
        assert!(has_text(&element, "Export Markdown"));
        assert_eq!(count_wheel_regions(&element), 1);
    }
}
