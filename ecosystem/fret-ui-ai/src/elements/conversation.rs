use fret_core::{Edges, Point, Px};
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps, LayoutStyle, StackProps};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{LayoutRefinement, Space};

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant, ScrollArea};

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
