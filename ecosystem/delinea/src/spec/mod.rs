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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AxisKind {
    X,
    Y,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisSpec {
    pub id: AxisId,
    pub kind: AxisKind,
    pub grid: GridId,
}

#[derive(Debug, Clone)]
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
    pub x_axis: AxisId,
    pub y_axis: AxisId,
}
