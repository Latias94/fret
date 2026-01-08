use fret_core::Rect;

use crate::ids::{AxisId, ChartId, DataZoomId, DatasetId, FieldId, GridId, SeriesId};
use crate::scale::AxisScale;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChartSpec {
    pub id: ChartId,
    pub viewport: Option<Rect>,
    pub datasets: Vec<DatasetSpec>,
    pub grids: Vec<GridSpec>,
    pub axes: Vec<AxisSpec>,
    pub data_zoom_x: Vec<DataZoomXSpec>,
    pub axis_pointer: Option<AxisPointerSpec>,
    pub series: Vec<SeriesSpec>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetSpec {
    pub id: DatasetId,
    pub fields: Vec<FieldSpec>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldSpec {
    pub id: FieldId,
    /// Column index in the dataset table.
    pub column: usize,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GridSpec {
    pub id: GridId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisKind {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisPosition {
    Top,
    Bottom,
    Left,
    Right,
}

impl AxisPosition {
    pub fn default_for_kind(kind: AxisKind) -> Self {
        match kind {
            AxisKind::X => AxisPosition::Bottom,
            AxisKind::Y => AxisPosition::Left,
        }
    }

    pub fn is_compatible(self, kind: AxisKind) -> bool {
        match kind {
            AxisKind::X => matches!(self, AxisPosition::Top | AxisPosition::Bottom),
            AxisKind::Y => matches!(self, AxisPosition::Left | AxisPosition::Right),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FilterMode {
    #[default]
    Filter,
    None,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisRange {
    #[default]
    Auto,
    LockMin {
        min: f64,
    },
    LockMax {
        max: f64,
    },
    Fixed {
        min: f64,
        max: f64,
    },
}

impl AxisRange {
    pub fn clamp_non_degenerate(&mut self) {
        match self {
            AxisRange::Auto => {}
            AxisRange::LockMin { min } => {
                if !min.is_finite() {
                    *min = 0.0;
                }
            }
            AxisRange::LockMax { max } => {
                if !max.is_finite() {
                    *max = 1.0;
                }
            }
            AxisRange::Fixed { min, max } => {
                if !min.is_finite() || !max.is_finite() || *max <= *min {
                    *min = 0.0;
                    *max = 1.0;
                }
            }
        }
    }

    pub fn locked_min(&self) -> Option<f64> {
        match *self {
            AxisRange::Auto => None,
            AxisRange::LockMin { min } => Some(min),
            AxisRange::LockMax { .. } => None,
            AxisRange::Fixed { min, .. } => Some(min),
        }
    }

    pub fn locked_max(&self) -> Option<f64> {
        match *self {
            AxisRange::Auto => None,
            AxisRange::LockMin { .. } => None,
            AxisRange::LockMax { max } => Some(max),
            AxisRange::Fixed { max, .. } => Some(max),
        }
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, AxisRange::Fixed { .. })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisSpec {
    pub id: AxisId,
    /// Optional display name (used by tooltips and legends).
    pub name: Option<String>,
    pub kind: AxisKind,
    pub grid: GridId,
    /// Axis placement in the cartesian grid (presentation-only).
    /// Defaults: X=Bottom, Y=Left.
    pub position: Option<AxisPosition>,
    pub scale: AxisScale,
    /// When set, the axis is constrained in data space.
    /// In v1, `Fixed` fully overrides view windows; partial locks override only one bound.
    pub range: Option<AxisRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DataZoomXSpec {
    pub id: DataZoomId,
    pub axis: AxisId,
    pub filter_mode: FilterMode,
}

impl Default for DataZoomXSpec {
    fn default() -> Self {
        Self {
            id: DataZoomId::new(0),
            axis: AxisId::new(0),
            filter_mode: FilterMode::Filter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisPointerSpec {
    pub enabled: bool,
    pub trigger: AxisPointerTrigger,
    /// When true, crosshair snaps to the nearest hit point (P0: single series hit).
    pub snap: bool,
    /// Maximum distance (in pixels) to activate the pointer/tooltip.
    pub trigger_distance_px: f32,
    /// Minimum pointer movement (in pixels) to recompute hit testing.
    pub throttle_px: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisPointerTrigger {
    /// Similar to ECharts tooltip trigger="item": show details for a single series hit.
    #[default]
    Item,
    /// Similar to ECharts tooltip trigger="axis": show values for all visible series at the same X.
    Axis,
}

impl Default for AxisPointerSpec {
    fn default() -> Self {
        Self {
            enabled: true,
            trigger: AxisPointerTrigger::Item,
            snap: false,
            trigger_distance_px: 12.0,
            throttle_px: 0.75,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SeriesKind {
    Line,
    Area,
    /// A filled band between `encode.y` (low) and `encode.y2` (high).
    Band,
    Bar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesEncode {
    pub x: FieldId,
    pub y: FieldId,
    pub y2: Option<FieldId>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AreaBaseline {
    /// Close the area to the minimum of the current Y window.
    #[default]
    AxisMin,
    /// Close the area to Y=0 in data space (clamped to the current Y window).
    Zero,
    /// Close the area to a fixed Y value in data space (clamped to the current Y window).
    Value(f64),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesSpec {
    pub id: SeriesId,
    /// Optional display name (used by tooltips and legends).
    pub name: Option<String>,
    pub kind: SeriesKind,
    pub dataset: DatasetId,
    pub encode: SeriesEncode,
    pub x_axis: AxisId,
    pub y_axis: AxisId,
    /// Area baseline configuration (only used when `kind == Area`).
    pub area_baseline: Option<AreaBaseline>,
}
