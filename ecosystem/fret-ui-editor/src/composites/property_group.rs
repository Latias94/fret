//! Inspector-style property group (collapsible header + section body).

mod element;
mod options;

use std::sync::Arc;

use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, ElementContextAccess, UiHost};

use element::property_group_element;

pub use options::PropertyGroupOptions;

pub type OnPropertyGroupToggle = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, bool) + 'static>;

#[derive(Clone)]
pub struct PropertyGroup {
    label: Arc<str>,
    options: PropertyGroupOptions,
    on_toggle: Option<OnPropertyGroupToggle>,
}

impl PropertyGroup {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            options: PropertyGroupOptions::default(),
            on_toggle: None,
        }
    }

    pub fn options(mut self, options: PropertyGroupOptions) -> Self {
        self.options = options;
        self
    }

    pub fn on_toggle(mut self, on_toggle: Option<OnPropertyGroupToggle>) -> Self {
        self.on_toggle = on_toggle;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        header_actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
        contents: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        property_group_element(
            cx,
            self.label,
            self.options,
            self.on_toggle,
            header_actions,
            contents,
        )
    }

    #[track_caller]
    pub fn into_element_in<'a, H: UiHost + 'a, Cx>(
        self,
        cx: &mut Cx,
        header_actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
        contents: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement
    where
        Cx: ElementContextAccess<'a, H>,
    {
        self.into_element(cx.elements(), header_actions, contents)
    }
}
