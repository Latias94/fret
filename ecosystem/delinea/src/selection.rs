use crate::engine::window::{DataWindowX, DataWindowY};
use crate::ids::AxisId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BrushSelection2D {
    pub x_axis: AxisId,
    pub y_axis: AxisId,
    pub x: DataWindowX,
    pub y: DataWindowY,
}
