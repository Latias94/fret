use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_icons::IconId;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::scroll::{ScrollHandle, VirtualListScrollHandle};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{Items, Justify, LayoutRefinement, Space};

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

use super::conversation::{
    CONVERSATION_SCROLL_BUTTON_SLOT_KEY, conversation_slot_placeholder, use_conversation_context,
};

#[derive(Clone)]
/// An overlay “scroll to bottom” affordance driven by a `VirtualListScrollHandle`.
///
/// Callers should compose this inside a `relative()` root so its absolute positioning resolves
/// correctly.
pub struct ConversationScrollButton {
    handle: Option<ScrollHandle>,
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
            handle: Some(handle.base_handle().clone()),
            threshold: Px(8.0),
            label: Arc::<str>::from("Scroll to bottom"),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn from_scroll_handle(handle: ScrollHandle) -> Self {
        Self {
            handle: Some(handle),
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
        let context_handle = use_conversation_context(cx).map(|context| context.scroll_handle());
        let Some(handle) = self.handle.or(context_handle) else {
            let semantics = self
                .test_id
                .map(|id| fret_ui::element::SemanticsDecoration::default().test_id(id));
            return conversation_slot_placeholder(
                cx,
                CONVERSATION_SCROLL_BUTTON_SLOT_KEY,
                semantics,
                Vec::new(),
            );
        };

        let max = handle.max_offset();
        let offset = handle.offset();
        let is_at_bottom = (max.y.0 - offset.y.0) <= self.threshold.0;
        let in_conversation = use_conversation_context(cx).is_some();

        if in_conversation && is_at_bottom {
            let semantics = self
                .test_id
                .map(|id| fret_ui::element::SemanticsDecoration::default().test_id(id));
            return conversation_slot_placeholder(
                cx,
                CONVERSATION_SCROLL_BUTTON_SLOT_KEY,
                semantics,
                Vec::new(),
            );
        }

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
        overlay_layout.inset.bottom = Some(bottom_offset).into();

        if is_at_bottom {
            return cx.container(
                ContainerProps {
                    layout: overlay_layout,
                    ..Default::default()
                },
                |_cx| Vec::<AnyElement>::new(),
            );
        }

        let handle_for_button = handle.clone();
        let on_activate: OnActivate = std::sync::Arc::new(move |host, action_cx, _reason| {
            let max = handle_for_button.max_offset();
            let cur = handle_for_button.offset();
            handle_for_button.scroll_to_offset(fret_core::Point::new(cur.x, max.y));
            host.request_redraw(action_cx.window);
        });

        let mut button = Button::new("")
            .a11y_label(self.label)
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Icon)
            .leading_icon(IconId::new_static("lucide.arrow-down"))
            .corner_radii_override(Corners::all(Px(999.0)))
            .on_activate(on_activate);

        if let Some(test_id) = self.test_id {
            button = button.test_id(test_id);
        }

        let button = button.into_element(cx);

        if in_conversation {
            return button.key_context(CONVERSATION_SCROLL_BUTTON_SLOT_KEY);
        }

        let row = ui::h_row(move |_cx| vec![button])
            .layout(LayoutRefinement::default().w_full())
            .justify(Justify::Center)
            .items(Items::Center)
            .into_element(cx);

        cx.container(
            ContainerProps {
                layout: overlay_layout,
                padding: Edges::all(Px(0.0)).into(),
                background: None,
                corner_radii: Corners::all(Px(0.0)),
                ..Default::default()
            },
            move |_cx| vec![row],
        )
    }
}

impl Default for ConversationScrollButton {
    fn default() -> Self {
        Self {
            handle: None,
            threshold: Px(8.0),
            label: Arc::<str>::from("Scroll to bottom"),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }
}
