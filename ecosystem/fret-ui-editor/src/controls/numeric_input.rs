//! Numeric text input control with editor-style commit/cancel outcomes.
//!
//! This control is intentionally lightweight:
//! - it owns a per-element draft `Model<String>` for text editing,
//! - commits parsed values on Enter,
//! - validates on commit (optional),
//! - cancels (reverts to formatted current value) on Escape,
//! - renders an inline error message when commit is rejected.

use std::panic::Location;
use std::sync::{Arc, Mutex};

use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::NumericPresentation;

pub use crate::primitives::NumericInputSelectionBehavior;

mod element;
mod keyboard;
mod model;
mod session;
#[cfg(test)]
mod tests;

use element::{numeric_input_hidden_text_entry, numeric_input_into_element_keyed};
pub use model::{
    NumericFormatFn, NumericInputErrorDisplay, NumericInputOptions, NumericInputOutcome,
    NumericParseFn, NumericValidateFn, OnNumericInputOutcome,
};

#[derive(Clone)]
pub struct NumericInput<T> {
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    on_outcome: Option<OnNumericInputOutcome>,
    options: NumericInputOptions,
    focus_target: Option<Arc<Mutex<Option<fret_ui::GlobalElementId>>>>,
}

impl<T> NumericInput<T>
where
    T: Copy + Default + 'static,
{
    pub fn new(model: Model<T>, format: NumericFormatFn<T>, parse: NumericParseFn<T>) -> Self {
        Self {
            model,
            format,
            parse,
            validate: None,
            on_outcome: None,
            options: NumericInputOptions::default(),
            focus_target: None,
        }
    }

    /// Construct a numeric input from a shared editor authoring bundle.
    pub fn from_presentation(model: Model<T>, presentation: NumericPresentation<T>) -> Self {
        let mut input = Self::new(model, presentation.format(), presentation.parse());
        input.options.prefix = presentation.chrome_prefix().cloned();
        input.options.suffix = presentation.chrome_suffix().cloned();
        input
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn on_outcome(mut self, on_outcome: Option<OnNumericInputOutcome>) -> Self {
        self.on_outcome = on_outcome;
        self
    }

    pub fn options(mut self, options: NumericInputOptions) -> Self {
        self.options = options;
        self
    }

    pub(crate) fn focus_target(
        mut self,
        focus_target: Arc<Mutex<Option<fret_ui::GlobalElementId>>>,
    ) -> Self {
        self.focus_target = Some(focus_target);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_at(cx, Location::caller(), None)
    }

    #[track_caller]
    pub(crate) fn into_element_with_hidden_text_entry_layout<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        hidden_layout: Option<LayoutStyle>,
    ) -> AnyElement {
        self.into_element_at(cx, Location::caller(), hidden_layout)
    }

    fn into_element_at<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        loc: &'static Location<'static>,
        hidden_layout: Option<LayoutStyle>,
    ) -> AnyElement {
        let model_id = self.model.id();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed_at(
                loc,
                ("fret-ui-editor.numeric_input", id_source, model_id),
                |cx| self.into_element_keyed_or_hidden(cx, hidden_layout),
            )
        } else {
            cx.keyed_at(
                loc,
                ("fret-ui-editor.numeric_input", callsite, model_id),
                |cx| self.into_element_keyed_or_hidden(cx, hidden_layout),
            )
        }
    }

    fn into_element_keyed_or_hidden<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        hidden_layout: Option<LayoutStyle>,
    ) -> AnyElement {
        if let Some(layout) = hidden_layout {
            numeric_input_hidden_text_entry(self, cx, layout)
        } else {
            numeric_input_into_element_keyed(self, cx)
        }
    }
}
