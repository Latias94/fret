use std::collections::HashSet;

use crate::cartesian::{DataPoint, DataRect};
use crate::series::SeriesId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotHoverOutput {
    pub series_id: SeriesId,
    pub data: DataPoint,
    pub value: Option<f64>,
}

/// A caller-owned output snapshot for plot interaction state.
///
/// This is intended for building higher-level behaviors such as linked plots, inspectors, and
/// multi-pane coordination without requiring direct access to the plot internals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotOutputSnapshot {
    pub view_bounds: DataRect,
    pub view_bounds_y2: Option<DataRect>,
    pub view_bounds_y3: Option<DataRect>,
    pub view_bounds_y4: Option<DataRect>,
    pub cursor: Option<DataPoint>,
    pub hover: Option<PlotHoverOutput>,
    pub query: Option<DataRect>,
}

/// Plot output state written by the plot widget.
///
/// Callers are expected to treat this as write-only from the widget side (i.e. do not mutate it
/// directly from application code). Use it as an observation point for interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotOutput {
    pub revision: u64,
    pub snapshot: PlotOutputSnapshot,
}

impl Default for PlotOutput {
    fn default() -> Self {
        Self {
            revision: 0,
            snapshot: PlotOutputSnapshot {
                view_bounds: DataRect {
                    x_min: 0.0,
                    x_max: 1.0,
                    y_min: 0.0,
                    y_max: 1.0,
                },
                view_bounds_y2: None,
                view_bounds_y3: None,
                view_bounds_y4: None,
                cursor: None,
                hover: None,
                query: None,
            },
        }
    }
}

/// Persistent plot interaction state owned by the caller (optional).
///
/// This mirrors common plotting libraries (e.g. ImPlot / egui_plot) where plot view state and user
/// preferences (hidden series, pinned series) outlive a single render pass.
///
/// By default, `PlotCanvas` owns an internal `PlotState`. Callers can provide a `Model<PlotState>`
/// to store this state externally (so it can be persisted, shared, or controlled programmatically).
#[derive(Debug, Clone)]
pub struct PlotState {
    /// Current view bounds in data space when `view_is_auto == false`.
    pub view_bounds: Option<DataRect>,
    /// If true, the plot view is derived from `data_bounds` each frame (auto-fit).
    pub view_is_auto: bool,
    /// Current view bounds for the right Y axis (if enabled) when `view_y2_is_auto == false`.
    pub view_bounds_y2: Option<DataRect>,
    /// If true, the right Y axis view is derived from `data_bounds` each frame (auto-fit).
    pub view_y2_is_auto: bool,
    /// Current view bounds for the 3rd Y axis (if enabled) when `view_y3_is_auto == false`.
    pub view_bounds_y3: Option<DataRect>,
    /// If true, the 3rd Y axis view is derived from `data_bounds` each frame (auto-fit).
    pub view_y3_is_auto: bool,
    /// Current view bounds for the 4th Y axis (if enabled) when `view_y4_is_auto == false`.
    pub view_bounds_y4: Option<DataRect>,
    /// If true, the 4th Y axis view is derived from `data_bounds` each frame (auto-fit).
    pub view_y4_is_auto: bool,
    /// An externally linked cursor position in data space.
    ///
    /// This is typically written by a plot coordinator (e.g. `LinkedPlotGroup`) so that other plots
    /// can render a synchronized cursor without requiring pointer hover in each plot.
    pub linked_cursor_x: Option<f64>,
    /// User-controlled series visibility.
    pub hidden_series: HashSet<SeriesId>,
    /// Optional pinned series ID for emphasis and tooltip pinning.
    pub pinned_series: Option<SeriesId>,
    /// Optional user query selection in data space.
    pub query: Option<DataRect>,
}

impl Default for PlotState {
    fn default() -> Self {
        Self {
            view_bounds: None,
            view_is_auto: true,
            view_bounds_y2: None,
            view_y2_is_auto: true,
            view_bounds_y3: None,
            view_y3_is_auto: true,
            view_bounds_y4: None,
            view_y4_is_auto: true,
            linked_cursor_x: None,
            hidden_series: HashSet::new(),
            pinned_series: None,
            query: None,
        }
    }
}
