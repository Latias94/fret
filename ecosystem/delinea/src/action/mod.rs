use fret_core::Point;

use crate::engine::window::{DataWindowX, DataWindowY, WindowSpanAnchor};
use crate::ids::{AxisId, DatasetId, LinkGroupId, SeriesId, VisualMapId};
use crate::link::BrushXExportPolicy;
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
    SetBrushSelection2D {
        x_axis: AxisId,
        y_axis: AxisId,
        x: DataWindowX,
        y: DataWindowY,
    },
    ClearBrushSelection,
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
        base: DataWindowX,
        window: DataWindowX,
        anchor: WindowSpanAnchor,
    },
    SetDataWindowYFromZoom {
        axis: AxisId,
        base: DataWindowY,
        window: DataWindowY,
        anchor: WindowSpanAnchor,
    },
    SetDataWindowX {
        axis: AxisId,
        window: Option<DataWindowX>,
    },
    SetDataWindowY {
        axis: AxisId,
        window: Option<DataWindowY>,
    },
    /// Sets an axis window using an ECharts-style percent range (0..=100).
    ///
    /// This is an input/authoring surface: the engine will compute the corresponding value window
    /// from data extents, and (for multi-axis charts) the computation may be order-sensitive
    /// (ECharts `dataZoomProcessor` semantics).
    SetAxisWindowPercent {
        axis: AxisId,
        /// When `None`, clears percent mode for the axis.
        range: Option<(f64, f64)>,
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
    SetViewWindow2DFromZoom {
        x_axis: AxisId,
        y_axis: AxisId,
        base_x: DataWindowX,
        base_y: DataWindowY,
        x: Option<DataWindowX>,
        y: Option<DataWindowY>,
    },
    SetVisualMapRange {
        visual_map: VisualMapId,
        range: Option<(f64, f64)>,
    },
    SetVisualMapPieceMask {
        visual_map: VisualMapId,
        /// When `None`, all buckets are treated as selected.
        mask: Option<u64>,
    },
    SetSeriesVisible {
        series: SeriesId,
        visible: bool,
    },
    /// Batch version of `SetSeriesVisible` for interaction patterns that update multiple series
    /// at once (legend isolate, range selection, reset).
    ///
    /// The engine will apply all updates and bump revisions at most once.
    SetSeriesVisibility {
        updates: Vec<(SeriesId, bool)>,
    },
    SetLinkGroup {
        group: Option<LinkGroupId>,
    },
    SetLinkBrushXExportPolicy {
        policy: BrushXExportPolicy,
    },
    SetDatasetRowRange {
        dataset: DatasetId,
        range: Option<RowRange>,
    },
}
