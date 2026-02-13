use std::sync::Arc;

use fret_core::{Edges, Px, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, SemanticsDecoration};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{LayoutRefinement, Space};

use crate::elements::MessageParts;
use crate::model::{AiMessage, MessageId};

#[cfg(debug_assertions)]
fn debug_assert_unique_message_ids(messages: &[AiMessage]) {
    use std::collections::HashMap;

    let mut seen: HashMap<MessageId, usize> = HashMap::with_capacity(messages.len());
    for (index, message) in messages.iter().enumerate() {
        if let Some(prev) = seen.insert(message.id, index) {
            panic!(
                "duplicate AiMessage.id detected (id={:#x}, a={}, b={}); message IDs must be stable and unique within a transcript",
                message.id, prev, index
            );
        }
    }
}

/// A virtualized transcript surface that renders an `Arc<[AiMessage]>` via `MessageParts`.
///
/// Key properties:
///
/// - **Keyed identity**: messages are keyed by `AiMessage.id` (`MessageId = u64`). IDs must be
///   stable and unique within the transcript.
/// - **Streaming-friendly scrolling**: pass a monotonic `content_revision` that changes whenever
///   “new content arrived” (including streaming append to the last assistant message) so the
///   stick-to-bottom behavior can remain correct.
/// - **Automation gates**: set `test_id_message_prefix` so per-message part selectors remain stable
///   for `fretboard diag` scripts.
#[derive(Clone)]
pub struct AiConversationTranscript {
    messages: Arc<[AiMessage]>,
    layout: LayoutRefinement,
    content_padding: Space,
    content_gap: Space,
    tail_padding: Px,
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
            .field("tail_padding", &self.tail_padding)
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
            tail_padding: Px(96.0),
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

    /// Extra blank space appended after the last message row.
    ///
    /// This is a pragmatic safety margin for virtualization tail underestimation: it ensures the
    /// last message's actions/blocks can be scrolled fully into view even when their measured
    /// height is larger than the initial estimate.
    pub fn tail_padding(mut self, padding: Px) -> Self {
        self.tail_padding = Px(padding.0.max(0.0));
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
            #[cfg(debug_assertions)]
            checked_ids_once: bool,
            #[cfg(debug_assertions)]
            checked_ids_revision: u64,
            #[cfg(debug_assertions)]
            checked_ids_len: usize,
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

        #[cfg(debug_assertions)]
        {
            let needs_check = cx.with_state(ConversationState::default, |st| {
                !st.checked_ids_once
                    || st.checked_ids_revision != revision
                    || st.checked_ids_len != self.messages.len()
            });
            if needs_check {
                debug_assert_unique_message_ids(&self.messages);
                let len = self.messages.len();
                cx.with_state(ConversationState::default, move |st| {
                    st.checked_ids_once = true;
                    st.checked_ids_revision = revision;
                    st.checked_ids_len = len;
                });
            }
        }

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
        let messages_len = messages.len();
        let tail_padding = self.tail_padding;
        let has_tail_padding = tail_padding.0 > 0.01 && messages_len > 0;
        let list_len = if has_tail_padding {
            messages_len.saturating_add(1)
        } else {
            messages_len
        };
        let debug_row_test_id_prefix = self.debug_row_test_id_prefix;
        let on_link_activate = self.on_link_activate;
        let test_id_message_prefix = self.test_id_message_prefix;

        let mut key_at = {
            let messages = messages.clone();
            move |index: usize| {
                messages
                    .get(index)
                    .map(|m| m.id)
                    .unwrap_or(u64::MAX.saturating_sub(index as u64))
            }
        };

        let mut row = {
            let messages = messages.clone();
            let prefix = debug_row_test_id_prefix.clone();
            let on_link_activate = on_link_activate.clone();
            move |cx: &mut ElementContext<'_, H>, index: usize| {
                if index >= messages.len() {
                    // Tail padding is modeled as an extra virtual list row to provide additional
                    // scroll reach beyond the last message. However, this row can temporarily
                    // overlap richer last-message content while measurement stabilizes (variable
                    // row height + estimate).
                    //
                    // Make the tail row non-interactive so it never intercepts pointer events
                    // (it is a blank spacer by construction).
                    return cx.interactivity_gate(true, false, |cx| {
                        vec![cx.container(
                            ContainerProps {
                                layout: LayoutStyle {
                                    size: fret_ui::element::SizeStyle {
                                        width: fret_ui::element::Length::Fill,
                                        height: fret_ui::element::Length::Px(tail_padding),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            |_cx| Vec::new(),
                        )]
                    });
                }

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
                cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            // Note: VirtualList already applies a per-row clip in paint. Clipping
                            // at this wrapper can desync hit-testing vs. visual geometry when row
                            // bounds lag behind dynamically-sized content.
                            overflow: fret_ui::element::Overflow::Visible,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |_cx| vec![bubble],
                )
                .attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id(test_id),
                )
            }
        };

        let mut options = fret_ui::element::VirtualListOptions::new(Px(96.0), 8);
        options.items_revision = revision;
        options.gap = if content_gap == Space::N8 {
            theme
                .metric_by_key("fret.ai.conversation.gap")
                .unwrap_or_else(|| decl_style::space(&theme, content_gap))
        } else {
            decl_style::space(&theme, content_gap)
        };

        let list_layout = LayoutStyle {
            size: fret_ui::element::SizeStyle {
                width: fret_ui::element::Length::Fill,
                height: fret_ui::element::Length::Fill,
                ..Default::default()
            },
            overflow: fret_ui::element::Overflow::Clip,
            ..Default::default()
        };

        let list =
            cx.virtual_list_keyed_with_layout(list_layout, list_len, options, &handle, key_at, row);

        let padding_px = if content_padding == Space::N4 {
            theme
                .metric_by_key("fret.ai.conversation.padding")
                .unwrap_or_else(|| decl_style::space(&theme, content_padding))
        } else {
            decl_style::space(&theme, content_padding)
        };
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
        list.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::List)
                .test_id(test_id),
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
