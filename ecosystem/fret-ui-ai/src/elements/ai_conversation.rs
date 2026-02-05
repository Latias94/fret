use std::sync::Arc;

use fret_core::{Edges, Px, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, SemanticsProps};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{LayoutRefinement, Space};

use crate::elements::MessageParts;
use crate::model::{AiMessage, MessageId};

#[derive(Clone)]
pub struct AiConversationTranscript {
    messages: Arc<[AiMessage]>,
    layout: LayoutRefinement,
    content_padding: Space,
    content_gap: Space,
    content_revision: u64,
    stick_to_bottom: bool,
    stick_threshold: Px,
    scroll_handle: Option<VirtualListScrollHandle>,
    on_link_activate: Option<fret_markdown::OnLinkActivate>,
    test_id_message_prefix: Option<Arc<str>>,
    debug_root_test_id: Option<Arc<str>>,
    debug_row_test_id_prefix: Option<Arc<str>>,
}

impl std::fmt::Debug for AiConversationTranscript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiConversationTranscript")
            .field("messages_len", &self.messages.len())
            .field("layout", &self.layout)
            .field("content_padding", &self.content_padding)
            .field("content_gap", &self.content_gap)
            .field("content_revision", &self.content_revision)
            .field("stick_to_bottom", &self.stick_to_bottom)
            .field("stick_threshold", &self.stick_threshold)
            .field("has_scroll_handle", &self.scroll_handle.is_some())
            .field("has_on_link_activate", &self.on_link_activate.is_some())
            .field(
                "test_id_message_prefix",
                &self.test_id_message_prefix.as_deref(),
            )
            .field("debug_root_test_id", &self.debug_root_test_id.as_deref())
            .field(
                "debug_row_test_id_prefix",
                &self.debug_row_test_id_prefix.as_deref(),
            )
            .finish()
    }
}

impl AiConversationTranscript {
    pub fn new(messages: impl IntoIterator<Item = AiMessage>) -> Self {
        Self::from_arc(messages.into_iter().collect::<Vec<_>>().into())
    }

    pub fn from_arc(messages: Arc<[AiMessage]>) -> Self {
        Self {
            messages,
            layout: LayoutRefinement::default(),
            content_padding: Space::N4,
            content_gap: Space::N8,
            content_revision: 0,
            stick_to_bottom: true,
            stick_threshold: Px(8.0),
            scroll_handle: None,
            on_link_activate: None,
            test_id_message_prefix: None,
            debug_root_test_id: None,
            debug_row_test_id_prefix: None,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    /// Revision marker used to decide when “new content arrived” for stick-to-bottom.
    pub fn content_revision(mut self, revision: u64) -> Self {
        self.content_revision = revision;
        self
    }

    pub fn content_padding(mut self, padding: Space) -> Self {
        self.content_padding = padding;
        self
    }

    pub fn content_gap(mut self, gap: Space) -> Self {
        self.content_gap = gap;
        self
    }

    pub fn stick_to_bottom(mut self, stick: bool) -> Self {
        self.stick_to_bottom = stick;
        self
    }

    pub fn stick_threshold(mut self, threshold: Px) -> Self {
        self.stick_threshold = threshold;
        self
    }

    pub fn scroll_handle(mut self, handle: VirtualListScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }

    pub fn on_link_activate(mut self, on_link_activate: fret_markdown::OnLinkActivate) -> Self {
        self.on_link_activate = Some(on_link_activate);
        self
    }

    /// Optional `test_id` prefix used to stamp stable selectors on message parts.
    ///
    /// Selectors are derived as `${prefix}${message_id}-...` and are intended for automation and
    /// regression gates.
    pub fn test_id_message_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_message_prefix = Some(prefix.into());
        self
    }

    pub fn debug_root_test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.debug_root_test_id = Some(test_id.into());
        self
    }

    pub fn debug_row_test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.debug_row_test_id_prefix = Some(prefix.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        #[derive(Debug, Default, Clone)]
        struct ConversationState {
            handle: VirtualListScrollHandle,
            last_revision: u64,
            pending_scroll_frames: u8,
            was_at_bottom: bool,
            initialized: bool,
        }

        let theme = Theme::global(&*cx.app).clone();

        let layout = self
            .layout
            .merge(LayoutRefinement::default().w_full().h_full().relative());

        let stick_to_bottom = self.stick_to_bottom;
        let stick_threshold = self.stick_threshold;
        let revision = if self.content_revision == 0 {
            self.messages.len().min(u64::MAX as usize) as u64
        } else {
            self.content_revision
        };

        let provided_handle = self.scroll_handle;
        let handle = cx.with_state(ConversationState::default, |st| {
            if let Some(handle) = provided_handle.clone() {
                st.handle = handle;
            }
            st.handle.clone()
        });

        let max = handle.max_offset();
        let offset = handle.offset();
        let is_at_bottom = (max.y.0 - offset.y.0) <= stick_threshold.0;

        let _effectively_at_bottom = cx.with_state(ConversationState::default, |st| {
            let eligible = st.was_at_bottom || st.pending_scroll_frames > 0 || is_at_bottom;

            if !st.initialized {
                st.initialized = true;
                st.last_revision = revision;
                st.was_at_bottom = is_at_bottom;
                if stick_to_bottom {
                    st.pending_scroll_frames = 2;
                }
            } else if stick_to_bottom && revision != st.last_revision && eligible {
                st.pending_scroll_frames = 2;
            }

            if stick_to_bottom && st.pending_scroll_frames > 0 {
                handle.scroll_to_bottom();
                st.pending_scroll_frames = st.pending_scroll_frames.saturating_sub(1);
            }

            st.last_revision = revision;
            st.was_at_bottom = eligible;
            is_at_bottom || st.pending_scroll_frames > 0
        });

        let content_padding = self.content_padding;
        let content_gap = self.content_gap;
        let messages = self.messages;
        let debug_row_test_id_prefix = self.debug_row_test_id_prefix;
        let on_link_activate = self.on_link_activate;
        let test_id_message_prefix = self.test_id_message_prefix;

        let key_at: Arc<dyn Fn(usize) -> u64> = Arc::new({
            let messages = messages.clone();
            move |index| messages.get(index).map(|m| m.id).unwrap_or(index as u64)
        });

        let row: Arc<dyn for<'a> Fn(&mut ElementContext<'a, H>, usize) -> AnyElement> = Arc::new({
            let messages = messages.clone();
            let prefix = debug_row_test_id_prefix.clone();
            let on_link_activate = on_link_activate.clone();
            move |cx, index| {
                let Some(msg) = messages.get(index) else {
                    return cx.text("");
                };

                let mut bubble = MessageParts::new(msg.role, msg.parts.clone());
                if let Some(handler) = on_link_activate.clone() {
                    bubble = bubble.on_link_activate(handler);
                }
                if let Some(prefix) = test_id_message_prefix.clone() {
                    bubble =
                        bubble.test_id_prefix(Arc::<str>::from(format!("{prefix}{}-", msg.id)));
                }
                let bubble = bubble.into_element(cx);

                let Some(prefix) = prefix.clone() else {
                    return bubble;
                };

                let test_id: Arc<str> = Arc::from(format!("{prefix}{index}"));
                cx.semantics(
                    SemanticsProps {
                        role: SemanticsRole::Group,
                        test_id: Some(test_id),
                        ..Default::default()
                    },
                    move |_cx| vec![bubble],
                )
            }
        });

        let mut options = fret_ui::element::VirtualListOptions::new(Px(96.0), 8);
        options.items_revision = revision;
        options.gap = decl_style::space(&theme, content_gap);

        let list_layout = LayoutStyle {
            size: fret_ui::element::SizeStyle {
                width: fret_ui::element::Length::Fill,
                height: fret_ui::element::Length::Fill,
                ..Default::default()
            },
            overflow: fret_ui::element::Overflow::Clip,
            ..Default::default()
        };

        let list = cx.virtual_list_keyed_retained_with_layout(
            list_layout,
            messages.len(),
            options,
            &handle,
            key_at,
            row,
        );

        let padding_px = decl_style::space(&theme, content_padding);
        let list = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(&theme, layout),
                padding: Edges::all(padding_px),
                ..Default::default()
            },
            move |_cx| vec![list],
        );

        let Some(test_id) = self.debug_root_test_id else {
            return list;
        };

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::List,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![list],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageIdStrategy {
    MessageId,
    Index,
}

pub fn message_key_for(messages: &[AiMessage], index: usize, strategy: MessageIdStrategy) -> u64 {
    match strategy {
        MessageIdStrategy::MessageId => messages.get(index).map(|m| m.id).unwrap_or(index as u64),
        MessageIdStrategy::Index => index as u64,
    }
}

pub fn first_message_id(messages: &[AiMessage]) -> Option<MessageId> {
    messages.first().map(|m| m.id)
}
