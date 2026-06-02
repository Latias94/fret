//! Declarative line-plot interaction and event routing owner.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use fret_core::{Event, MouseButton, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::input_map::{ModifierKey, ModifiersMask, PlotInputMap};
use crate::models::YAxis;
use crate::plot::view::{
    clamp_view_to_data_scaled, clamp_zoom_factors, data_rect_from_plot_points_scaled,
    local_from_absolute, sanitize_data_rect_scaled, zoom_view_at_px_scaled,
};
use crate::series::SeriesId;
use crate::state::{PlotDragOutput, PlotDragPhase, PlotOutputSnapshot, PlotState};
use crate::style::LinePlotStyle;

use super::legend::{LinePlotLegendHit, line_plot_legend_hit};
use super::{
    line_plot_current_view_bounds_for_event, line_plot_inner_rect,
    line_plot_pointer_output_snapshot, line_plot_view_bounds_for_y_axis, model::PlotPanelModel,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct LinePlotPanSession {
    last_position: Point,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LinePlotBoxZoomSession {
    pub(super) start: Point,
    pub(super) current: Point,
    button: MouseButton,
    required_mods: ModifiersMask,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LinePlotQueryDragSession {
    pub(super) start: Point,
    pub(super) current: Point,
    button: MouseButton,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum LinePlotDragSession {
    LineX {
        id: u64,
        button: MouseButton,
        offset_x: f64,
        current_x: f64,
    },
    LineY {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        offset_y: f64,
        current_y: f64,
    },
    Point {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        offset: DataPoint,
        current: DataPoint,
    },
    Rect {
        id: u64,
        axis: YAxis,
        button: MouseButton,
        handle: LinePlotDragRectHandle,
        offset: DataPoint,
        start: DataRect,
        current: DataRect,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LinePlotDragRectHandle {
    Inside,
    Left,
    Right,
    Top,
    Bottom,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinePlotWheelRegion {
    Plot,
    XAxis,
    YAxis,
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

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_line_plot_draggable_overlay_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    drag_session: &Rc<RefCell<Option<LinePlotDragSession>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<PlotDragOutput> {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return None;
    }

    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            ..
        }) if plot.contains(*position)
            && PlotInputMap::default().pan.matches(*button, *modifiers)
            && line_plot_legend_hit(model, plot, *position).is_none() =>
        {
            let view_bounds = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let overlays = state
                .read_ref(app, |state| state.overlays.clone())
                .unwrap_or_default();
            let local = local_from_absolute(plot.origin, *position);
            let threshold = style.hover_threshold.0.max(1.0);
            let mut best: Option<(f32, LinePlotDragSession)> = None;

            for point in &overlays.drag_points {
                if !point.point.x.is_finite() || !point.point.y.is_finite() {
                    continue;
                }
                let Some(transform) = line_plot_transform_for_y_axis(
                    plot,
                    view_bounds,
                    model,
                    point.axis,
                    x_scale,
                    y_scale,
                ) else {
                    continue;
                };
                let p_px = transform.data_to_px(point.point);
                if !p_px.x.0.is_finite() || !p_px.y.0.is_finite() {
                    continue;
                }
                let hit_r = point.radius.0.max(threshold);
                let dx = local.x.0 - p_px.x.0;
                let dy = local.y.0 - p_px.y.0;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > hit_r {
                    continue;
                }
                let data = transform.px_to_data(local);
                if !data.x.is_finite() || !data.y.is_finite() {
                    continue;
                };
                let candidate = LinePlotDragSession::Point {
                    id: point.id,
                    axis: point.axis,
                    button: *button,
                    offset: DataPoint {
                        x: data.x - point.point.x,
                        y: data.y - point.point.y,
                    },
                    current: point.point,
                };
                if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                    best = Some((dist, candidate));
                }
            }

            if best.is_none() {
                let transform_x =
                    line_plot_transform_for_x_axis(plot, view_bounds, x_scale, y_scale);
                for line in &overlays.drag_lines_x {
                    if !line.x.is_finite() {
                        continue;
                    }
                    let Some(x_px) = transform_x.data_x_to_px(line.x) else {
                        continue;
                    };
                    let dist = (local.x.0 - x_px.0).abs();
                    if dist > threshold {
                        continue;
                    }
                    let data = transform_x.px_to_data(local);
                    if !data.x.is_finite() {
                        continue;
                    }
                    let candidate = LinePlotDragSession::LineX {
                        id: line.id,
                        button: *button,
                        offset_x: data.x - line.x,
                        current_x: line.x,
                    };
                    if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                        best = Some((dist, candidate));
                    }
                }

                for line in &overlays.drag_lines_y {
                    if !line.y.is_finite() {
                        continue;
                    }
                    let Some(transform) = line_plot_transform_for_y_axis(
                        plot,
                        view_bounds,
                        model,
                        line.axis,
                        x_scale,
                        y_scale,
                    ) else {
                        continue;
                    };
                    let Some(y_px) = transform.data_y_to_px(line.y) else {
                        continue;
                    };
                    let dist = (local.y.0 - y_px.0).abs();
                    if dist > threshold {
                        continue;
                    }
                    let data = transform.px_to_data(local);
                    if !data.y.is_finite() {
                        continue;
                    }
                    let candidate = LinePlotDragSession::LineY {
                        id: line.id,
                        axis: line.axis,
                        button: *button,
                        offset_y: data.y - line.y,
                        current_y: line.y,
                    };
                    if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                        best = Some((dist, candidate));
                    }
                }

                for rect in &overlays.drag_rects {
                    let Some(transform) = line_plot_transform_for_y_axis(
                        plot,
                        view_bounds,
                        model,
                        rect.axis,
                        x_scale,
                        y_scale,
                    ) else {
                        continue;
                    };
                    let a = transform.data_to_px(DataPoint {
                        x: rect.rect.x_min,
                        y: rect.rect.y_min,
                    });
                    let b = transform.data_to_px(DataPoint {
                        x: rect.rect.x_max,
                        y: rect.rect.y_max,
                    });
                    if !a.x.0.is_finite()
                        || !a.y.0.is_finite()
                        || !b.x.0.is_finite()
                        || !b.y.0.is_finite()
                    {
                        continue;
                    }

                    let left = a.x.0.min(b.x.0);
                    let right = a.x.0.max(b.x.0);
                    let top = a.y.0.min(b.y.0);
                    let bottom = a.y.0.max(b.y.0);
                    let inside = local.x.0 >= left
                        && local.x.0 <= right
                        && local.y.0 >= top
                        && local.y.0 <= bottom;
                    if !inside {
                        continue;
                    }

                    let dist_left = (local.x.0 - left).abs();
                    let dist_right = (local.x.0 - right).abs();
                    let dist_top = (local.y.0 - top).abs();
                    let dist_bottom = (local.y.0 - bottom).abs();
                    let mut handle = LinePlotDragRectHandle::Inside;
                    let mut dist = 0.0f32;
                    let mut set_handle = |d: f32, h: LinePlotDragRectHandle| {
                        if d <= threshold && (handle == LinePlotDragRectHandle::Inside || d < dist)
                        {
                            handle = h;
                            dist = d;
                        }
                    };
                    set_handle(dist_left, LinePlotDragRectHandle::Left);
                    set_handle(dist_right, LinePlotDragRectHandle::Right);
                    set_handle(dist_top, LinePlotDragRectHandle::Top);
                    set_handle(dist_bottom, LinePlotDragRectHandle::Bottom);

                    let data = transform.px_to_data(local);
                    if !data.x.is_finite() || !data.y.is_finite() {
                        continue;
                    }
                    let offset = match handle {
                        LinePlotDragRectHandle::Inside => DataPoint {
                            x: data.x - rect.rect.x_min,
                            y: data.y - rect.rect.y_min,
                        },
                        LinePlotDragRectHandle::Left => DataPoint {
                            x: data.x - rect.rect.x_min,
                            y: 0.0,
                        },
                        LinePlotDragRectHandle::Right => DataPoint {
                            x: data.x - rect.rect.x_max,
                            y: 0.0,
                        },
                        LinePlotDragRectHandle::Top => DataPoint {
                            x: 0.0,
                            y: data.y - rect.rect.y_max,
                        },
                        LinePlotDragRectHandle::Bottom => DataPoint {
                            x: 0.0,
                            y: data.y - rect.rect.y_min,
                        },
                    };
                    let candidate = LinePlotDragSession::Rect {
                        id: rect.id,
                        axis: rect.axis,
                        button: *button,
                        handle,
                        offset,
                        start: rect.rect,
                        current: rect.rect,
                    };
                    if best.as_ref().is_none_or(|(best_dist, _)| dist < *best_dist) {
                        best = Some((dist, candidate));
                    }
                }
            }

            let (_, session) = best?;
            *drag_session.borrow_mut() = Some(session);
            Some(line_plot_drag_output(session, PlotDragPhase::Start))
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) => {
            let Some(mut session) = *drag_session.borrow() else {
                return None;
            };
            let button = match session {
                LinePlotDragSession::LineX { button, .. } => button,
                LinePlotDragSession::LineY { button, .. } => button,
                LinePlotDragSession::Point { button, .. } => button,
                LinePlotDragSession::Rect { button, .. } => button,
            };
            let phase = if line_plot_mouse_buttons_contains(*buttons, button) {
                PlotDragPhase::Update
            } else {
                drag_session.borrow_mut().take();
                PlotDragPhase::End
            };
            line_plot_update_drag_session_at_position(
                &mut session,
                *position,
                plot,
                line_plot_current_view_bounds_for_event(
                    app,
                    Some(state),
                    model,
                    style,
                    x_scale,
                    y_scale,
                ),
                model,
                x_scale,
                y_scale,
            );
            if phase != PlotDragPhase::End {
                *drag_session.borrow_mut() = Some(session);
            }
            Some(line_plot_drag_output(session, phase))
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position, button, ..
        }) => {
            let mut session = drag_session.borrow_mut().take()?;
            let session_button = match session {
                LinePlotDragSession::LineX { button, .. } => button,
                LinePlotDragSession::LineY { button, .. } => button,
                LinePlotDragSession::Point { button, .. } => button,
                LinePlotDragSession::Rect { button, .. } => button,
            };
            if session_button != *button {
                *drag_session.borrow_mut() = Some(session);
                return None;
            }
            line_plot_update_drag_session_at_position(
                &mut session,
                *position,
                plot,
                line_plot_current_view_bounds_for_event(
                    app,
                    Some(state),
                    model,
                    style,
                    x_scale,
                    y_scale,
                ),
                model,
                x_scale,
                y_scale,
            );
            Some(line_plot_drag_output(session, PlotDragPhase::End))
        }
        _ => None,
    }
}

fn line_plot_update_drag_session_at_position(
    session: &mut LinePlotDragSession,
    position: Point,
    plot: Rect,
    view_bounds: DataRect,
    model: &PlotPanelModel,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    match session {
        LinePlotDragSession::LineX {
            offset_x,
            current_x,
            ..
        } => {
            let transform = line_plot_transform_for_x_axis(plot, view_bounds, x_scale, y_scale);
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if data.x.is_finite() {
                *current_x = data.x - *offset_x;
            }
        }
        LinePlotDragSession::LineY {
            axis,
            offset_y,
            current_y,
            ..
        } => {
            let Some(transform) =
                line_plot_transform_for_y_axis(plot, view_bounds, model, *axis, x_scale, y_scale)
            else {
                return;
            };
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if data.y.is_finite() {
                *current_y = data.y - *offset_y;
            }
        }
        LinePlotDragSession::Point {
            axis,
            offset,
            current,
            ..
        } => {
            let Some(transform) =
                line_plot_transform_for_y_axis(plot, view_bounds, model, *axis, x_scale, y_scale)
            else {
                return;
            };
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if data.x.is_finite() && data.y.is_finite() {
                *current = DataPoint {
                    x: data.x - offset.x,
                    y: data.y - offset.y,
                };
            }
        }
        LinePlotDragSession::Rect {
            axis,
            handle,
            offset,
            start,
            current,
            ..
        } => {
            let Some(transform) =
                line_plot_transform_for_y_axis(plot, view_bounds, model, *axis, x_scale, y_scale)
            else {
                return;
            };
            let local = local_from_absolute(plot.origin, position);
            let data = transform.px_to_data(local);
            if !data.x.is_finite() || !data.y.is_finite() {
                return;
            }

            let mut next = *current;
            match handle {
                LinePlotDragRectHandle::Inside => {
                    let w = start.width();
                    let h = start.height();
                    next.x_min = data.x - offset.x;
                    next.x_max = next.x_min + w;
                    next.y_min = data.y - offset.y;
                    next.y_max = next.y_min + h;
                }
                LinePlotDragRectHandle::Left => {
                    next.x_min = data.x - offset.x;
                    if next.x_min > next.x_max {
                        next.x_min = next.x_max;
                    }
                }
                LinePlotDragRectHandle::Right => {
                    next.x_max = data.x - offset.x;
                    if next.x_max < next.x_min {
                        next.x_max = next.x_min;
                    }
                }
                LinePlotDragRectHandle::Top => {
                    next.y_max = data.y - offset.y;
                    if next.y_max < next.y_min {
                        next.y_max = next.y_min;
                    }
                }
                LinePlotDragRectHandle::Bottom => {
                    next.y_min = data.y - offset.y;
                    if next.y_min > next.y_max {
                        next.y_min = next.y_max;
                    }
                }
            }
            *current = next;
        }
    }
}

fn line_plot_drag_output(session: LinePlotDragSession, phase: PlotDragPhase) -> PlotDragOutput {
    match session {
        LinePlotDragSession::LineX { id, current_x, .. } => PlotDragOutput::LineX {
            id,
            x: current_x,
            phase,
        },
        LinePlotDragSession::LineY {
            id,
            axis,
            current_y,
            ..
        } => PlotDragOutput::LineY {
            id,
            axis,
            y: current_y,
            phase,
        },
        LinePlotDragSession::Point {
            id, axis, current, ..
        } => PlotDragOutput::Point {
            id,
            axis,
            point: current,
            phase,
        },
        LinePlotDragSession::Rect {
            id, axis, current, ..
        } => PlotDragOutput::Rect {
            id,
            axis,
            rect: current,
            phase,
        },
    }
}

fn line_plot_transform_for_x_axis(
    plot: Rect,
    view_bounds: DataRect,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> PlotTransform {
    PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size),
        data: view_bounds,
        x_scale,
        y_scale,
    }
}

fn line_plot_transform_for_y_axis(
    plot: Rect,
    primary_view_bounds: DataRect,
    model: &PlotPanelModel,
    axis: YAxis,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<PlotTransform> {
    let data = match axis {
        YAxis::Left => primary_view_bounds,
        YAxis::Right => {
            line_plot_view_bounds_for_y_axis(primary_view_bounds, model.data_bounds_y2?)
        }
        YAxis::Right2 => {
            line_plot_view_bounds_for_y_axis(primary_view_bounds, model.data_bounds_y3?)
        }
        YAxis::Right3 => {
            line_plot_view_bounds_for_y_axis(primary_view_bounds, model.data_bounds_y4?)
        }
    };
    Some(PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), plot.size),
        data,
        x_scale,
        y_scale,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_line_plot_query_drag_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    query_drag_session: &Rc<RefCell<Option<LinePlotQueryDragSession>>>,
    active_selection: &Rc<Cell<Option<LinePlotSelectionOverlay>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    let input_map = PlotInputMap::default();
    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            ..
        }) if plot.contains(*position)
            && input_map
                .query_drag
                .is_some_and(|chord| chord.matches(*button, *modifiers)) =>
        {
            let local = local_from_absolute(plot.origin, *position);
            *query_drag_session.borrow_mut() = Some(LinePlotQueryDragSession {
                start: local,
                current: local,
                button: *button,
            });
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: local,
                current: local,
                kind: LinePlotSelectionKind::Query,
            }));
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) => {
            let Some(mut session) = *query_drag_session.borrow() else {
                return false;
            };
            if !line_plot_mouse_buttons_contains(*buttons, session.button) {
                query_drag_session.borrow_mut().take();
                active_selection.set(None);
                return true;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: session.start,
                current: session.current,
                kind: LinePlotSelectionKind::Query,
            }));
            *query_drag_session.borrow_mut() = Some(session);
            true
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position, button, ..
        }) => {
            let Some(mut session) = query_drag_session.borrow_mut().take() else {
                return false;
            };
            if session.button != *button {
                *query_drag_session.borrow_mut() = Some(session);
                return false;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(None);
            let w = (session.start.x.0 - session.current.x.0).abs();
            let h = (session.start.y.0 - session.current.y.0).abs();
            if w < 4.0 || h < 4.0 {
                return true;
            }

            let current_view = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let Some(next) = line_plot_query_rect_from_plot_points_raw(
                current_view,
                plot.size,
                session.start,
                session.current,
                x_scale,
                y_scale,
            ) else {
                return true;
            };

            state
                .update(app, |state, _cx| {
                    state.query = Some(next);
                    true
                })
                .ok()
                .unwrap_or(false)
        }
        _ => false,
    }
}

pub(in crate::declarative) fn line_plot_query_rect_from_plot_points_raw(
    view_bounds: DataRect,
    viewport: Size,
    a: Point,
    b: Point,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> Option<DataRect> {
    let viewport_w = viewport.width.0;
    let viewport_h = viewport.height.0;
    if !viewport_w.is_finite() || !viewport_h.is_finite() || viewport_w <= 0.0 || viewport_h <= 0.0
    {
        return None;
    }

    let x0 = a.x.0.min(b.x.0).clamp(0.0, viewport_w);
    let x1 = a.x.0.max(b.x.0).clamp(0.0, viewport_w);
    let y0 = a.y.0.min(b.y.0).clamp(0.0, viewport_h);
    let y1 = a.y.0.max(b.y.0).clamp(0.0, viewport_h);

    let transform = PlotTransform {
        viewport: Rect::new(Point::new(Px(0.0), Px(0.0)), viewport),
        data: view_bounds,
        x_scale,
        y_scale,
    };
    let a = transform.px_to_data(Point::new(Px(x0), Px(y0)));
    let b = transform.px_to_data(Point::new(Px(x1), Px(y1)));
    if !a.x.is_finite() || !a.y.is_finite() || !b.x.is_finite() || !b.y.is_finite() {
        return None;
    }

    Some(DataRect {
        x_min: a.x.min(b.x),
        x_max: a.x.max(b.x),
        y_min: a.y.min(b.y),
        y_max: a.y.max(b.y),
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_line_plot_box_zoom_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    box_zoom_session: &Rc<RefCell<Option<LinePlotBoxZoomSession>>>,
    active_selection: &Rc<Cell<Option<LinePlotSelectionOverlay>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    let input_map = PlotInputMap::default();
    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            ..
        }) if plot.contains(*position) => {
            let start_box_primary = input_map.box_zoom.matches(*button, *modifiers);
            let start_box_alt = input_map
                .box_zoom_alt
                .is_some_and(|chord| chord.matches(*button, *modifiers));
            if !start_box_primary && !start_box_alt {
                return false;
            }
            if line_plot_legend_hit(model, plot, *position).is_some() {
                return false;
            }

            let local = local_from_absolute(plot.origin, *position);
            *box_zoom_session.borrow_mut() = Some(LinePlotBoxZoomSession {
                start: local,
                current: local,
                button: *button,
                required_mods: if start_box_primary {
                    input_map.box_zoom.modifiers
                } else {
                    input_map
                        .box_zoom_alt
                        .unwrap_or(input_map.box_zoom)
                        .modifiers
                },
            });
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: local,
                current: local,
                kind: LinePlotSelectionKind::BoxZoom,
            }));
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) => {
            let Some(mut session) = *box_zoom_session.borrow() else {
                return false;
            };
            if !line_plot_mouse_buttons_contains(*buttons, session.button) {
                box_zoom_session.borrow_mut().take();
                active_selection.set(None);
                return true;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(Some(LinePlotSelectionOverlay {
                start: session.start,
                current: session.current,
                kind: LinePlotSelectionKind::BoxZoom,
            }));
            *box_zoom_session.borrow_mut() = Some(session);
            true
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button,
            modifiers,
            ..
        }) => {
            let Some(mut session) = box_zoom_session.borrow_mut().take() else {
                return false;
            };
            if session.button != *button {
                *box_zoom_session.borrow_mut() = Some(session);
                return false;
            }
            session.current = local_from_absolute(plot.origin, *position);
            active_selection.set(None);
            let (start, end) = line_plot_apply_box_select_modifiers(
                plot.size,
                session.start,
                session.current,
                *modifiers,
                input_map.box_zoom_expand_x,
                input_map.box_zoom_expand_y,
                session.required_mods,
            );
            let w = (start.x.0 - end.x.0).abs();
            let h = (start.y.0 - end.y.0).abs();
            if w < 4.0 || h < 4.0 {
                return true;
            }

            let current_view = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let axis_locks = state
                .read_ref(app, |state| state.axis_locks)
                .unwrap_or_default();
            if axis_locks.x.zoom && axis_locks.y.zoom {
                return true;
            }

            let Some(mut next) = data_rect_from_plot_points_scaled(
                current_view,
                plot.size,
                start,
                end,
                x_scale,
                y_scale,
            ) else {
                return true;
            };
            if style.clamp_to_data_bounds {
                next = clamp_view_to_data_scaled(
                    next,
                    model.data_bounds,
                    style.overscroll_fraction,
                    x_scale,
                    y_scale,
                );
            }
            if axis_locks.x.zoom {
                next.x_min = current_view.x_min;
                next.x_max = current_view.x_max;
            }
            if axis_locks.y.zoom {
                next.y_min = current_view.y_min;
                next.y_max = current_view.y_max;
            }
            next = sanitize_data_rect_scaled(next, x_scale, y_scale);
            if next == current_view {
                return true;
            }

            state
                .update(app, |state, _cx| {
                    state.view_is_auto = false;
                    state.view_bounds = Some(next);
                    true
                })
                .ok()
                .unwrap_or(false)
        }
        _ => false,
    }
}

fn line_plot_mouse_buttons_contains(buttons: fret_core::MouseButtons, button: MouseButton) -> bool {
    match button {
        MouseButton::Left => buttons.left,
        MouseButton::Right => buttons.right,
        MouseButton::Middle => buttons.middle,
        MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
    }
}

fn line_plot_apply_box_select_modifiers(
    plot_size: Size,
    start: Point,
    end: Point,
    modifiers: fret_core::Modifiers,
    expand_x: Option<ModifierKey>,
    expand_y: Option<ModifierKey>,
    required: ModifiersMask,
) -> (Point, Point) {
    let mut start = start;
    let mut end = end;

    if expand_x.is_some_and(|key| key.is_pressed(modifiers) && !key.is_required_by(required)) {
        start.x = Px(0.0);
        end.x = plot_size.width;
    }
    if expand_y.is_some_and(|key| key.is_pressed(modifiers) && !key.is_required_by(required)) {
        start.y = Px(0.0);
        end.y = plot_size.height;
    }

    (start, end)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_line_plot_pan_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    pan_session: &Rc<RefCell<Option<LinePlotPanSession>>>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers,
            ..
        }) if !modifiers.shift && !modifiers.alt && !modifiers.ctrl && plot.contains(*position) => {
            if line_plot_legend_hit(model, plot, *position).is_some() {
                return false;
            }
            *pan_session.borrow_mut() = Some(LinePlotPanSession {
                last_position: *position,
            });
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position, buttons, ..
        }) if buttons.left => {
            let Some(mut session) = *pan_session.borrow() else {
                return false;
            };
            let current_view = line_plot_current_view_bounds_for_event(
                app,
                Some(state),
                model,
                style,
                x_scale,
                y_scale,
            );
            let dx_px = position.x.0 - session.last_position.x.0;
            let dy_px = position.y.0 - session.last_position.y.0;
            if dx_px == 0.0 && dy_px == 0.0 {
                return true;
            }
            let mut next =
                pan_line_plot_view_bounds(current_view, plot, dx_px, dy_px, x_scale, y_scale);
            let axis_locks = state
                .read_ref(app, |state| state.axis_locks)
                .unwrap_or_default();
            if axis_locks.x.pan {
                next.x_min = current_view.x_min;
                next.x_max = current_view.x_max;
            }
            if axis_locks.y.pan {
                next.y_min = current_view.y_min;
                next.y_max = current_view.y_max;
            }
            let _ = state.update(app, |state, _cx| {
                state.view_is_auto = false;
                state.view_bounds = Some(next);
            });
            session.last_position = *position;
            *pan_session.borrow_mut() = Some(session);
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move { buttons, .. }) if !buttons.left => {
            pan_session.borrow_mut().take().is_some()
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            button: MouseButton::Left,
            ..
        }) => pan_session.borrow_mut().take().is_some(),
        _ => false,
    }
}

fn pan_line_plot_view_bounds(
    view: DataRect,
    plot: Rect,
    dx_px: f32,
    dy_px: f32,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> DataRect {
    let pan_axis = |scale: AxisScale, min: f64, max: f64, delta_px: f32, span_px: f32| {
        let Some(axis_min) = scale.to_axis(min) else {
            return (min, max);
        };
        let Some(axis_max) = scale.to_axis(max) else {
            return (min, max);
        };
        if span_px <= 0.0 {
            return (min, max);
        }
        let axis_delta = -(delta_px as f64) / span_px as f64 * (axis_max - axis_min);
        (
            scale.from_axis(axis_min + axis_delta).unwrap_or(min),
            scale.from_axis(axis_max + axis_delta).unwrap_or(max),
        )
    };
    let (x_min, x_max) = pan_axis(x_scale, view.x_min, view.x_max, dx_px, plot.size.width.0);
    let (y_min, y_max) = pan_axis(y_scale, view.y_min, view.y_max, -dy_px, plot.size.height.0);
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

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_line_plot_wheel_zoom_event<H: UiHost>(
    app: &mut H,
    state: &Model<PlotState>,
    event: &Event,
    bounds: Rect,
    model: &PlotPanelModel,
    style: LinePlotStyle,
    x_scale: AxisScale,
    y_scale: AxisScale,
) -> bool {
    let Event::Pointer(fret_core::PointerEvent::Wheel {
        position,
        delta,
        modifiers,
        ..
    }) = event
    else {
        return false;
    };

    let Some(region) = line_plot_wheel_region_at(bounds, style, *position) else {
        return false;
    };
    let plot = line_plot_inner_rect(bounds, style);
    if plot.size.width.0 <= 0.0 || plot.size.height.0 <= 0.0 {
        return false;
    }

    let input_map = PlotInputMap::default();
    if let Some(required) = input_map.wheel_zoom_mod
        && !required.is_pressed(*modifiers)
    {
        return false;
    }

    let delta_y = delta.y.0;
    if !delta_y.is_finite() {
        return false;
    }

    let speed = if input_map.wheel_zoom_log2_per_px.is_finite() {
        input_map.wheel_zoom_log2_per_px
    } else {
        PlotInputMap::default().wheel_zoom_log2_per_px
    };
    let zoom = clamp_zoom_factors(2.0_f32.powf(delta_y * speed));
    let mut zoom_x = zoom;
    let mut zoom_y = zoom;

    match region {
        LinePlotWheelRegion::Plot => {
            let x_only = input_map
                .wheel_zoom_x_only_mod
                .is_some_and(|modifier| modifier.is_pressed(*modifiers));
            let y_only = input_map
                .wheel_zoom_y_only_mod
                .is_some_and(|modifier| modifier.is_pressed(*modifiers));
            if x_only {
                zoom_y = 1.0;
            } else if y_only {
                zoom_x = 1.0;
            }
        }
        LinePlotWheelRegion::XAxis => {
            zoom_y = 1.0;
        }
        LinePlotWheelRegion::YAxis => {
            zoom_x = 1.0;
        }
    }

    let axis_locks = state
        .read_ref(app, |state| state.axis_locks)
        .unwrap_or_default();
    if axis_locks.x.zoom {
        zoom_x = 1.0;
    }
    if axis_locks.y.zoom {
        zoom_y = 1.0;
    }

    if zoom_x == 1.0 && zoom_y == 1.0 {
        return false;
    }

    let current =
        line_plot_current_view_bounds_for_event(app, Some(state), model, style, x_scale, y_scale);
    let local = local_from_absolute(plot.origin, *position);
    let Some(mut next) =
        zoom_view_at_px_scaled(current, plot.size, local, zoom_x, zoom_y, x_scale, y_scale)
    else {
        return false;
    };
    if style.clamp_to_data_bounds {
        next = clamp_view_to_data_scaled(
            next,
            model.data_bounds,
            style.overscroll_fraction,
            x_scale,
            y_scale,
        );
    }
    next = sanitize_data_rect_scaled(next, x_scale, y_scale);
    if next == current {
        return false;
    }

    state
        .update(app, |state, _cx| {
            state.view_is_auto = false;
            state.view_bounds = Some(next);
            true
        })
        .ok()
        .unwrap_or(false)
}

fn line_plot_wheel_region_at(
    bounds: Rect,
    style: LinePlotStyle,
    position: Point,
) -> Option<LinePlotWheelRegion> {
    let plot = line_plot_inner_rect(bounds, style);
    if plot.contains(position) {
        return Some(LinePlotWheelRegion::Plot);
    }

    let pad = style.padding.0.max(0.0);
    let axis_gap = style.axis_gap.0.max(0.0);
    let y_axis = Rect::new(
        Point::new(Px(bounds.origin.x.0 + pad), plot.origin.y),
        Size::new(Px(axis_gap), plot.size.height),
    );
    if y_axis.contains(position) {
        return Some(LinePlotWheelRegion::YAxis);
    }

    let x_axis = Rect::new(
        Point::new(plot.origin.x, Px(plot.origin.y.0 + plot.size.height.0)),
        Size::new(plot.size.width, Px(axis_gap)),
    );
    if x_axis.contains(position) {
        return Some(LinePlotWheelRegion::XAxis);
    }

    None
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
