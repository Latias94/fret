//! Vector editors (Vec2/Vec3/Vec4) built on top of `DragValue<T>`.
//!
//! These controls are policy-heavy and meant for inspector-like surfaces:
//! - compact axis labels (X/Y/Z/W)
//! - axis color tokens (`editor.axis.*`)
//! - shared numeric formatting/parsing policies

mod axis;
mod element;
mod layout;
mod model;
mod options;

pub use axis::{
    AxisReset, AxisResetOptions, OnAxisReset, OnVecEditAxisOutcome, VecEditAxis, VecEditAxisOutcome,
};
pub use model::{Vec2Edit, Vec3Edit, Vec4Edit};
pub use options::{VecEditLayoutVariant, VecEditOptions};
