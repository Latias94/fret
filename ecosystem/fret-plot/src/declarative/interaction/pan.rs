//! Declarative line-plot pan event owner.

use std::cell::RefCell;
use std::rc::Rc;

use fret_core::{Event, MouseButton, Point, Rect};
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::cartesian::{AxisScale, DataRect};
use crate::plot::view::{apply_axis_locks, sanitize_data_rect_scaled};
use crate::state::PlotState;
use crate::style::LinePlotStyle;

use super::super::geometry::line_plot_inner_rect;
use super::super::legend::line_plot_legend_hit;
use super::super::model::PlotPanelModel;
use super::super::output::line_plot_current_view_bounds_for_event;

#[derive(Debug, Clone, Copy)]
pub(in crate::declarative) struct LinePlotPanSession {
    last_position: Point,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::declarative) fn handle_line_plot_pan_event<H: UiHost>(
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
            next = apply_axis_locks(current_view, next, axis_locks.x.pan, axis_locks.y.pan);
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
