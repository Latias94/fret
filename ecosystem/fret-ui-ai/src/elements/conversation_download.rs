use std::sync::Arc;

use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::LayoutRefinement;

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

#[derive(Clone)]
/// A small “download/export transcript” button.
///
/// This component emits an intent via `on_activate`; performing the actual effect (clipboard/file IO)
/// is app-owned.
pub struct ConversationDownload {
    label: Arc<str>,
    disabled: bool,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ConversationDownload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversationDownload")
            .field("label", &self.label.as_ref())
            .field("disabled", &self.disabled)
            .field("has_on_activate", &self.on_activate.is_some())
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
            on_activate: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
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
        let mut btn = Button::new(self.label)
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Sm)
            .disabled(self.disabled)
            .refine_layout(self.layout);

        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(on_activate);
        }
        if let Some(test_id) = self.test_id {
            btn = btn.test_id(test_id);
        }

        btn.into_element(cx)
    }
}
