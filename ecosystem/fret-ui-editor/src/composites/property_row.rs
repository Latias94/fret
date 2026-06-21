//! Inspector-style property row composite (label + value + actions).
mod element;
mod layout;
mod options;
mod reset;

use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::inspector_layout::InspectorLayoutMetrics;
use crate::primitives::readout::editor_property_row_label_text_props;

use element::property_row_element;
pub use layout::PropertyRowLayoutVariant;
pub use options::PropertyRowOptions;
pub use reset::{OnPropertyRowReset, PropertyRowReset, PropertyRowResetOptions};

#[cfg(test)]
pub(crate) use element::PROPERTY_ROW_VALUE_SLOT;

pub(crate) fn property_row_label_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let (fg, row_height) = {
        let theme = Theme::global(&*cx.app);
        let metrics = InspectorLayoutMetrics::resolve(theme);
        (editor_muted_foreground(theme), metrics.density.row_height)
    };

    cx.text_props(editor_property_row_label_text_props(
        text.into(),
        fg,
        row_height,
    ))
}

#[derive(Clone, Default)]
pub struct PropertyRow {
    pub options: PropertyRowOptions,
    pub reset: Option<PropertyRowReset>,
}

impl PropertyRow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn options(mut self, options: PropertyRowOptions) -> Self {
        self.options = options;
        self
    }

    pub fn reset(mut self, reset: Option<PropertyRowReset>) -> Self {
        self.reset = reset;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        label: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        value: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
    ) -> AnyElement {
        let options = self.options;
        let reset = self.reset;
        let id_source = options.id_source.clone();
        if let Some(id_source) = id_source.as_deref() {
            // Only key when the caller provides an explicit identity source. Keying by callsite
            // alone breaks loop-built rows by collapsing them into a single element identity.
            cx.keyed(("fret-ui-editor.property_row", id_source), move |cx| {
                property_row_element(cx, options, reset, label, value, actions)
            })
        } else {
            property_row_element(cx, options, reset, label, value, actions)
        }
    }
}

#[cfg(test)]
mod tests;
