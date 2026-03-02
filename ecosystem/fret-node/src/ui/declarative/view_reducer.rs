use fret_canvas::view::PanZoom2D;
use fret_core::{Point, Px, Rect};

use crate::core::CanvasPoint;
use crate::io::NodeGraphViewState;

pub fn view_from_state(state: &NodeGraphViewState) -> PanZoom2D {
    PanZoom2D {
        pan: Point::new(Px(state.pan.x), Px(state.pan.y)),
        zoom: state.zoom,
    }
}

pub fn write_view_to_state(state: &mut NodeGraphViewState, view: PanZoom2D) {
    state.pan = CanvasPoint {
        x: view.pan.x.0,
        y: view.pan.y.0,
    };
    state.zoom = view.zoom;
}

pub fn apply_pan_by_screen_delta(
    state: &mut NodeGraphViewState,
    dx_screen_px: f32,
    dy_screen_px: f32,
) {
    let mut view = view_from_state(state);
    view.pan_by_screen_delta(dx_screen_px, dy_screen_px);
    write_view_to_state(state, view);
}

pub fn apply_zoom_about_screen_point(
    state: &mut NodeGraphViewState,
    bounds: Rect,
    center_screen: Point,
    new_zoom: f32,
    min_zoom: f32,
    max_zoom: f32,
) {
    let mut view = view_from_state(state);
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0);
    let new_zoom = PanZoom2D::sanitize_zoom(new_zoom, zoom).clamp(min_zoom, max_zoom);
    view.zoom_about_screen_point(bounds, center_screen, new_zoom);
    write_view_to_state(state, view);
}

pub fn apply_fit_view_to_canvas_rect(
    state: &mut NodeGraphViewState,
    bounds: Rect,
    target_canvas: Rect,
    padding_screen_px: f32,
    min_zoom: f32,
    max_zoom: f32,
) -> bool {
    if !bounds.size.width.0.is_finite()
        || !bounds.size.height.0.is_finite()
        || bounds.size.width.0 <= 0.0
        || bounds.size.height.0 <= 0.0
    {
        return false;
    }

    if !target_canvas.size.width.0.is_finite()
        || !target_canvas.size.height.0.is_finite()
        || target_canvas.size.width.0 <= 0.0
        || target_canvas.size.height.0 <= 0.0
    {
        return false;
    }

    let padding = if padding_screen_px.is_finite() {
        padding_screen_px.max(0.0)
    } else {
        0.0
    };

    let avail_w = (bounds.size.width.0 - 2.0 * padding).max(1.0);
    let avail_h = (bounds.size.height.0 - 2.0 * padding).max(1.0);

    let zoom_x = avail_w / target_canvas.size.width.0;
    let zoom_y = avail_h / target_canvas.size.height.0;
    if !zoom_x.is_finite() || !zoom_y.is_finite() {
        return false;
    }

    let new_zoom = PanZoom2D::sanitize_zoom(zoom_x.min(zoom_y), 1.0).clamp(min_zoom, max_zoom);

    let screen_center = Point::new(
        Px(bounds.origin.x.0 + padding + avail_w * 0.5),
        Px(bounds.origin.y.0 + padding + avail_h * 0.5),
    );
    let canvas_center = Point::new(
        Px(target_canvas.origin.x.0 + target_canvas.size.width.0 * 0.5),
        Px(target_canvas.origin.y.0 + target_canvas.size.height.0 * 0.5),
    );

    let new_pan_x = (screen_center.x.0 - bounds.origin.x.0) / new_zoom - canvas_center.x.0;
    let new_pan_y = (screen_center.y.0 - bounds.origin.y.0) / new_zoom - canvas_center.y.0;
    if !new_pan_x.is_finite() || !new_pan_y.is_finite() {
        return false;
    }

    write_view_to_state(
        state,
        PanZoom2D {
            pan: Point::new(Px(new_pan_x), Px(new_pan_y)),
            zoom: new_zoom,
        },
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pan_by_screen_delta_scales_by_zoom() {
        let mut state = NodeGraphViewState::default();
        state.zoom = 2.0;
        state.pan = CanvasPoint { x: 0.0, y: 0.0 };

        apply_pan_by_screen_delta(&mut state, 10.0, -6.0);

        // Pan is in canvas units, so screen delta is divided by zoom.
        assert!((state.pan.x - 5.0).abs() <= 1.0e-6);
        assert!((state.pan.y - -3.0).abs() <= 1.0e-6);
    }

    #[test]
    fn zoom_about_screen_point_keeps_canvas_point_stable() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );

        let mut state = NodeGraphViewState::default();
        state.pan = CanvasPoint { x: -3.0, y: 5.0 };
        state.zoom = 2.0;

        let center = Point::new(Px(200.0), Px(150.0));
        let before = view_from_state(&state).screen_to_canvas(bounds, center);

        apply_zoom_about_screen_point(&mut state, bounds, center, 3.0, 0.05, 64.0);

        let after = view_from_state(&state).screen_to_canvas(bounds, center);

        assert!((before.x.0 - after.x.0).abs() <= 1.0e-6);
        assert!((before.y.0 - after.y.0).abs() <= 1.0e-6);
    }

    #[test]
    fn fit_view_places_target_inside_bounds_with_padding() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );
        let target = Rect::new(
            Point::new(Px(100.0), Px(50.0)),
            fret_core::Size::new(Px(400.0), Px(200.0)),
        );

        let mut state = NodeGraphViewState::default();
        let applied = apply_fit_view_to_canvas_rect(&mut state, bounds, target, 24.0, 0.05, 64.0);
        assert!(applied);

        let view = view_from_state(&state);

        let tl = view.canvas_to_screen(bounds, target.origin);
        let br = view.canvas_to_screen(
            bounds,
            Point::new(
                Px(target.origin.x.0 + target.size.width.0),
                Px(target.origin.y.0 + target.size.height.0),
            ),
        );

        assert!(tl.x.0 >= bounds.origin.x.0 + 24.0 - 1.0e-3);
        assert!(tl.y.0 >= bounds.origin.y.0 + 24.0 - 1.0e-3);
        assert!(br.x.0 <= bounds.origin.x.0 + bounds.size.width.0 - 24.0 + 1.0e-3);
        assert!(br.y.0 <= bounds.origin.y.0 + bounds.size.height.0 - 24.0 + 1.0e-3);
    }
}
