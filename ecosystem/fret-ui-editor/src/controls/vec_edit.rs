//! Vector editors (Vec2/Vec3/Vec4) built on top of `DragValue<T>`.
//!
//! These controls are policy-heavy and meant for inspector-like surfaces:
//! - compact axis labels (X/Y/Z/W)
//! - axis color tokens (`editor.axis.*`)
//! - shared numeric formatting/parsing policies

use std::panic::Location;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::controls::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::NumericPresentation;

mod axis;
mod element;
mod layout;
mod options;

pub use axis::{
    AxisReset, AxisResetOptions, OnAxisReset, OnVecEditAxisOutcome, VecEditAxis, VecEditAxisOutcome,
};
pub use options::{VecEditLayoutVariant, VecEditOptions};

#[derive(Clone)]
pub struct Vec2Edit<T> {
    pub x: Model<T>,
    pub y: Model<T>,
    pub reset_x: Option<AxisReset>,
    pub reset_y: Option<AxisReset>,
    pub format: NumericFormatFn<T>,
    pub parse: NumericParseFn<T>,
    pub validate: Option<NumericValidateFn<T>>,
    pub on_axis_outcome: Option<OnVecEditAxisOutcome>,
    pub options: VecEditOptions,
}

impl<T> Vec2Edit<T>
where
    T: crate::primitives::drag_value_core::DragValueScalar + Default,
{
    pub fn new(
        x: Model<T>,
        y: Model<T>,
        format: NumericFormatFn<T>,
        parse: NumericParseFn<T>,
    ) -> Self {
        Self {
            x,
            y,
            reset_x: None,
            reset_y: None,
            format,
            parse,
            validate: None,
            on_axis_outcome: None,
            options: VecEditOptions::default(),
        }
    }

    /// Construct a vec editor from a shared editor authoring bundle.
    pub fn from_presentation(
        x: Model<T>,
        y: Model<T>,
        presentation: NumericPresentation<T>,
    ) -> Self {
        let mut edit = Self::new(x, y, presentation.format(), presentation.parse());
        edit.options.prefix = presentation.chrome_prefix().cloned();
        edit.options.suffix = presentation.chrome_suffix().cloned();
        edit
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn reset_x(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_x = reset;
        self
    }

    pub fn reset_y(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_y = reset;
        self
    }

    pub fn options(mut self, options: VecEditOptions) -> Self {
        self.options = options;
        self
    }

    pub fn on_axis_outcome(mut self, on_axis_outcome: Option<OnVecEditAxisOutcome>) -> Self {
        self.on_axis_outcome = on_axis_outcome;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let x_id = self.x.id();
        let y_id = self.y.id();
        let model_ids = (x_id, y_id);

        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());

        let id_source = self.options.id_source.clone();
        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.vec2_edit", id_source, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.vec2_edit", callsite, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }
}

#[derive(Clone)]
pub struct Vec3Edit<T> {
    pub x: Model<T>,
    pub y: Model<T>,
    pub z: Model<T>,
    pub reset_x: Option<AxisReset>,
    pub reset_y: Option<AxisReset>,
    pub reset_z: Option<AxisReset>,
    pub format: NumericFormatFn<T>,
    pub parse: NumericParseFn<T>,
    pub validate: Option<NumericValidateFn<T>>,
    pub on_axis_outcome: Option<OnVecEditAxisOutcome>,
    pub options: VecEditOptions,
}

impl<T> Vec3Edit<T>
where
    T: crate::primitives::drag_value_core::DragValueScalar + Default,
{
    pub fn new(
        x: Model<T>,
        y: Model<T>,
        z: Model<T>,
        format: NumericFormatFn<T>,
        parse: NumericParseFn<T>,
    ) -> Self {
        Self {
            x,
            y,
            z,
            reset_x: None,
            reset_y: None,
            reset_z: None,
            format,
            parse,
            validate: None,
            on_axis_outcome: None,
            options: VecEditOptions::default(),
        }
    }

    /// Construct a vec editor from a shared editor authoring bundle.
    pub fn from_presentation(
        x: Model<T>,
        y: Model<T>,
        z: Model<T>,
        presentation: NumericPresentation<T>,
    ) -> Self {
        let mut edit = Self::new(x, y, z, presentation.format(), presentation.parse());
        edit.options.prefix = presentation.chrome_prefix().cloned();
        edit.options.suffix = presentation.chrome_suffix().cloned();
        edit
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn reset_x(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_x = reset;
        self
    }

    pub fn reset_y(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_y = reset;
        self
    }

    pub fn reset_z(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_z = reset;
        self
    }

    pub fn options(mut self, options: VecEditOptions) -> Self {
        self.options = options;
        self
    }

    pub fn on_axis_outcome(mut self, on_axis_outcome: Option<OnVecEditAxisOutcome>) -> Self {
        self.on_axis_outcome = on_axis_outcome;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let x_id = self.x.id();
        let y_id = self.y.id();
        let z_id = self.z.id();
        let model_ids = (x_id, y_id, z_id);

        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());

        let id_source = self.options.id_source.clone();
        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.vec3_edit", id_source, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.vec3_edit", callsite, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }
}

#[derive(Clone)]
pub struct Vec4Edit<T> {
    pub x: Model<T>,
    pub y: Model<T>,
    pub z: Model<T>,
    pub w: Model<T>,
    pub reset_x: Option<AxisReset>,
    pub reset_y: Option<AxisReset>,
    pub reset_z: Option<AxisReset>,
    pub reset_w: Option<AxisReset>,
    pub format: NumericFormatFn<T>,
    pub parse: NumericParseFn<T>,
    pub validate: Option<NumericValidateFn<T>>,
    pub on_axis_outcome: Option<OnVecEditAxisOutcome>,
    pub options: VecEditOptions,
}

impl<T> Vec4Edit<T>
where
    T: crate::primitives::drag_value_core::DragValueScalar + Default,
{
    pub fn new(
        x: Model<T>,
        y: Model<T>,
        z: Model<T>,
        w: Model<T>,
        format: NumericFormatFn<T>,
        parse: NumericParseFn<T>,
    ) -> Self {
        Self {
            x,
            y,
            z,
            w,
            reset_x: None,
            reset_y: None,
            reset_z: None,
            reset_w: None,
            format,
            parse,
            validate: None,
            on_axis_outcome: None,
            options: VecEditOptions::default(),
        }
    }

    /// Construct a vec editor from a shared editor authoring bundle.
    pub fn from_presentation(
        x: Model<T>,
        y: Model<T>,
        z: Model<T>,
        w: Model<T>,
        presentation: NumericPresentation<T>,
    ) -> Self {
        let mut edit = Self::new(x, y, z, w, presentation.format(), presentation.parse());
        edit.options.prefix = presentation.chrome_prefix().cloned();
        edit.options.suffix = presentation.chrome_suffix().cloned();
        edit
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn reset_x(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_x = reset;
        self
    }

    pub fn reset_y(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_y = reset;
        self
    }

    pub fn reset_z(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_z = reset;
        self
    }

    pub fn reset_w(mut self, reset: Option<AxisReset>) -> Self {
        self.reset_w = reset;
        self
    }

    pub fn options(mut self, options: VecEditOptions) -> Self {
        self.options = options;
        self
    }

    pub fn on_axis_outcome(mut self, on_axis_outcome: Option<OnVecEditAxisOutcome>) -> Self {
        self.on_axis_outcome = on_axis_outcome;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let x_id = self.x.id();
        let y_id = self.y.id();
        let z_id = self.z.id();
        let w_id = self.w.id();
        let model_ids = (x_id, y_id, z_id, w_id);

        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());

        let id_source = self.options.id_source.clone();
        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.vec4_edit", id_source, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.vec4_edit", callsite, model_ids), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3Edit;
    use crate::primitives::NumericPresentation;
    use fret_app::App;
    use std::sync::Arc;

    #[test]
    fn vec3_edit_from_presentation_adopts_format_parse_and_chrome_affixes() {
        let mut app = App::new();
        let x = app.models_mut().insert(1.0f64);
        let y = app.models_mut().insert(2.0f64);
        let z = app.models_mut().insert(3.0f64);
        let presentation = NumericPresentation::<f64>::fixed_decimals(2)
            .with_chrome_prefix("$")
            .with_chrome_suffix("ms");

        let edit = Vec3Edit::from_presentation(x, y, z, presentation);

        assert_eq!((edit.format)(1.25).as_ref(), "1.25");
        assert_eq!((edit.parse)("1.25"), Some(1.25));
        assert_eq!(edit.options.prefix, Some(Arc::from("$")));
        assert_eq!(edit.options.suffix, Some(Arc::from("ms")));
    }
}
