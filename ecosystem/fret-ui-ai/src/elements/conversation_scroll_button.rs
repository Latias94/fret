use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::scroll::{ScrollStrategy, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{Justify, LayoutRefinement, Space};

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

#[derive(Clone)]
/// An overlay “scroll to bottom” affordance driven by a `VirtualListScrollHandle`.
///
/// Callers should compose this inside a `relative()` root so its absolute positioning resolves
/// correctly.
pub struct ConversationScrollButton {
    handle: VirtualListScrollHandle,
    threshold: Px,
    label: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ConversationScrollButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversationScrollButton")
            .field("threshold", &self.threshold)
            .field("label", &self.label.as_ref())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ConversationScrollButton {
    pub fn new(handle: VirtualListScrollHandle) -> Self {
        Self {
            handle,
            threshold: Px(8.0),
            label: Arc::<str>::from("Scroll to bottom"),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn threshold(mut self, threshold: Px) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    /// Returns an overlay element that is hidden when the handle is effectively at bottom.
    ///
    /// Callers should compose this inside a `relative()` root (e.g. a `Stack`) so the absolute
    /// positioning resolves correctly.
    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let max = self.handle.max_offset();
        let offset = self.handle.offset();
        let is_at_bottom = (max.y.0 - offset.y.0) <= self.threshold.0;

        let bottom_offset = theme
            .metric_by_key("fret.ai.conversation.scroll_button.offset_bottom")
            .unwrap_or_else(|| decl_style::space(&theme, Space::N4));

        let mut overlay_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .absolute()
                .left(Space::N0)
                .right(Space::N0)
                .bottom(Space::N4)
                .merge(self.layout),
        );
        overlay_layout.inset.bottom = Some(bottom_offset);

        if is_at_bottom {
            return cx.container(
                ContainerProps {
                    layout: overlay_layout,
                    ..Default::default()
                },
                |_cx| Vec::<AnyElement>::new(),
            );
        }

        let handle_for_button = self.handle.clone();
        let on_activate: OnActivate = std::sync::Arc::new(move |host, action_cx, _reason| {
            handle_for_button.scroll_to_item(usize::MAX, ScrollStrategy::End);
            host.request_redraw(action_cx.window);
        });

        let mut button = Button::new(self.label)
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Icon)
            .children(vec![cx.text("↓")])
            .on_activate(on_activate);

        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }

        let button = button.into_element(cx);

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .justify(Justify::Center),
            move |_cx| vec![button],
        );

        cx.container(
            ContainerProps {
                layout: overlay_layout,
                padding: Edges::all(Px(0.0)),
                background: None,
                corner_radii: Corners::all(Px(0.0)),
                ..Default::default()
            },
            move |_cx| vec![row],
        )
    }
}
