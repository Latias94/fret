use std::collections::BTreeMap;

use delinea::{BrushSelection2D, LinkEvent, RowRange, SeriesId};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ChartCanvasOutputSnapshot {
    pub brush_selection_2d: Option<BrushSelection2D>,
    pub brush_x_row_ranges_by_series: BTreeMap<SeriesId, RowRange>,
    pub link_events: Vec<LinkEvent>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ChartCanvasOutput {
    pub revision: u64,
    pub snapshot: ChartCanvasOutputSnapshot,
}
