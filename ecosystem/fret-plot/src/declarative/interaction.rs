//! Declarative line-plot interaction and event routing owner.

use fret_core::{Event, MouseButton, Point, Rect};
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::cartesian::{AxisScale, DataRect};
use crate::series::SeriesId;
use crate::state::{PlotOutputSnapshot, PlotState};
use crate::style::LinePlotStyle;

use super::geometry::line_plot_inner_rect;
use super::legend::{LinePlotLegendHit, line_plot_legend_hit};
use super::model::PlotPanelModel;
use super::output::line_plot_pointer_output_snapshot;

mod box_zoom;
mod draggable;
mod pan;
mod query;
mod wheel;

pub(super) use box_zoom::{LinePlotBoxZoomSession, handle_line_plot_box_zoom_event};
pub(super) use draggable::{LinePlotDragSession, handle_line_plot_draggable_overlay_event};
pub(super) use pan::{LinePlotPanSession, handle_line_plot_pan_event};
pub(super) use query::{
    LinePlotQueryDragSession, handle_line_plot_query_drag_event,
    line_plot_query_rect_from_plot_points_raw,
};
pub(super) use wheel::handle_line_plot_wheel_zoom_event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::declarative) enum LinePlotSelectionKind {
    Query,
    BoxZoom,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::declarative) struct LinePlotSelectionOverlay {
    pub(in crate::declarative) start: Point,
    pub(in crate::declarative) current: Point,
    pub(in crate::declarative) kind: LinePlotSelectionKind,
}

pub(super) fn handle_line_plot_legend_pointer_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
) -> bool {
    let Event::Pointer(fret_core::PointerEvent::Down {
        position,
        button: MouseButton::Left,
        modifiers,
        ..
    }) = event
    else {
        return false;
    };

    let plot = line_plot_inner_rect(bounds, style);
    let Some((series_id, hit)) = line_plot_legend_hit(model, plot, *position) else {
        return false;
    };

    state
        .update(app, |state, _cx| match hit {
            _ if modifiers.shift => {
                let ids: Vec<SeriesId> = model.series.iter().map(|series| series.id).collect();
                let visible_count = ids
                    .iter()
                    .filter(|series_id| !state.hidden_series.contains(series_id))
                    .count();
                let is_solo = visible_count == 1 && !state.hidden_series.contains(&series_id);
                if is_solo {
                    state.hidden_series.clear();
                } else {
                    state.hidden_series = ids.into_iter().filter(|id| *id != series_id).collect();
                    state.hidden_series.remove(&series_id);
                }
                true
            }
            LinePlotLegendHit::Swatch => {
                let total = model.series.len();
                let hidden_count = model
                    .series
                    .iter()
                    .filter(|series| state.hidden_series.contains(&series.id))
                    .count();
                let visible_count = total.saturating_sub(hidden_count);
                if state.hidden_series.contains(&series_id) {
                    state.hidden_series.remove(&series_id);
                    state.pinned_series = state.pinned_series.filter(|id| *id != series_id);
                    true
                } else if visible_count <= 1 {
                    false
                } else {
                    state.hidden_series.insert(series_id);
                    state.pinned_series = state.pinned_series.filter(|id| *id != series_id);
                    true
                }
            }
            LinePlotLegendHit::Label => {
                if state.pinned_series == Some(series_id) {
                    state.pinned_series = None;
                } else {
                    state.pinned_series = Some(series_id);
                    state.hidden_series.remove(&series_id);
                }
                true
            }
        })
        .ok()
        .unwrap_or(false)
}

pub(super) fn line_plot_panel_event_snapshot(
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
    view_bounds: DataRect,
    query: Option<DataRect>,
) -> Option<PlotOutputSnapshot> {
    let Event::Pointer(fret_core::PointerEvent::Move { position, .. }) = event else {
        return None;
    };
    Some(line_plot_pointer_output_snapshot(
        *position,
        bounds,
        model,
        style,
        x_scale,
        y_scale,
        view_bounds,
        query,
    ))
}

fn line_plot_mouse_buttons_contains(buttons: fret_core::MouseButtons, button: MouseButton) -> bool {
    match button {
        MouseButton::Left => buttons.left,
        MouseButton::Right => buttons.right,
        MouseButton::Middle => buttons.middle,
        MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
    }
}

pub(super) fn line_plot_legend_hover_from_event(
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
) -> Option<Option<SeriesId>> {
    let Event::Pointer(fret_core::PointerEvent::Move { position, .. }) = event else {
        return None;
    };

    let plot = line_plot_inner_rect(bounds, style);
    Some(
        line_plot_legend_hit(model, plot, *position)
            .map(|(series_id, _hit)| series_id)
            .filter(|series_id| model.series.iter().any(|series| series.id == *series_id)),
    )
}
