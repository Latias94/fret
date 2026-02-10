use std::collections::BTreeMap;

use delinea::engine::window::DataWindow;
use delinea::{BrushSelection2D, LinkEvent, RowRange, SeriesId};

use crate::LinkAxisKey;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ChartCanvasOutputSnapshot {
    pub brush_selection_2d: Option<BrushSelection2D>,
    pub brush_x_row_ranges_by_series: BTreeMap<SeriesId, RowRange>,
    pub link_events: Vec<LinkEvent>,
    /// The current effective domain windows keyed in `LinkAxisKey` space.
    ///
    /// This is used by `LinkedChartGroup` to propagate domain window changes even when link
    /// events are not observed (for example, when a consumer polls outputs rather than draining
    /// per-step event queues).
    pub domain_windows_by_key: BTreeMap<LinkAxisKey, Option<DataWindow>>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ChartCanvasOutput {
    pub revision: u64,
    /// Monotonic-ish counter that advances when `snapshot.link_events` is updated with a new
    /// non-empty batch of link events.
    ///
    /// This exists because link events are inherently transient, and consumers (like
    /// `LinkedChartGroup`) need a stable way to detect that a new event batch was produced even
    /// if they observe the output model later.
    pub link_events_revision: u64,
    pub snapshot: ChartCanvasOutputSnapshot,
}
