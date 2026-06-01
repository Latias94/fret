//! Editor-grade horizontal slider control (v1).
//!
//! This is intentionally a small, policy-layer widget:
//! - pointer down sets the value (clamped / stepped),
//! - pointer drag updates the value continuously (best-effort cleanup when pointer-up is missed),
//! - visuals reuse the shared editor "frame" chrome policy to stay consistent with other controls.
//! - optional value display and a typing mode (double-click).

use std::panic::Location;

use crate::controls::numeric_input::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::NumericPresentation;
use crate::primitives::drag_value_core::DragValueScalar;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

mod chrome;
mod element;
mod frame;
mod model;
mod pointer;
#[cfg(test)]
mod tests;
mod typing;
mod value_math;

use element::slider_into_element_keyed;
pub use model::SliderOptions;
use model::{default_slider_format, default_slider_parse};

#[derive(Clone)]
pub struct Slider<T> {
    model: Model<T>,
    min: f64,
    max: f64,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    options: SliderOptions,
}

impl<T> Slider<T>
where
    T: DragValueScalar + Default,
{
    pub fn new(model: Model<T>, min: f64, max: f64) -> Self {
        Self {
            model,
            min,
            max,
            format: default_slider_format(),
            parse: default_slider_parse(),
            validate: None,
            options: SliderOptions::default(),
        }
    }

    /// Construct a slider from a shared editor authoring bundle.
    pub fn from_presentation(
        model: Model<T>,
        min: f64,
        max: f64,
        presentation: NumericPresentation<T>,
    ) -> Self {
        let mut slider = Self::new(model, min, max);
        slider.format = presentation.format();
        slider.parse = presentation.parse();
        slider.options.prefix = presentation.chrome_prefix().cloned();
        slider.options.suffix = presentation.chrome_suffix().cloned();
        slider
    }

    pub fn format(mut self, format: NumericFormatFn<T>) -> Self {
        self.format = format;
        self
    }

    pub fn parse(mut self, parse: NumericParseFn<T>) -> Self {
        self.parse = parse;
        self
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn options(mut self, options: SliderOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        // Important: key internal state per slider instance so multiple sliders don't share
        // drag/typing state.
        //
        // Do not use `test_id` for identity: test ids are for diagnostics/automation, not widget
        // identity. Instead, follow egui/imgui-style identity rules:
        // - Prefer an explicit `id_source` (PushID/id_source equivalent) when provided.
        // - Otherwise key by `(callsite, model.id())` to prevent helper-function callsite
        //   collisions while keeping per-instance state separation.
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());

        let id_source = self.options.id_source.clone();
        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.slider", id_source, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.slider", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        slider_into_element_keyed(self, cx)
    }
}
