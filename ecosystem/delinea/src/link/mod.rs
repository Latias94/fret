use crate::engine::window::DataWindow;
use crate::ids::{AxisId, GridId, LinkGroupId};
use crate::selection::BrushSelection2D;
use crate::spec::AxisKind;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BrushXExportPolicy {
    /// Default: derive `brush_x_row_ranges_by_series` only for series whose axis pair matches the
    /// brush selection (ADR 0206).
    #[default]
    AxisPairOnly,
    /// Opt-in: also derive `brush_x_row_ranges_by_series` for any visible series whose `(dataset,
    /// encode.x)` matches at least one brushed series. This allows cross-grid "X linking" without
    /// introducing sparse selections.
    SameDatasetXField,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AxisPointerAnchor {
    pub grid: Option<GridId>,
    pub axis_kind: AxisKind,
    pub axis: AxisId,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LinkEvent {
    AxisPointerChanged {
        anchor: Option<AxisPointerAnchor>,
    },
    DomainWindowChanged {
        axis: AxisId,
        window: Option<DataWindow>,
    },
    BrushSelectionChanged {
        selection: Option<BrushSelection2D>,
    },
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LinkConfig {
    pub group: Option<LinkGroupId>,
    pub brush_x_export_policy: BrushXExportPolicy,
}
