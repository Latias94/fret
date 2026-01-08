use fret_core::Point;

use crate::engine::window::{DataWindowX, DataWindowY};
use crate::ids::{AxisId, DatasetId, LinkGroupId, SeriesId};
use crate::spec::FilterMode;
use crate::transform::RowRange;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Action {
    HoverAt {
        point: Point,
    },
    ToggleAxisPanLock {
        axis: AxisId,
    },
    ToggleAxisZoomLock {
        axis: AxisId,
    },
    PanDataWindowXFromBase {
        axis: AxisId,
        base: DataWindowX,
        delta_px: f32,
        viewport_span_px: f32,
    },
    PanDataWindowYFromBase {
        axis: AxisId,
        base: DataWindowY,
        delta_px: f32,
        viewport_span_px: f32,
    },
    ZoomDataWindowXFromBase {
        axis: AxisId,
        base: DataWindowX,
        center_px: f32,
        log2_scale: f32,
        viewport_span_px: f32,
    },
    ZoomDataWindowYFromBase {
        axis: AxisId,
        base: DataWindowY,
        center_px: f32,
        log2_scale: f32,
        viewport_span_px: f32,
    },
    SetDataWindowXFromZoom {
        axis: AxisId,
        window: DataWindowX,
    },
    SetDataWindowYFromZoom {
        axis: AxisId,
        window: DataWindowY,
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
