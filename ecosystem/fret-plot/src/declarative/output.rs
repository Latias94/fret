//! Declarative line-plot output and view snapshot owner.

use fret_core::{Point, Rect};
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::plot::view::sanitize_data_rect_scaled;
use crate::state::{PlotDragOutput, PlotOutput, PlotOutputSnapshot, PlotState};
use crate::style::LinePlotStyle;

use super::geometry::line_plot_inner_rect;
use super::model::PlotPanelModel;

pub(super) fn publish_line_plot_panel_output<H: UiHost>(
    app: &mut H,
    output: Option<&Model<PlotOutput>>,
    snapshot: PlotOutputSnapshot,
) -> bool {
    let Some(output) = output else {
        return false;
    };
    if output
        .read_ref(app, |state| state.snapshot == snapshot)
        .unwrap_or(false)
    {
        return false;
    }
    output
        .update(app, |state, _cx| {
            state.revision = state.revision.wrapping_add(1);
            state.snapshot = snapshot;
            true
        })
        .ok()
        .unwrap_or(false)
}

pub(super) fn line_plot_query_from_state<H: UiHost>(
    app: &H,
    state: Option<&Model<PlotState>>,
) -> Option<DataRect> {
    state.and_then(|state| state.read_ref(app, |state| state.query).ok().flatten())
}

pub(super) fn line_plot_output_snapshot(
    view_bounds: DataRect,
    cursor: Option<DataPoint>,
    query: Option<DataRect>,
) -> PlotOutputSnapshot {
    line_plot_output_snapshot_with_drag(view_bounds, cursor, query, None)
}

pub(super) fn line_plot_output_snapshot_with_drag(
    view_bounds: DataRect,
    cursor: Option<DataPoint>,
    query: Option<DataRect>,
    drag: Option<PlotDragOutput>,
) -> PlotOutputSnapshot {
    PlotOutputSnapshot {
        view_bounds,
        view_bounds_y2: None,
        view_bounds_y3: None,
        view_bounds_y4: None,
        cursor,
        hover: None,
        query,
        drag,
    }
}

pub(super) fn line_plot_pointer_output_snapshot(
    pointer: Point,
    bounds: Rect,
    _model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
    view_bounds: DataRect,
    query: Option<DataRect>,
) -> PlotOutputSnapshot {
    let plot = line_plot_inner_rect(bounds, style);
    let cursor = cursor_data_for_line_plot_pointer(pointer, plot, view_bounds, x_scale, y_scale);
    line_plot_output_snapshot(view_bounds, cursor, query)
}

pub(super) fn line_plot_view_bounds_from_state(
    model: &PlotPanelModel,
    state: Option<&PlotState>,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    if let Some(state) = state
        && !state.view_is_auto
        && let Some(view) = state.view_bounds
    {
        return sanitize_data_rect_scaled(view, x_scale, y_scale);
    }
    let data_bounds = sanitize_data_rect_scaled(model.data_bounds, x_scale, y_scale);
    if style.clamp_to_data_bounds {
        expand_line_plot_data_bounds(data_bounds, style.overscroll_fraction, x_scale, y_scale)
    } else {
        data_bounds
    }
}

pub(super) fn line_plot_current_view_bounds_for_event<H: UiHost>(
    app: &H,
    state: Option<&Model<PlotState>>,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    state
        .and_then(|state| {
            state
                .read_ref(app, |state| {
                    line_plot_view_bounds_from_state(model, Some(state), style, x_scale, y_scale)
                })
                .ok()
        })
        .unwrap_or_else(|| line_plot_view_bounds_from_state(model, None, style, x_scale, y_scale))
}

fn expand_line_plot_data_bounds(
    bounds: DataRect,
    fraction: f32,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    let fraction = fraction.max(0.0) as f64;
    if fraction <= 0.0 {
        return bounds;
    }
    let expand_axis = |scale: AxisScale, min: f64, max: f64| -> (f64, f64) {
        let Some(axis_min) = scale.to_axis(min) else {
            return (min, max);
        };
        let Some(axis_max) = scale.to_axis(max) else {
            return (min, max);
        };
        let span = axis_max - axis_min;
        if !span.is_finite() || span <= 0.0 {
            return (min, max);
        }
        let pad = span * fraction;
        let next_min = scale.from_axis(axis_min - pad).unwrap_or(min);
        let next_max = scale.from_axis(axis_max + pad).unwrap_or(max);
        (next_min, next_max)
    };
    let (x_min, x_max) = expand_axis(x_scale, bounds.x_min, bounds.x_max);
    let (y_min, y_max) = expand_axis(y_scale, bounds.y_min, bounds.y_max);
    sanitize_data_rect_scaled(
        DataRect {
            x_min,
            x_max,
            y_min,
            y_max,
        },
        x_scale,
        y_scale,
    )
}

fn cursor_data_for_line_plot_pointer(
    pointer: Point,
    plot: Rect,
    view_bounds: DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataPoint> {
    if !plot.contains(pointer) || plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }
    let transform = PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let data = transform.px_to_data(pointer);
    (data.x.is_finite() && data.y.is_finite()).then_some(data)
}
