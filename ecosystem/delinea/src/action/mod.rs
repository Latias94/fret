use fret_core::Point;

use crate::engine::window::{DataWindowX, DataWindowY};
use crate::ids::{AxisId, LinkGroupId, SeriesId};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Action {
    HoverAt {
        point: Point,
    },
    SetDataWindowX {
        axis: AxisId,
        window: Option<DataWindowX>,
    },
    SetDataWindowY {
        axis: AxisId,
        window: Option<DataWindowY>,
    },
    SetViewWindow2D {
        x_axis: AxisId,
        y_axis: AxisId,
        x: Option<DataWindowX>,
        y: Option<DataWindowY>,
    },
    SetSeriesVisible {
        series: SeriesId,
        visible: bool,
    },
    SetLinkGroup {
        group: Option<LinkGroupId>,
    },
}
