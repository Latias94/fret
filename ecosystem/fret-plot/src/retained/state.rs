use std::collections::HashSet;

use crate::cartesian::{DataPoint, DataRect};
use crate::series::SeriesId;
use fret_core::geometry::{Point, Px};
use fret_core::scene::Color;

use super::models::YAxis;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotHoverOutput {
    pub series_id: SeriesId,
    pub data: DataPoint,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotDragPhase {
    Start,
    Update,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlotDragOutput {
    LineX {
        id: u64,
        x: f64,
        phase: PlotDragPhase,
    },
    LineY {
        id: u64,
        axis: YAxis,
        y: f64,
        phase: PlotDragPhase,
    },
    Point {
        id: u64,
        axis: YAxis,
        point: DataPoint,
        phase: PlotDragPhase,
    },
    Rect {
        id: u64,
        axis: YAxis,
        rect: DataRect,
        phase: PlotDragPhase,
    },
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
    pub drag: Option<PlotDragOutput>,
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
                drag: None,
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
    /// Plot overlays owned by the caller (e.g. reference lines).
    pub overlays: PlotOverlays,
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
            overlays: PlotOverlays::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfLineX {
    pub x: f64,
    pub color: Option<Color>,
    pub width: Px,
}

impl InfLineX {
    pub fn new(x: f64) -> Self {
        Self {
            x,
            color: None,
            width: Px(1.0),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn width(mut self, width: Px) -> Self {
        self.width = width;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfLineY {
    pub y: f64,
    pub axis: YAxis,
    pub color: Option<Color>,
    pub width: Px,
}

impl InfLineY {
    pub fn new(y: f64, axis: YAxis) -> Self {
        Self {
            y,
            axis,
            color: None,
            width: Px(1.0),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn width(mut self, width: Px) -> Self {
        self.width = width;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlotOverlays {
    pub inf_lines_x: Vec<InfLineX>,
    pub inf_lines_y: Vec<InfLineY>,
    pub drag_lines_x: Vec<DragLineX>,
    pub drag_lines_y: Vec<DragLineY>,
    pub drag_points: Vec<DragPoint>,
    pub drag_rects: Vec<DragRect>,
    pub tags_x: Vec<TagX>,
    pub tags_y: Vec<TagY>,
    pub text: Vec<PlotText>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DragLineX {
    pub id: u64,
    pub x: f64,
    pub label: Option<String>,
    pub show_value: bool,
    pub color: Option<Color>,
    pub width: Px,
}

impl DragLineX {
    pub fn new(id: u64, x: f64) -> Self {
        Self {
            id,
            x,
            label: None,
            show_value: true,
            color: None,
            width: Px(1.0),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn width(mut self, width: Px) -> Self {
        self.width = width;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DragLineY {
    pub id: u64,
    pub y: f64,
    pub axis: YAxis,
    pub label: Option<String>,
    pub show_value: bool,
    pub color: Option<Color>,
    pub width: Px,
}

impl DragLineY {
    pub fn new(id: u64, y: f64, axis: YAxis) -> Self {
        Self {
            id,
            y,
            axis,
            label: None,
            show_value: true,
            color: None,
            width: Px(1.0),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn width(mut self, width: Px) -> Self {
        self.width = width;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DragPoint {
    pub id: u64,
    pub point: DataPoint,
    pub axis: YAxis,
    pub label: Option<String>,
    pub show_value: bool,
    pub color: Option<Color>,
    pub radius: Px,
}

impl DragPoint {
    pub fn new(id: u64, point: DataPoint, axis: YAxis) -> Self {
        Self {
            id,
            point,
            axis,
            label: None,
            show_value: false,
            color: None,
            radius: Px(4.0),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn radius(mut self, radius: Px) -> Self {
        self.radius = radius;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DragRect {
    pub id: u64,
    pub rect: DataRect,
    pub axis: YAxis,
    pub label: Option<String>,
    pub show_value: bool,
    pub color: Option<Color>,
    pub border_width: Px,
    pub fill: Option<Color>,
}

impl DragRect {
    pub fn new(id: u64, rect: DataRect, axis: YAxis) -> Self {
        Self {
            id,
            rect,
            axis,
            label: None,
            show_value: false,
            color: None,
            border_width: Px(1.0),
            fill: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn border_width(mut self, border_width: Px) -> Self {
        self.border_width = border_width;
        self
    }

    pub fn fill(mut self, fill: Color) -> Self {
        self.fill = Some(fill);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagX {
    pub x: f64,
    pub label: Option<String>,
    pub show_value: bool,
    pub color: Option<Color>,
}

impl TagX {
    pub fn new(x: f64) -> Self {
        Self {
            x,
            label: None,
            show_value: true,
            color: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagY {
    pub y: f64,
    pub axis: YAxis,
    pub label: Option<String>,
    pub show_value: bool,
    pub color: Option<Color>,
}

impl TagY {
    pub fn new(y: f64, axis: YAxis) -> Self {
        Self {
            y,
            axis,
            label: None,
            show_value: true,
            color: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlotText {
    pub x: f64,
    pub y: f64,
    pub axis: YAxis,
    pub text: String,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub padding: Px,
    pub corner_radius: Px,
    pub offset: Point,
}

impl PlotText {
    pub fn new(x: f64, y: f64, axis: YAxis, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            axis,
            text: text.into(),
            color: None,
            background: None,
            border: None,
            padding: Px(0.0),
            corner_radius: Px(0.0),
            offset: Point::new(Px(0.0), Px(0.0)),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn background(mut self, background: Color) -> Self {
        self.background = Some(background);
        self
    }

    pub fn border(mut self, border: Color) -> Self {
        self.border = Some(border);
        self
    }

    pub fn padding(mut self, padding: Px) -> Self {
        self.padding = padding;
        self
    }

    pub fn corner_radius(mut self, radius: Px) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn offset(mut self, offset: Point) -> Self {
        self.offset = offset;
        self
    }
}
