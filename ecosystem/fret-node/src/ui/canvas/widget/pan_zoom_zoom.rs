use fret_canvas::view::PanZoom2D;
use fret_core::Point;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::core::CanvasPoint;

pub(super) fn zoom_about_center_factor<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    bounds: fret_core::Rect,
    factor: f32,
) {
    let zoom = canvas.cached_zoom;
    if !zoom.is_finite() || zoom <= 0.0 {
        return;
    }
    if !factor.is_finite() || factor <= 0.0 {
        return;
    }

    let new_zoom = (zoom * factor).clamp(
        canvas.style.geometry.min_zoom,
        canvas.style.geometry.max_zoom,
    );
    if (new_zoom - zoom).abs() <= 1.0e-6 {
        return;
    }

    let mut view = cached_pan_zoom_view(canvas, zoom);
    let center = Point::new(
        fret_core::Px(0.5 * bounds.size.width.0),
        fret_core::Px(0.5 * bounds.size.height.0),
    );
    view.zoom_about_screen_point(bounds, center, new_zoom);
    write_cached_pan_zoom(canvas, view);
}

pub(super) fn zoom_about_pointer_factor<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
    factor: f32,
) {
    let zoom = canvas.cached_zoom;
    if !zoom.is_finite() || zoom <= 0.0 {
        return;
    }
    if !factor.is_finite() || factor <= 0.0 {
        return;
    }
    if !position.x.0.is_finite() || !position.y.0.is_finite() {
        return;
    }

    let new_zoom = (zoom * factor).clamp(
        canvas.style.geometry.min_zoom,
        canvas.style.geometry.max_zoom,
    );
    if (new_zoom - zoom).abs() <= 1.0e-6 {
        return;
    }

    let mut view = cached_pan_zoom_view(canvas, zoom);
    view.zoom_about_canvas_point(position, new_zoom);
    write_cached_pan_zoom(canvas, view);
}

fn cached_pan_zoom_view<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    zoom: f32,
) -> PanZoom2D {
    PanZoom2D {
        pan: Point::new(
            fret_core::Px(canvas.cached_pan.x),
            fret_core::Px(canvas.cached_pan.y),
        ),
        zoom,
    }
}

fn write_cached_pan_zoom<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    view: PanZoom2D,
) {
    canvas.cached_pan = CanvasPoint {
        x: view.pan.x.0,
        y: view.pan.y.0,
    };
    canvas.cached_zoom = view.zoom;
}
