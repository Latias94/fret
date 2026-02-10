use crate::engine::window::{DataWindowX, DataWindowY};
use crate::ids::{AxisId, GridId};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BrushSelection2D {
    /// The grid that owns this brush selection (when known).
    ///
    /// In multi-grid charts, downstream consumers must not guess routing based on axis ids alone.
    pub grid: Option<GridId>,
    pub x_axis: AxisId,
    pub y_axis: AxisId,
    pub x: DataWindowX,
    pub y: DataWindowY,
}
