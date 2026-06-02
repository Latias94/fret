//! Declarative line-plot query drag event owner.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use fret_core::{Event, MouseButton, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::cartesian::{AxisScale, DataRect, PlotTransform};
use crate::input_map::PlotInputMap;
use crate::plot::view::local_from_absolute;
use crate::state::PlotState;
use crate::style::LinePlotStyle;

use super::super::geometry::line_plot_inner_rect;
use super::super::model::PlotPanelModel;
use super::super::output::line_plot_current_view_bounds_for_event;
use super::{LinePlotSelectionKind, LinePlotSelectionOverlay, line_plot_mouse_buttons_contains};

#[derive(Debug, Clone, Copy)]
pub(in crate::declarative) struct LinePlotQueryDragSession {
    start: Point,
    current: Point,
    button: MouseButton,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::declarative) fn handle_line_plot_query_drag_event<H: UiHost>(
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
