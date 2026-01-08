use fret_core::Point;

use crate::engine::window::{DataWindowX, DataWindowY};
use crate::engine::window_policy::FilterMode;
use crate::ids::{AxisId, DatasetId, LinkGroupId, SeriesId};
use crate::view::RowRange;

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
    SetDataWindowXFilterMode {
        axis: AxisId,
        mode: Option<FilterMode>,
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
    SetDatasetRowRange {
        dataset: DatasetId,
        range: Option<RowRange>,
    },
}
