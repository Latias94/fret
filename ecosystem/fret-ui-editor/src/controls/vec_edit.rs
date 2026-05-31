//! Vector editors (Vec2/Vec3/Vec4) built on top of `DragValue<T>`.
//!
//! These controls are policy-heavy and meant for inspector-like surfaces:
//! - compact axis labels (X/Y/Z/W)
//! - axis color tokens (`editor.axis.*`)
//! - shared numeric formatting/parsing policies

use std::panic::Location;
use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::controls::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::NumericPresentation;
use crate::primitives::input_group::derived_test_id;

mod axis;
mod layout;

use axis::axis_group;
pub use axis::{
    AxisReset, AxisResetOptions, OnAxisReset, OnVecEditAxisOutcome, VecEditAxis, VecEditAxisOutcome,
};
use layout::{derived_id_source, resolve_vec_edit_layout_plan};

#[derive(Debug, Clone)]
pub struct VecEditOptions {
    pub layout: LayoutStyle,
    pub prefix: Option<Arc<str>>,
    pub suffix: Option<Arc<str>>,
    /// Explicit identity source for internal element keys.
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    pub id_source: Option<Arc<str>>,
    pub variant: VecEditLayoutVariant,
    pub gap: Px,
    pub axis_gap: Px,
    pub auto_stack_below: Option<Px>,
    pub test_id: Option<Arc<str>>,
}

impl Default for VecEditOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            prefix: None,
            suffix: None,
            id_source: None,
            variant: VecEditLayoutVariant::Auto,
            gap: Px(6.0),
            axis_gap: Px(4.0),
            auto_stack_below: None,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VecEditLayoutVariant {
    Row,
    Column,
    /// Choose `Row` vs `Column` based on last frame bounds.
    ///
    /// This is a policy-only heuristic intended to avoid “tiny inputs” when a property grid is
    /// narrow (common in editor sidebars).
    #[default]
    Auto,
}

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

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let layout = resolve_vec_edit_layout_plan(cx, &self.options, 2);
        let grow = layout.grow;
        let direction = layout.direction;
        let x_color = layout.axis_colors.x;
        let y_color = layout.axis_colors.y;
        let x_id_source = derived_id_source(self.options.id_source.as_ref(), "x");
        let y_id_source = derived_id_source(self.options.id_source.as_ref(), "y");
        let x_test_id = derived_test_id(self.options.test_id.as_ref(), "x");
        let y_test_id = derived_test_id(self.options.test_id.as_ref(), "y");

        let mut el = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction,
                gap: SpacingLength::Px(self.options.gap),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: if direction == Axis::Horizontal {
                    CrossAlign::Center
                } else {
                    CrossAlign::Stretch
                },
                wrap: false,
            },
            move |cx| {
                vec![
                    axis_group(
                        cx,
                        VecEditAxis::X,
                        self.options.axis_gap,
                        self.reset_x.clone(),
                        grow,
                        x_id_source.clone(),
                        x_test_id.clone(),
                        Arc::from("X"),
                        x_color,
                        self.x.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::Y,
                        self.options.axis_gap,
                        self.reset_y.clone(),
                        grow,
                        y_id_source.clone(),
                        y_test_id.clone(),
                        Arc::from("Y"),
                        y_color,
                        self.y.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                ]
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
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

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let layout = resolve_vec_edit_layout_plan(cx, &self.options, 3);
        let grow = layout.grow;
        let direction = layout.direction;
        let x_color = layout.axis_colors.x;
        let y_color = layout.axis_colors.y;
        let z_color = layout.axis_colors.z;
        let x_id_source = derived_id_source(self.options.id_source.as_ref(), "x");
        let y_id_source = derived_id_source(self.options.id_source.as_ref(), "y");
        let z_id_source = derived_id_source(self.options.id_source.as_ref(), "z");
        let x_test_id = derived_test_id(self.options.test_id.as_ref(), "x");
        let y_test_id = derived_test_id(self.options.test_id.as_ref(), "y");
        let z_test_id = derived_test_id(self.options.test_id.as_ref(), "z");

        let mut el = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction,
                gap: SpacingLength::Px(self.options.gap),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: if direction == Axis::Horizontal {
                    CrossAlign::Center
                } else {
                    CrossAlign::Stretch
                },
                wrap: false,
            },
            move |cx| {
                vec![
                    axis_group(
                        cx,
                        VecEditAxis::X,
                        self.options.axis_gap,
                        self.reset_x.clone(),
                        grow,
                        x_id_source.clone(),
                        x_test_id.clone(),
                        Arc::from("X"),
                        x_color,
                        self.x.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::Y,
                        self.options.axis_gap,
                        self.reset_y.clone(),
                        grow,
                        y_id_source.clone(),
                        y_test_id.clone(),
                        Arc::from("Y"),
                        y_color,
                        self.y.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::Z,
                        self.options.axis_gap,
                        self.reset_z.clone(),
                        grow,
                        z_id_source.clone(),
                        z_test_id.clone(),
                        Arc::from("Z"),
                        z_color,
                        self.z.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                ]
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
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

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let layout = resolve_vec_edit_layout_plan(cx, &self.options, 4);
        let grow = layout.grow;
        let direction = layout.direction;
        let x_color = layout.axis_colors.x;
        let y_color = layout.axis_colors.y;
        let z_color = layout.axis_colors.z;
        let w_color = layout.axis_colors.w;
        let x_id_source = derived_id_source(self.options.id_source.as_ref(), "x");
        let y_id_source = derived_id_source(self.options.id_source.as_ref(), "y");
        let z_id_source = derived_id_source(self.options.id_source.as_ref(), "z");
        let w_id_source = derived_id_source(self.options.id_source.as_ref(), "w");
        let x_test_id = derived_test_id(self.options.test_id.as_ref(), "x");
        let y_test_id = derived_test_id(self.options.test_id.as_ref(), "y");
        let z_test_id = derived_test_id(self.options.test_id.as_ref(), "z");
        let w_test_id = derived_test_id(self.options.test_id.as_ref(), "w");

        let mut el = cx.flex(
            FlexProps {
                layout: self.options.layout,
                direction,
                gap: SpacingLength::Px(self.options.gap),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: if direction == Axis::Horizontal {
                    CrossAlign::Center
                } else {
                    CrossAlign::Stretch
                },
                wrap: false,
            },
            move |cx| {
                vec![
                    axis_group(
                        cx,
                        VecEditAxis::X,
                        self.options.axis_gap,
                        self.reset_x.clone(),
                        grow,
                        x_id_source.clone(),
                        x_test_id.clone(),
                        Arc::from("X"),
                        x_color,
                        self.x.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::Y,
                        self.options.axis_gap,
                        self.reset_y.clone(),
                        grow,
                        y_id_source.clone(),
                        y_test_id.clone(),
                        Arc::from("Y"),
                        y_color,
                        self.y.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::Z,
                        self.options.axis_gap,
                        self.reset_z.clone(),
                        grow,
                        z_id_source.clone(),
                        z_test_id.clone(),
                        Arc::from("Z"),
                        z_color,
                        self.z.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                    axis_group(
                        cx,
                        VecEditAxis::W,
                        self.options.axis_gap,
                        self.reset_w.clone(),
                        grow,
                        w_id_source.clone(),
                        w_test_id.clone(),
                        Arc::from("W"),
                        w_color,
                        self.w.clone(),
                        self.options.prefix.clone(),
                        self.options.suffix.clone(),
                        self.format.clone(),
                        self.parse.clone(),
                        self.validate.clone(),
                        self.on_axis_outcome.clone(),
                    ),
                ]
            },
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            el = el.test_id(test_id.clone());
        }
        el
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
