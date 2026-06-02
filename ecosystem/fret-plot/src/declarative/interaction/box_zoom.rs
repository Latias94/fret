//! Declarative line-plot box zoom event owner.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use fret_core::{Event, MouseButton, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::cartesian::AxisScale;
use crate::input_map::{ModifierKey, ModifiersMask, PlotInputMap};
use crate::plot::view::{
    clamp_view_to_data_scaled, data_rect_from_plot_points_scaled, local_from_absolute,
    sanitize_data_rect_scaled,
};
use crate::state::PlotState;
use crate::style::LinePlotStyle;

use super::super::geometry::line_plot_inner_rect;
use super::super::legend::line_plot_legend_hit;
use super::super::model::PlotPanelModel;
use super::super::output::line_plot_current_view_bounds_for_event;
use super::{LinePlotSelectionKind, LinePlotSelectionOverlay, line_plot_mouse_buttons_contains};

#[derive(Debug, Clone, Copy)]
pub(in crate::declarative) struct LinePlotBoxZoomSession {
    start: Point,
    current: Point,
    button: MouseButton,
    required_mods: ModifiersMask,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::declarative) fn handle_line_plot_box_zoom_event<H: UiHost>(
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
