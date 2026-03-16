use std::any::Any;
use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_icons::IconId;
use fret_runtime::ActionId;
use fret_ui::action::OnActivate;
use fret_ui::element::{AnyElement, ContainerProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{Items, Justify, LayoutRefinement, Space};

use fret_ui_shadcn::facade::{Button, ButtonSize, ButtonVariant};

use super::conversation::{CONVERSATION_DOWNLOAD_SLOT_KEY, use_conversation_context};

type ActionPayloadFactory = Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>;

/// A small “download/export transcript” button.
///
/// This component emits an intent via `on_activate`; performing the actual effect (clipboard/file IO)
/// is app-owned.
pub struct ConversationDownload {
    label: Arc<str>,
    disabled: bool,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<OnActivate>,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    icon: IconId,
    show_label: bool,
}

impl std::fmt::Debug for ConversationDownload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversationDownload")
            .field("label", &self.label.as_ref())
            .field("disabled", &self.disabled)
            .field("action", &self.action)
            .field("action_payload", &self.action_payload.is_some())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("has_children", &self.children.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ConversationDownload {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            action: None,
            action_payload: None,
            on_activate: None,
            children: None,
            test_id: None,
            layout: LayoutRefinement::default(),
            icon: IconId::new_static("lucide.download"),
            show_label: false,
        }
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = icon;
        self
    }

    /// Show a text label (instead of the default icon-only affordance).
    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Bind a stable action ID to this conversation download control (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Attach a payload for parameterized conversation-download actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`ConversationDownload::action_payload`], but computes the payload lazily.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let has_custom_children = self.children.is_some();
        let theme = Theme::global(&*cx.app).clone();
        let layout = self.layout;
        let in_conversation = use_conversation_context(cx).is_some();

        let mut btn = if let Some(children) = self.children {
            Button::new("")
                .children(children)
                .size(ButtonSize::Icon)
                .refine_layout(layout.clone())
        } else if self.show_label {
            Button::new(self.label.clone())
                .size(ButtonSize::Sm)
                .refine_layout(layout.clone())
        } else {
            Button::new("")
                .a11y_label(self.label)
                .size(ButtonSize::Icon)
                .leading_icon(self.icon)
                .corner_radii_override(Corners::all(Px(999.0)))
        }
        .variant(ButtonVariant::Outline)
        .disabled(self.disabled);

        if let Some(action) = self.action {
            btn = btn.action(action);
        }
        if let Some(payload) = self.action_payload {
            btn = btn.action_payload_factory(payload);
        }
        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(on_activate);
        }
        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }

        let btn = btn.into_element(cx);

        if in_conversation {
            return btn.key_context(CONVERSATION_DOWNLOAD_SLOT_KEY);
        }

        if self.show_label || has_custom_children {
            return btn;
        }

        let overlay_layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .absolute()
                .left(Space::N0)
                .right(Space::N0)
                .top(Space::N4)
                .merge(layout),
        );
        let pad = decl_style::space(&theme, Space::N4);

        cx.container(
            ContainerProps {
                layout: overlay_layout,
                padding: Edges::symmetric(pad, Px(0.0)).into(),
                ..Default::default()
            },
            move |cx| {
                vec![
                    ui::h_row(move |_cx| vec![btn])
                        .layout(LayoutRefinement::default().w_full())
                        .justify(Justify::End)
                        .items(Items::Center)
                        .into_element(cx),
                ]
            },
        )
    }
}
