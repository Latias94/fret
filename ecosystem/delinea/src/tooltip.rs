#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::ids::{AxisId, SeriesId};
use crate::spec::AxisKind;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TooltipOutput {
    Item(TooltipItemOutput),
    Axis(TooltipAxisOutput),
}

impl TooltipOutput {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Item(_) => false,
            Self::Axis(axis) => axis.series.is_empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TooltipItemOutput {
    pub series: SeriesId,
    pub data_index: u32,
    pub x_axis: AxisId,
    pub y_axis: AxisId,
    pub x_value: f64,
    pub y_value: f64,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TooltipAxisOutput {
    pub axis: AxisId,
    pub axis_kind: AxisKind,
    pub axis_value: f64,
    pub series: Vec<TooltipSeriesEntry>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TooltipSeriesEntry {
    pub series: SeriesId,
    pub value_axis: AxisId,
    pub value: TooltipSeriesValue,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TooltipSeriesValue {
    Missing,
    Scalar(f64),
    Range { min: f64, max: f64 },
}
