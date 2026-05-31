use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, SpacingLength};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::derived_test_id;

use super::axis::{VecEditAxis, axis_group};
use super::layout::{derived_id_source, resolve_vec_edit_layout_plan};
use super::{Vec2Edit, Vec3Edit, Vec4Edit};

impl<T> Vec2Edit<T>
where
    T: DragValueScalar + Default,
{
    pub(super) fn into_element_keyed<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
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

impl<T> Vec3Edit<T>
where
    T: DragValueScalar + Default,
{
    pub(super) fn into_element_keyed<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
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

impl<T> Vec4Edit<T>
where
    T: DragValueScalar + Default,
{
    pub(super) fn into_element_keyed<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
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
