use std::sync::Arc;

use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::LayoutRefinement;

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant};

#[derive(Clone)]
pub struct InlineCitation {
    label: Arc<str>,
    on_activate: Option<OnActivate>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for InlineCitation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineCitation")
            .field("label", &self.label.as_ref())
            .field("has_on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl InlineCitation {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            on_activate: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
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
            .variant(ButtonVariant::Link)
            .size(ButtonSize::Sm)
            .refine_layout(self.layout);

        if let Some(on_activate) = self.on_activate {
            btn = btn.on_activate(on_activate);
        }
        if let Some(id) = self.test_id {
            btn = btn.test_id(id);
        }

        btn.into_element(cx)
    }
}
