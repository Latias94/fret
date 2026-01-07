use fret_core::Rect;

use crate::ids::{AxisId, ChartId, DatasetId, GridId, SeriesId};

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
    pub series: Vec<SeriesSpec>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DatasetSpec {
    pub id: DatasetId,
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
    pub kind: AxisKind,
    pub grid: GridId,
    /// When set, the axis is constrained in data space.
    /// In v1, `Fixed` fully overrides view windows; partial locks override only one bound.
    pub range: Option<AxisRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SeriesKind {
    Line,
    Bar,
    Scatter,
    Area,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SeriesSpec {
    pub id: SeriesId,
    pub kind: SeriesKind,
    pub dataset: DatasetId,
    /// Column index for x values (temporary, P0).
    pub x_col: usize,
    /// Column index for y values (temporary, P0).
    pub y_col: usize,
    pub x_axis: AxisId,
    pub y_axis: AxisId,
}
