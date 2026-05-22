use std::collections::BTreeMap;

use delinea::engine::window::DataWindow;
use delinea::{BrushSelection2D, ChartEngine, LinkEvent, RowRange, SeriesId};

use crate::linking::{ChartLinkRouter, LinkAxisKey};
use crate::{TooltipFormatter, TooltipTextLine};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ChartCanvasOutputSnapshot {
    pub brush_selection_2d: Option<BrushSelection2D>,
    pub brush_x_row_ranges_by_series: BTreeMap<SeriesId, RowRange>,
    pub link_events: Vec<LinkEvent>,
    pub tooltip_lines: Vec<TooltipTextLine>,
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

pub fn chart_canvas_output_link_events_batch(
    output: &ChartCanvasOutput,
    drained_link_events: Vec<LinkEvent>,
) -> (u64, Vec<LinkEvent>) {
    if drained_link_events.is_empty() {
        (
            output.link_events_revision,
            output.snapshot.link_events.clone(),
        )
    } else {
        (
            output.link_events_revision.wrapping_add(1),
            drained_link_events,
        )
    }
}

pub fn chart_canvas_output_snapshot_for_engine(
    engine: &ChartEngine,
    router: &ChartLinkRouter,
    link_events: Vec<LinkEvent>,
    tooltip_formatter: &dyn TooltipFormatter,
) -> ChartCanvasOutputSnapshot {
    let mut domain_windows_by_key = BTreeMap::new();

    for (axis, st) in &engine.state().data_zoom_x {
        let Some(window) = st.window else {
            continue;
        };
        let Some(key) = router.axis_key(*axis) else {
            continue;
        };
        if router.axis_for_key(key) != Some(*axis) {
            continue;
        }
        domain_windows_by_key.insert(key, Some(window));
    }

    for (axis, window) in &engine.state().data_window_y {
        let Some(key) = router.axis_key(*axis) else {
            continue;
        };
        if router.axis_for_key(key) != Some(*axis) {
            continue;
        }
        domain_windows_by_key.insert(key, Some(*window));
    }

    let tooltip_lines = if let Some(axis_pointer) = engine.output().axis_pointer.as_ref() {
        tooltip_formatter.format_axis_pointer(engine, &engine.output().axis_windows, axis_pointer)
    } else {
        Vec::new()
    };

    ChartCanvasOutputSnapshot {
        brush_selection_2d: engine.state().brush_selection_2d,
        brush_x_row_ranges_by_series: engine.output().brush_x_row_ranges_by_series.clone(),
        link_events,
        tooltip_lines,
        domain_windows_by_key,
    }
}

pub fn update_chart_canvas_output(
    output: &mut ChartCanvasOutput,
    snapshot: ChartCanvasOutputSnapshot,
    link_events_revision: u64,
) -> bool {
    if output.snapshot == snapshot && output.link_events_revision == link_events_revision {
        return false;
    }

    output.revision = output.revision.wrapping_add(1);
    output.link_events_revision = link_events_revision;
    output.snapshot = snapshot;
    true
}
