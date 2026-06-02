//! Declarative line-plot wheel zoom event owner.

use fret_core::{Event, Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::cartesian::AxisScale;
use crate::input_map::PlotInputMap;
use crate::plot::view::{
    clamp_view_to_data_scaled, clamp_zoom_factors, local_from_absolute, sanitize_data_rect_scaled,
    zoom_view_at_px_scaled,
};
use crate::state::PlotState;
use crate::style::LinePlotStyle;

use super::super::geometry::line_plot_inner_rect;
use super::super::model::PlotPanelModel;
use super::super::output::line_plot_current_view_bounds_for_event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinePlotWheelRegion {
    Plot,
    XAxis,
    YAxis,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::declarative) fn handle_line_plot_wheel_zoom_event<H: UiHost>(
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
