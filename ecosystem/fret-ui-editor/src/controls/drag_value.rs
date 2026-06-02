//! Editor-grade numeric control: drag-to-scrub with an optional typing mode.
//!
//! v1 goals (workstream):
//! - scrub (drag-to-change) with Shift slow / Alt fast outcomes,
//! - double-click to switch into a typing mode,
//! - Escape cancels scrub to the pre-edit value (handled by `DragValueCore`).

use std::panic::Location;
use std::sync::Arc;

use crate::controls::numeric_input::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::{EditSessionOutcome, NumericPresentation};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

#[cfg(test)]
mod tests;

mod element;
mod model;
mod options;
mod scrub;
mod scrub_element;
mod session;
mod typing;

use element::drag_value_into_element_keyed;
pub use options::DragValueOptions;

pub type DragValueOutcome = EditSessionOutcome;
pub type OnDragValueOutcome =
    Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, DragValueOutcome) + 'static>;

#[derive(Clone)]
pub struct DragValue<T> {
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    on_outcome: Option<OnDragValueOutcome>,
    options: DragValueOptions,
}

impl<T> DragValue<T>
where
    T: DragValueScalar + Default,
{
    pub fn new(model: Model<T>, format: NumericFormatFn<T>, parse: NumericParseFn<T>) -> Self {
        Self {
            model,
            format,
            parse,
            validate: None,
            on_outcome: None,
            options: DragValueOptions::default(),
        }
    }

    /// Construct a drag value from a shared editor authoring bundle.
    pub fn from_presentation(model: Model<T>, presentation: NumericPresentation<T>) -> Self {
        let mut drag_value = Self::new(model, presentation.format(), presentation.parse());
        drag_value.options.prefix = presentation.chrome_prefix().cloned();
        drag_value.options.suffix = presentation.chrome_suffix().cloned();
        drag_value
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn on_outcome(mut self, on_outcome: Option<OnDragValueOutcome>) -> Self {
        self.on_outcome = on_outcome;
        self
    }

    pub fn options(mut self, options: DragValueOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.drag_value", id_source, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.drag_value", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        drag_value_into_element_keyed(self, cx)
    }
}
