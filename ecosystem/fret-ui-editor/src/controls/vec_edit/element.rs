use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::drag_value_core::DragValueScalar;

use super::axis::VecEditAxis;
use super::{Vec2Edit, Vec3Edit, Vec4Edit};

mod assembly;

use assembly::{VecEditElementArgs, VecEditElementAxis, vec_edit_element};

impl<T> Vec2Edit<T>
where
    T: DragValueScalar + Default,
{
    pub(super) fn into_element_keyed<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> AnyElement {
        vec_edit_element(
            cx,
            VecEditElementArgs {
                options: self.options,
                axes: vec![
                    VecEditElementAxis::new(VecEditAxis::X, "x", "X", self.x, self.reset_x),
                    VecEditElementAxis::new(VecEditAxis::Y, "y", "Y", self.y, self.reset_y),
                ],
                format: self.format,
                parse: self.parse,
                validate: self.validate,
                on_axis_outcome: self.on_axis_outcome,
            },
        )
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
        vec_edit_element(
            cx,
            VecEditElementArgs {
                options: self.options,
                axes: vec![
                    VecEditElementAxis::new(VecEditAxis::X, "x", "X", self.x, self.reset_x),
                    VecEditElementAxis::new(VecEditAxis::Y, "y", "Y", self.y, self.reset_y),
                    VecEditElementAxis::new(VecEditAxis::Z, "z", "Z", self.z, self.reset_z),
                ],
                format: self.format,
                parse: self.parse,
                validate: self.validate,
                on_axis_outcome: self.on_axis_outcome,
            },
        )
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
        vec_edit_element(
            cx,
            VecEditElementArgs {
                options: self.options,
                axes: vec![
                    VecEditElementAxis::new(VecEditAxis::X, "x", "X", self.x, self.reset_x),
                    VecEditElementAxis::new(VecEditAxis::Y, "y", "Y", self.y, self.reset_y),
                    VecEditElementAxis::new(VecEditAxis::Z, "z", "Z", self.z, self.reset_z),
                    VecEditElementAxis::new(VecEditAxis::W, "w", "W", self.w, self.reset_w),
                ],
                format: self.format,
                parse: self.parse,
                validate: self.validate,
                on_axis_outcome: self.on_axis_outcome,
            },
        )
    }
}
