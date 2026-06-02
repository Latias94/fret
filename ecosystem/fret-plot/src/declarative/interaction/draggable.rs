//! Declarative line-plot draggable overlay event owner.

use std::cell::RefCell;
use std::rc::Rc;

use fret_core::{Event, MouseButton, Point, Px, Rect};
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::input_map::PlotInputMap;
use crate::models::YAxis;
use crate::plot::view::local_from_absolute;
use crate::state::{PlotDragOutput, PlotDragPhase, PlotState};
use crate::style::LinePlotStyle;

use super::super::geometry::{line_plot_inner_rect, line_plot_view_bounds_for_y_axis};
use super::super::legend::line_plot_legend_hit;
use super::super::model::PlotPanelModel;
use super::super::output::line_plot_current_view_bounds_for_event;
use super::line_plot_mouse_buttons_contains;

#[derive(Debug, Clone, Copy)]
pub(in crate::declarative) enum LinePlotDragSession {
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
pub(in crate::declarative) enum LinePlotDragRectHandle {
    Inside,
    Left,
    Right,
    Top,
    Bottom,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::declarative) fn handle_line_plot_draggable_overlay_event<H: UiHost>(
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
