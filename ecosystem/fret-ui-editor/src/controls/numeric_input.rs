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
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::NumericPresentation;

pub use crate::primitives::NumericInputSelectionBehavior;

mod element;
mod keyboard;
mod model;
mod session;
#[cfg(test)]
mod tests;

use element::numeric_input_into_element_keyed;
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
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(
                ("fret-ui-editor.numeric_input", id_source, model_id),
                |cx| self.into_element_keyed(cx),
            )
        } else {
            cx.keyed(("fret-ui-editor.numeric_input", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        numeric_input_into_element_keyed(self, cx)
    }
}
