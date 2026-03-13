use std::sync::Arc;

use fret_core::{Corners, Edges, Point, Px, SemanticsRole};
use fret_icons::IconId;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, InteractivityGateProps, LayoutStyle, SemanticsDecoration,
    SemanticsProps, StackProps,
};
use fret_ui::scroll::{ScrollHandle, ScrollStrategy, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{Items, Justify, LayoutRefinement, Space};

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant, ScrollArea};

use crate::model::MessageId;
use crate::{Message, MessageContent, MessageRole};

pub(crate) const CONVERSATION_CONTENT_SLOT_KEY: &str = "__fret_ui_ai.conversation.content";
pub(crate) const CONVERSATION_MANAGED_CONTENT_SLOT_KEY: &str =
    "__fret_ui_ai.conversation.managed_content";
pub(crate) const CONVERSATION_DOWNLOAD_SLOT_KEY: &str = "__fret_ui_ai.conversation.download";
pub(crate) const CONVERSATION_SCROLL_BUTTON_SLOT_KEY: &str =
    "__fret_ui_ai.conversation.scroll_button";

#[derive(Debug, Clone)]
pub(crate) struct ConversationContext {
    scroll_handle: ScrollHandle,
}

impl ConversationContext {
    pub(crate) fn new(scroll_handle: ScrollHandle) -> Self {
        Self { scroll_handle }
    }

    pub(crate) fn scroll_handle(&self) -> ScrollHandle {
        self.scroll_handle.clone()
    }
}

#[derive(Debug, Default, Clone)]
struct ConversationLocalState {
    context: Option<ConversationContext>,
}

#[derive(Debug, Default, Clone)]
struct ConversationRuntimeState {
    handle: ScrollHandle,
    last_revision: u64,
    pending_scroll_frames: u8,
    was_at_bottom: bool,
    initialized: bool,
}

pub(crate) fn use_conversation_context<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<ConversationContext> {
    cx.inherited_state::<ConversationLocalState>()
        .and_then(|state| state.context.clone())
}

pub(crate) fn conversation_slot_placeholder<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    slot_key: &'static str,
    semantics: Option<SemanticsDecoration>,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut slot = cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: LayoutStyle::default(),
            present: false,
            interactive: false,
        },
        move |_cx| children,
    );
    slot.key_context = Some(Arc::<str>::from(slot_key));
    if let Some(decoration) = semantics {
        slot = slot.attach_semantics(decoration);
    }
    slot
}

fn content_with_default_layout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    content_padding: Space,
    content_gap: Space,
    children: Vec<AnyElement>,
) -> AnyElement {
    let padding_px = if content_padding == Space::N4 {
        theme
            .metric_by_key("fret.ai.conversation.padding")
            .unwrap_or_else(|| decl_style::space(theme, content_padding))
    } else {
        decl_style::space(theme, content_padding)
    };

    cx.container(
        ContainerProps {
            layout: LayoutStyle::default(),
            padding: Edges::all(padding_px).into(),
            ..Default::default()
        },
        move |cx| {
            vec![
                ui::v_stack(move |_cx| children)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(content_gap)
                    .items(Items::Stretch)
                    .into_element(cx),
            ]
        },
    )
}

#[derive(Debug, Default)]
struct ResolvedConversationChildren {
    content: Option<AnyElement>,
    managed_content: Option<AnyElement>,
    overlay_downloads: Vec<AnyElement>,
    overlay_scroll_buttons: Vec<AnyElement>,
    legacy_content_children: Vec<AnyElement>,
}

fn resolve_conversation_root_children(children: Vec<AnyElement>) -> ResolvedConversationChildren {
    let mut resolved = ResolvedConversationChildren::default();

    for mut child in children {
        match child.key_context.as_deref() {
            Some(CONVERSATION_MANAGED_CONTENT_SLOT_KEY) if resolved.managed_content.is_none() => {
                child.key_context = None;
                resolved.managed_content = Some(child);
            }
            Some(CONVERSATION_CONTENT_SLOT_KEY) if resolved.content.is_none() => {
                child.key_context = None;
                resolved.content = Some(child);
            }
            Some(CONVERSATION_DOWNLOAD_SLOT_KEY) => {
                child.key_context = None;
                resolved.overlay_downloads.push(child);
            }
            Some(CONVERSATION_SCROLL_BUTTON_SLOT_KEY) => {
                child.key_context = None;
                resolved.overlay_scroll_buttons.push(child);
            }
            _ => resolved.legacy_content_children.push(child),
        }
    }

    resolved
}

fn conversation_scroll_handle<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    provided_handle: Option<ScrollHandle>,
) -> ScrollHandle {
    cx.root_state(ConversationRuntimeState::default, |state| {
        if let Some(handle) = provided_handle {
            state.handle = handle;
        }
        state.handle.clone()
    })
}

fn render_conversation_root<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    layout: LayoutRefinement,
    content_padding: Space,
    content_gap: Space,
    revision: u64,
    stick_to_bottom: bool,
    stick_threshold: Px,
    show_scroll_to_bottom_button: bool,
    test_id: Option<Arc<str>>,
    handle: ScrollHandle,
    children: Vec<AnyElement>,
) -> AnyElement {
    let scroll_layout = LayoutRefinement::default().w_full().h_full();

    let max = handle.max_offset();
    let offset = handle.offset();
    let is_at_bottom = (max.y.0 - offset.y.0) <= stick_threshold.0;

    let effectively_at_bottom = cx.root_state(ConversationRuntimeState::default, |state| {
        let eligible = state.was_at_bottom || state.pending_scroll_frames > 0 || is_at_bottom;

        if !state.initialized {
            state.initialized = true;
            state.last_revision = revision;
            state.was_at_bottom = is_at_bottom;
            if stick_to_bottom {
                state.pending_scroll_frames = 2;
            }
        } else if stick_to_bottom && revision != state.last_revision && eligible {
            state.pending_scroll_frames = 2;
        }

        if stick_to_bottom && state.pending_scroll_frames > 0 {
            let next = Point::new(handle.offset().x, handle.max_offset().y);
            handle.scroll_to_offset(next);
            state.pending_scroll_frames = state.pending_scroll_frames.saturating_sub(1);
        }

        state.last_revision = revision;
        state.was_at_bottom = eligible;
        is_at_bottom || state.pending_scroll_frames > 0
    });

    let resolved = resolve_conversation_root_children(children);
    let scroll_body = if let Some(mut managed_content) = resolved.managed_content {
        if let Some(test_id) = test_id.clone() {
            managed_content = managed_content.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            );
        }
        managed_content
    } else {
        let content = resolved.content.unwrap_or_else(|| {
            content_with_default_layout(
                cx,
                theme,
                content_padding,
                content_gap,
                resolved.legacy_content_children,
            )
        });

        let mut scroll_area = ScrollArea::new(vec![content])
            .scroll_handle(handle.clone())
            .refine_layout(scroll_layout)
            .into_element(cx);
        if let Some(test_id) = test_id.clone() {
            scroll_area = scroll_area.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            );
        }
        scroll_area
    };

    let root_layout = decl_style::layout_style(theme, layout);
    cx.stack_props(
        StackProps {
            layout: root_layout,
        },
        move |cx| {
            let mut out = vec![scroll_body];

            if !resolved.overlay_downloads.is_empty() {
                let overlay_layout = decl_style::layout_style(
                    theme,
                    LayoutRefinement::default()
                        .absolute()
                        .left(Space::N0)
                        .right(Space::N0)
                        .top(Space::N4),
                );
                let pad = decl_style::space(theme, Space::N4);
                let children = resolved.overlay_downloads;
                out.push(cx.container(
                    ContainerProps {
                        layout: overlay_layout,
                        padding: Edges::symmetric(pad, Px(0.0)).into(),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            ui::h_row(move |_cx| children)
                                .layout(LayoutRefinement::default().w_full())
                                .justify(Justify::End)
                                .items(Items::Center)
                                .gap(Space::N2)
                                .into_element(cx),
                        ]
                    },
                ));
            }

            if !resolved.overlay_scroll_buttons.is_empty() {
                let mut overlay_layout = decl_style::layout_style(
                    theme,
                    LayoutRefinement::default()
                        .absolute()
                        .left(Space::N0)
                        .right(Space::N0)
                        .bottom(Space::N4),
                );
                if let Some(bottom) =
                    theme.metric_by_key("fret.ai.conversation.scroll_button.offset_bottom")
                {
                    overlay_layout.inset.bottom = Some(bottom).into();
                }
                let children = resolved.overlay_scroll_buttons;
                out.push(cx.container(
                    ContainerProps {
                        layout: overlay_layout,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            ui::h_row(move |_cx| children)
                                .layout(LayoutRefinement::default().w_full())
                                .justify(Justify::Center)
                                .items(Items::Center)
                                .gap(Space::N2)
                                .into_element(cx),
                        ]
                    },
                ));
            } else if show_scroll_to_bottom_button && !effectively_at_bottom {
                let handle_for_button = handle.clone();
                let on_activate: OnActivate =
                    std::sync::Arc::new(move |host, action_cx, _reason| {
                        let max = handle_for_button.max_offset();
                        let cur = handle_for_button.offset();
                        handle_for_button.scroll_to_offset(Point::new(cur.x, max.y));
                        host.request_redraw(action_cx.window);
                    });

                let button = Button::new("")
                    .a11y_label("Scroll to bottom")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Icon)
                    .leading_icon(IconId::new_static("lucide.arrow-down"))
                    .corner_radii_override(Corners::all(Px(999.0)))
                    .on_activate(on_activate)
                    .into_element(cx);

                let mut overlay_layout = decl_style::layout_style(
                    theme,
                    LayoutRefinement::default()
                        .absolute()
                        .left(Space::N0)
                        .right(Space::N0)
                        .bottom(Space::N4),
                );
                if let Some(bottom) =
                    theme.metric_by_key("fret.ai.conversation.scroll_button.offset_bottom")
                {
                    overlay_layout.inset.bottom = Some(bottom).into();
                }

                out.push(cx.container(
                    ContainerProps {
                        layout: overlay_layout,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            ui::h_row(move |_cx| vec![button])
                                .layout(LayoutRefinement::default().w_full())
                                .justify(Justify::Center)
                                .items(Items::Center)
                                .into_element(cx),
                        ]
                    },
                ));
            }

            out
        },
    )
}

/// A minimal message record used by `ConversationTranscript`.
///
/// Prefer `AiMessage` + `AiConversationTranscript` when you need rich parts (markdown/tool calls).
#[derive(Debug, Clone)]
pub struct ConversationMessage {
    pub id: MessageId,
    pub role: MessageRole,
    pub text: Arc<str>,
}

impl ConversationMessage {
    pub fn new(id: MessageId, role: MessageRole, text: impl Into<Arc<str>>) -> Self {
        Self {
            id,
            role,
            text: text.into(),
        }
    }
}

/// A virtualized transcript surface for `ConversationMessage`.
///
/// This is a lightweight, text-only transcript used by UI Gallery harnesses. For richer AI chat
/// surfaces, prefer `AiConversationTranscript`.
#[derive(Debug, Clone)]
pub struct ConversationTranscript {
    messages: Arc<[ConversationMessage]>,
    layout: LayoutRefinement,
    content_padding: Space,
    content_gap: Space,
    content_revision: u64,
    stick_to_bottom: bool,
    stick_threshold: Px,
    show_scroll_to_bottom_button: bool,
    scroll_handle: Option<VirtualListScrollHandle>,
    debug_root_test_id: Option<Arc<str>>,
    debug_row_test_id_prefix: Option<Arc<str>>,
}

/// Compound content slot aligned with AI Elements `ConversationContent`.
#[derive(Debug)]
pub struct ConversationContent {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    content_padding: Space,
    content_gap: Space,
    test_id: Option<Arc<str>>,
}

impl ConversationContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            content_padding: Space::N4,
            content_gap: Space::N8,
            test_id: None,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let content = content_with_default_layout(
            cx,
            &theme,
            self.content_padding,
            self.content_gap,
            self.children,
        );

        let content = cx.container(
            ContainerProps {
                layout: decl_style::layout_style(&theme, self.layout),
                ..Default::default()
            },
            move |_cx| vec![content],
        );

        let content = if let Some(test_id) = self.test_id {
            content.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            )
        } else {
            content
        };

        content.key_context(CONVERSATION_CONTENT_SLOT_KEY)
    }
}

impl ConversationTranscript {
    pub fn new(messages: impl IntoIterator<Item = ConversationMessage>) -> Self {
        Self::from_arc(messages.into_iter().collect::<Vec<_>>().into())
    }

    pub fn from_arc(messages: Arc<[ConversationMessage]>) -> Self {
        Self {
            messages,
            layout: LayoutRefinement::default(),
            content_padding: Space::N4,
            content_gap: Space::N8,
            content_revision: 0,
            stick_to_bottom: true,
            stick_threshold: Px(8.0),
            show_scroll_to_bottom_button: true,
            scroll_handle: None,
            debug_root_test_id: None,
            debug_row_test_id_prefix: None,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    /// Revision marker used to decide when “new content arrived” for stick-to-bottom.
    ///
    /// Recommended: pass `messages.len() as u64` or a monotonic “last message id” counter.
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

    /// How close (in px) the scroll position must be to the max offset to be considered “at bottom”.
    pub fn stick_threshold(mut self, threshold: Px) -> Self {
        self.stick_threshold = threshold;
        self
    }

    pub fn show_scroll_to_bottom_button(mut self, show: bool) -> Self {
        self.show_scroll_to_bottom_button = show;
        self
    }

    pub fn scroll_handle(mut self, handle: VirtualListScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
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

        let show_scroll_to_bottom_button = self.show_scroll_to_bottom_button;
        let provided_handle = self.scroll_handle;
        let handle = cx.root_state(ConversationState::default, |st| {
            if let Some(handle) = provided_handle.clone() {
                st.handle = handle;
            }
            st.handle.clone()
        });

        let max = handle.max_offset();
        let offset = handle.offset();
        let is_at_bottom = (max.y.0 - offset.y.0) <= stick_threshold.0;

        let effectively_at_bottom = cx.root_state(ConversationState::default, |st| {
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

        let key_at: Arc<dyn Fn(usize) -> u64> = Arc::new({
            let messages = messages.clone();
            move |index| messages.get(index).map(|m| m.id).unwrap_or(index as u64)
        });

        let row: Arc<dyn for<'a> Fn(&mut ElementContext<'a, H>, usize) -> AnyElement> = Arc::new({
            let messages = messages.clone();
            let prefix = debug_row_test_id_prefix.clone();
            move |cx, index| {
                let Some(msg) = messages.get(index) else {
                    return cx.text("");
                };

                let content =
                    MessageContent::new(msg.role, [cx.text(msg.text.clone())]).into_element(cx);
                let bubble = Message::new(msg.role, [content]).into_element(cx);
                let Some(prefix) = prefix.clone() else {
                    return bubble;
                };

                let test_id: Arc<str> = Arc::from(format!("{prefix}{}", msg.id));
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

        let mut options = fret_ui::element::VirtualListOptions::new(Px(64.0), 8);
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

        let list = cx.virtual_list_keyed_retained_with_layout(
            list_layout,
            messages.len(),
            options,
            &handle,
            key_at,
            row,
        );

        let padding_px = if content_padding == Space::N4 {
            theme
                .metric_by_key("fret.ai.conversation.padding")
                .unwrap_or_else(|| decl_style::space(&theme, content_padding))
        } else {
            decl_style::space(&theme, content_padding)
        };
        let list = cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Fill,
                        height: fret_ui::element::Length::Fill,
                        ..Default::default()
                    },
                    overflow: fret_ui::element::Overflow::Clip,
                    ..Default::default()
                },
                padding: Edges::all(padding_px).into(),
                ..Default::default()
            },
            move |_cx| vec![list],
        );

        let root_layout = decl_style::layout_style(&theme, layout);
        let debug_root_test_id = self.debug_root_test_id;
        let root = if let Some(test_id) = debug_root_test_id {
            cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(test_id),
                    ..Default::default()
                },
                move |_cx| vec![list],
            )
        } else {
            list
        };

        cx.stack_props(
            StackProps {
                layout: root_layout,
            },
            move |cx| {
                let mut out = vec![root];

                if show_scroll_to_bottom_button && !effectively_at_bottom {
                    let handle_for_button = handle.clone();
                    let on_activate: OnActivate = std::sync::Arc::new(move |host, cx, _reason| {
                        handle_for_button.scroll_to_item(usize::MAX, ScrollStrategy::End);
                        host.request_redraw(cx.window);
                    });

                    let button = Button::new("")
                        .a11y_label("Scroll to bottom")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .leading_icon(IconId::new_static("lucide.arrow-down"))
                        .corner_radii_override(Corners::all(Px(999.0)))
                        .on_activate(on_activate)
                        .into_element(cx);

                    let overlay_layout = decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .absolute()
                            .left(Space::N0)
                            .right(Space::N0)
                            .bottom(Space::N4),
                    );
                    let mut overlay_layout = overlay_layout;
                    if let Some(bottom) =
                        theme.metric_by_key("fret.ai.conversation.scroll_button.offset_bottom")
                    {
                        overlay_layout.inset.bottom = Some(bottom).into();
                    }

                    out.push(cx.container(
                        ContainerProps {
                            layout: overlay_layout,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                ui::h_row(move |_cx| vec![button])
                                    .layout(LayoutRefinement::default().w_full())
                                    .justify(Justify::Center)
                                    .items(Items::Center)
                                    .into_element(cx),
                            ]
                        },
                    ));
                }

                out
            },
        )
    }
}

/// A scrollable “conversation” container that manages stick-to-bottom behavior for arbitrary children.
///
/// This is a generic composition helper (often used to wrap a transcript + overlays).
#[derive(Debug)]
pub struct Conversation {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    content_padding: Space,
    content_gap: Space,
    content_revision: u64,
    stick_to_bottom: bool,
    stick_threshold: Px,
    show_scroll_to_bottom_button: bool,
    scroll_handle: Option<ScrollHandle>,
    test_id: Option<Arc<str>>,
}

impl Conversation {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
            content_padding: Space::N4,
            content_gap: Space::N8,
            content_revision: 0,
            stick_to_bottom: true,
            stick_threshold: Px(8.0),
            show_scroll_to_bottom_button: true,
            scroll_handle: None,
            test_id: None,
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    /// Revision marker used to decide when “new content arrived” for stick-to-bottom.
    ///
    /// Recommended: pass `messages.len() as u64` or a monotonic “last message id” counter.
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

    /// How close (in px) the scroll position must be to the max offset to be considered “at bottom”.
    pub fn stick_threshold(mut self, threshold: Px) -> Self {
        self.stick_threshold = threshold;
        self
    }

    pub fn show_scroll_to_bottom_button(mut self, show: bool) -> Self {
        self.show_scroll_to_bottom_button = show;
        self
    }

    pub fn scroll_handle(mut self, handle: ScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let layout = self
            .layout
            .merge(LayoutRefinement::default().w_full().h_full().relative());
        let handle = conversation_scroll_handle(cx, self.scroll_handle.clone());
        let context = ConversationContext::new(handle.clone());
        cx.root_state(ConversationLocalState::default, |state| {
            state.context = Some(context);
        });
        let built_children = children(cx);

        render_conversation_root(
            cx,
            &theme,
            layout,
            self.content_padding,
            self.content_gap,
            self.content_revision,
            self.stick_to_bottom,
            self.stick_threshold,
            self.show_scroll_to_bottom_button,
            self.test_id,
            handle,
            built_children,
        )
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let layout = self
            .layout
            .merge(LayoutRefinement::default().w_full().h_full().relative());
        let handle = conversation_scroll_handle(cx, self.scroll_handle.clone());
        render_conversation_root(
            cx,
            &theme,
            layout,
            self.content_padding,
            self.content_gap,
            self.content_revision,
            self.stick_to_bottom,
            self.stick_threshold,
            self.show_scroll_to_bottom_button,
            self.test_id,
            handle,
            self.children,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{ConversationDownload, ConversationScrollButton};
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{ElementKind, TextProps};

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

    #[test]
    fn conversation_direct_children_resolve_content_and_overlay_slots() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let handle = ScrollHandle::default();
        handle.set_viewport_size(Size::new(Px(320.0), Px(120.0)));
        handle.set_content_size(Size::new(Px(320.0), Px(720.0)));

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Conversation::new([])
                    .scroll_handle(handle.clone())
                    .stick_to_bottom(false)
                    .test_id("conversation-root")
                    .into_element_with_children(cx, |cx| {
                        vec![
                            ConversationContent::new([cx.text("Body")])
                                .test_id("conversation-content")
                                .into_element(cx),
                            ConversationDownload::new("Download")
                                .children([cx.text("Download")])
                                .into_element(cx),
                            ConversationScrollButton::default()
                                .test_id("conversation-scroll")
                                .into_element(cx),
                        ]
                    })
            });

        assert!(has_test_id(&element, "conversation-root"));
        assert!(has_test_id(&element, "conversation-content"));
        assert!(has_test_id(&element, "conversation-scroll.chrome"));
        assert!(has_text(&element, "Download"));
    }

    #[test]
    fn conversation_legacy_children_render_without_explicit_content_slot() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let handle = ScrollHandle::default();
        handle.set_viewport_size(Size::new(Px(320.0), Px(160.0)));
        handle.set_content_size(Size::new(Px(320.0), Px(320.0)));

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Conversation::new([cx.text("Legacy body")])
                    .scroll_handle(handle.clone())
                    .stick_to_bottom(false)
                    .into_element(cx)
            });

        assert!(has_text(&element, "Legacy body"));
    }
}
