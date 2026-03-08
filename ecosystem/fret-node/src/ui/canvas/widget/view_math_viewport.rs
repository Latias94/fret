use fret_canvas::view::{PanZoomConstraints2D, clamp_pan_zoom_view};
use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, CanvasSize};
use crate::ui::canvas::state::ViewSnapshot;

use super::{CanvasViewport2D, PanZoom2D};

pub(super) fn viewport_from_pan_zoom(
    bounds: Rect,
    pan: CanvasPoint,
    zoom: f32,
) -> CanvasViewport2D {
    CanvasViewport2D::new(
        bounds,
        PanZoom2D {
            pan: Point::new(Px(pan.x), Px(pan.y)),
            zoom,
        },
    )
}

pub(super) fn viewport_from_snapshot(bounds: Rect, snapshot: &ViewSnapshot) -> CanvasViewport2D {
    viewport_from_pan_zoom(bounds, snapshot.pan, snapshot.zoom)
}

pub(super) fn screen_to_canvas(
    bounds: Rect,
    screen: Point,
    pan: CanvasPoint,
    zoom: f32,
) -> CanvasPoint {
    let viewport = viewport_from_pan_zoom(bounds, pan, zoom);
    let point = viewport.screen_to_canvas(screen);
    CanvasPoint {
        x: point.x.0,
        y: point.y.0,
    }
}

pub(super) fn clamp_pan_to_translate_extent(
    pan: CanvasPoint,
    zoom: f32,
    bounds: Rect,
    extent: crate::core::CanvasRect,
) -> CanvasPoint {
    let extent_rect = Rect::new(
        Point::new(Px(extent.origin.x), Px(extent.origin.y)),
        Size::new(Px(extent.size.width), Px(extent.size.height)),
    );

    let view = clamp_pan_zoom_view(
        bounds,
        PanZoom2D {
            pan: Point::new(Px(pan.x), Px(pan.y)),
            zoom,
        },
        PanZoomConstraints2D {
            min_zoom: zoom,
            max_zoom: zoom,
            translate_extent_canvas: Some(extent_rect),
        },
    );

    CanvasPoint {
        x: view.pan.x.0,
        y: view.pan.y.0,
    }
}

pub(super) fn snap_canvas_point(pos: CanvasPoint, grid: CanvasSize) -> CanvasPoint {
    CanvasPoint {
        x: snap_axis(pos.x, grid.width),
        y: snap_axis(pos.y, grid.height),
    }
}

fn snap_axis(value: f32, grid: f32) -> f32 {
    if !value.is_finite() {
        return value;
    }
    if !grid.is_finite() || grid <= 0.0 {
        return value;
    }
    (value / grid).round() * grid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_canvas_point_rounds_each_axis_independently() {
        let snapped = snap_canvas_point(
            CanvasPoint { x: 23.0, y: 46.0 },
            CanvasSize {
                width: 10.0,
                height: 25.0,
            },
        );
        assert_eq!(snapped.x, 20.0);
        assert_eq!(snapped.y, 50.0);
    }

    #[test]
    fn snap_canvas_point_ignores_invalid_grid_axis() {
        let snapped = snap_canvas_point(
            CanvasPoint { x: 23.0, y: 46.0 },
            CanvasSize {
                width: 0.0,
                height: f32::NAN,
            },
        );
        assert_eq!(snapped.x, 23.0);
        assert_eq!(snapped.y, 46.0);
    }
}
