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

use fret_ui_shadcn::facade::{Button, ButtonSize, ButtonVariant};

use super::conversation::{
    CONVERSATION_SCROLL_BUTTON_SLOT_KEY, conversation_slot_placeholder, use_conversation_context,
};

/// An overlay “scroll to bottom” affordance driven by a `VirtualListScrollHandle`.
///
/// Callers should compose this inside a `relative()` root so its absolute positioning resolves
/// correctly.
pub struct ConversationScrollButton {
    handle: Option<ScrollHandle>,
    threshold: Px,
    label: Arc<str>,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ConversationScrollButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversationScrollButton")
            .field("threshold", &self.threshold)
            .field("label", &self.label.as_ref())
            .field("has_children", &self.children.is_some())
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
            children: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn from_scroll_handle(handle: ScrollHandle) -> Self {
        Self {
            handle: Some(handle),
            threshold: Px(8.0),
            label: Arc::<str>::from("Scroll to bottom"),
            children: None,
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

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
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
    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        self.children(children(cx)).into_element(cx)
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

        let mut button = if let Some(children) = self.children {
            Button::new("")
                .a11y_label(self.label)
                .children(children)
                .size(ButtonSize::Sm)
        } else {
            Button::new("")
                .a11y_label(self.label)
                .size(ButtonSize::Icon)
                .leading_icon(IconId::new_static("lucide.arrow-down"))
        }
        .variant(ButtonVariant::Outline)
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
            children: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Conversation, ConversationContent};
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
    fn custom_children_render_inside_conversation_overlay_slot() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let handle = ScrollHandle::default();
        handle.set_viewport_size(Size::new(Px(320.0), Px(160.0)));
        handle.set_content_size(Size::new(Px(320.0), Px(640.0)));

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Conversation::new([])
                    .scroll_handle(handle.clone())
                    .stick_to_bottom(false)
                    .into_element_with_children(cx, |cx| {
                        vec![
                            ConversationContent::new([cx.text("Body")]).into_element(cx),
                            ConversationScrollButton::default()
                                .test_id("conversation-scroll")
                                .children([cx.text("Jump to latest")])
                                .into_element(cx),
                        ]
                    })
            });

        assert!(has_text(&element, "Jump to latest"));
        assert!(has_test_id(&element, "conversation-scroll.chrome"));
    }
}
