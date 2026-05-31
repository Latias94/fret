//! Axis-labeled drag value (joined input group).
//!
//! This is used by Vec/Transform-style inspectors where the axis marker ("X/Y/Z/W") should feel
//! like part of the numeric field instead of a separate, differently-styled widget.

use std::panic::Location;
use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::controls::numeric_input::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::NumericPresentation;
use crate::primitives::drag_value_core::DragValueScalar;

mod element;
mod ids;
mod model;
mod session;
#[cfg(test)]
mod tests;

pub use model::{
    AxisDragValueOptions, AxisDragValueOutcome, AxisDragValueResetAction, OnAxisDragValueOutcome,
};

#[derive(Clone)]
pub struct AxisDragValue<T> {
    axis_label: Arc<str>,
    axis_tint: Color,
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    on_outcome: Option<OnAxisDragValueOutcome>,
    options: AxisDragValueOptions,
}

impl<T> AxisDragValue<T>
where
    T: DragValueScalar + Default,
{
    pub fn new(
        axis_label: Arc<str>,
        axis_tint: Color,
        model: Model<T>,
        format: NumericFormatFn<T>,
        parse: NumericParseFn<T>,
    ) -> Self {
        Self {
            axis_label,
            axis_tint,
            model,
            format,
            parse,
            validate: None,
            on_outcome: None,
            options: AxisDragValueOptions::default(),
        }
    }

    /// Construct an axis drag value from a shared editor authoring bundle.
    pub fn from_presentation(
        axis_label: Arc<str>,
        axis_tint: Color,
        model: Model<T>,
        presentation: NumericPresentation<T>,
    ) -> Self {
        let mut drag_value = Self::new(
            axis_label,
            axis_tint,
            model,
            presentation.format(),
            presentation.parse(),
        );
        drag_value.options.prefix = presentation.chrome_prefix().cloned();
        drag_value.options.suffix = presentation.chrome_suffix().cloned();
        drag_value
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn on_outcome(mut self, on_outcome: Option<OnAxisDragValueOutcome>) -> Self {
        self.on_outcome = on_outcome;
        self
    }

    pub fn options(mut self, options: AxisDragValueOptions) -> Self {
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
            cx.keyed(
                ("fret-ui-editor.axis_drag_value", id_source, model_id),
                |cx| self.into_element_keyed(cx),
            )
        } else {
            cx.keyed(
                ("fret-ui-editor.axis_drag_value", callsite, model_id),
                |cx| self.into_element_keyed(cx),
            )
        }
    }
}
