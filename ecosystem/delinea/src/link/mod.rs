use fret_core::Point;

use crate::ids::{LinkGroupId, SeriesId};
use crate::selection::BrushSelection2D;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BrushXExportPolicy {
    /// Default: derive `brush_x_row_ranges_by_series` only for series whose axis pair matches the
    /// brush selection (ADR 1145).
    #[default]
    AxisPairOnly,
    /// Opt-in: also derive `brush_x_row_ranges_by_series` for any visible series whose `(dataset,
    /// encode.x)` matches at least one brushed series. This allows cross-grid "X linking" without
    /// introducing sparse selections.
    SameDatasetXField,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LinkEvent {
    CursorMoved { point: Point },
    DomainWindowChanged { series: Option<SeriesId> },
    BrushSelectionChanged { selection: Option<BrushSelection2D> },
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LinkConfig {
    pub group: Option<LinkGroupId>,
    pub brush_x_export_policy: BrushXExportPolicy,
}
