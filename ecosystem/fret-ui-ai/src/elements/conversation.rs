use std::sync::Arc;

use fret_core::{Edges, Point, Px, SemanticsRole};
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, SemanticsProps, StackProps};
use fret_ui::scroll::{ScrollHandle, ScrollStrategy, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{LayoutRefinement, Space};

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant, ScrollArea};

use crate::{Message, MessageRole};

#[derive(Debug, Clone)]
pub struct ConversationMessage {
    pub id: u64,
    pub role: MessageRole,
    pub text: Arc<str>,
}

impl ConversationMessage {
    pub fn new(id: u64, role: MessageRole, text: impl Into<Arc<str>>) -> Self {
        Self {
            id,
            role,
            text: text.into(),
        }
    }
}

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
        let handle = cx.with_state(ConversationState::default, |st| {
            if let Some(handle) = provided_handle.clone() {
                st.handle = handle;
            }
            st.handle.clone()
        });

        let max = handle.max_offset();
        let offset = handle.offset();
        let is_at_bottom = (max.y.0 - offset.y.0) <= stick_threshold.0;

        let effectively_at_bottom = cx.with_state(ConversationState::default, |st| {
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

                let bubble = Message::new(msg.role, msg.text.clone()).into_element(cx);
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
                layout: LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: fret_ui::element::Length::Fill,
                        height: fret_ui::element::Length::Fill,
                        ..Default::default()
                    },
                    overflow: fret_ui::element::Overflow::Clip,
                    ..Default::default()
                },
                padding: Edges::all(padding_px),
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

                    let button = Button::new("Scroll to bottom")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .children(vec![cx.text("↓")])
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

                    let button_for_row = button.clone();
                    out.push(cx.container(
                        ContainerProps {
                            layout: overlay_layout,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .layout(LayoutRefinement::default().w_full())
                                    .justify_center(),
                                move |_cx| vec![button_for_row.clone()],
                            )]
                        },
                    ));
                }

                out
            },
        )
    }
}

#[derive(Debug, Clone)]
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

    pub fn scroll_handle(mut self, handle: ScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        #[derive(Debug, Default, Clone)]
        struct ConversationState {
            handle: ScrollHandle,
            last_revision: u64,
            pending_scroll_frames: u8,
            was_at_bottom: bool,
            initialized: bool,
        }

        let theme = Theme::global(&*cx.app).clone();

        let layout = self
            .layout
            .merge(LayoutRefinement::default().w_full().h_full().relative());
        let scroll_layout = LayoutRefinement::default().w_full().h_full();

        let stick_to_bottom = self.stick_to_bottom;
        let stick_threshold = self.stick_threshold;
        let revision = self.content_revision;
        let show_scroll_to_bottom_button = self.show_scroll_to_bottom_button;

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

        let effectively_at_bottom = cx.with_state(ConversationState::default, |st| {
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
                let next = Point::new(handle.offset().x, handle.max_offset().y);
                handle.scroll_to_offset(next);
                st.pending_scroll_frames = st.pending_scroll_frames.saturating_sub(1);
            }

            st.last_revision = revision;
            st.was_at_bottom = eligible;
            is_at_bottom || st.pending_scroll_frames > 0
        });

        let content_padding = self.content_padding;
        let content_gap = self.content_gap;
        let children = self.children;
        let padding_px = decl_style::space(&theme, content_padding);
        let content = cx.container(
            ContainerProps {
                layout: LayoutStyle::default(),
                padding: Edges::all(padding_px),
                ..Default::default()
            },
            move |cx| {
                vec![stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .gap(content_gap)
                        .layout(LayoutRefinement::default().w_full())
                        .items_stretch(),
                    move |_cx| children,
                )]
            },
        );

        let scroll_area = ScrollArea::new(vec![content])
            .scroll_handle(handle.clone())
            .refine_layout(scroll_layout)
            .into_element(cx);

        let root_layout = decl_style::layout_style(&theme, layout);
        cx.stack_props(
            StackProps {
                layout: root_layout,
            },
            move |cx| {
                let mut out = vec![scroll_area];

                if show_scroll_to_bottom_button && !effectively_at_bottom {
                    let handle_for_button = handle.clone();
                    let on_activate: OnActivate = std::sync::Arc::new(move |host, cx, _reason| {
                        let max = handle_for_button.max_offset();
                        let cur = handle_for_button.offset();
                        handle_for_button.scroll_to_offset(Point::new(cur.x, max.y));
                        host.request_redraw(cx.window);
                    });

                    let button = Button::new("Scroll to bottom")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Icon)
                        .children(vec![cx.text("↓")])
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

                    let button_for_row = button.clone();
                    out.push(cx.container(
                        ContainerProps {
                            layout: overlay_layout,
                            ..Default::default()
                        },
                        move |cx| {
                            vec![stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .layout(LayoutRefinement::default().w_full())
                                    .justify_center(),
                                move |_cx| vec![button_for_row.clone()],
                            )]
                        },
                    ));
                }

                out
            },
        )
    }
}
