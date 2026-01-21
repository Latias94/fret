use fret_core::{Point, Px, Rect, Size};

use super::PanZoom2D;

fn rect_min_max(rect: Rect) -> (f32, f32, f32, f32) {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;
    (x0.min(x1), x0.max(x1), y0.min(y1), y0.max(y1))
}

fn rect_from_min_max(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Rect {
    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px((max_x - min_x).max(0.0)), Px((max_y - min_y).max(0.0))),
    )
}

/// Maps a window/screen-space rect into canvas space under a `PanZoom2D` view.
///
/// Rects are treated as axis-aligned AABBs and remain axis-aligned under uniform scale.
pub fn screen_rect_to_canvas_rect(bounds: Rect, view: PanZoom2D, screen: Rect) -> Rect {
    let (sx0, sx1, sy0, sy1) = rect_min_max(screen);
    if !(sx0.is_finite() && sx1.is_finite() && sy0.is_finite() && sy1.is_finite()) {
        return Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(0.0), Px(0.0)));
    }

    let p0 = view.screen_to_canvas(bounds, Point::new(Px(sx0), Px(sy0)));
    let p1 = view.screen_to_canvas(bounds, Point::new(Px(sx1), Px(sy1)));
    let min_x = p0.x.0.min(p1.x.0);
    let max_x = p0.x.0.max(p1.x.0);
    let min_y = p0.y.0.min(p1.y.0);
    let max_y = p0.y.0.max(p1.y.0);
    rect_from_min_max(min_x, max_x, min_y, max_y)
}

/// Maps a canvas-space rect into window/screen space under a `PanZoom2D` view.
///
/// Rects are treated as axis-aligned AABBs and remain axis-aligned under uniform scale.
pub fn canvas_rect_to_screen_rect(bounds: Rect, view: PanZoom2D, canvas: Rect) -> Rect {
    let (cx0, cx1, cy0, cy1) = rect_min_max(canvas);
    if !(cx0.is_finite() && cx1.is_finite() && cy0.is_finite() && cy1.is_finite()) {
        return Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(0.0), Px(0.0)));
    }

    let p0 = view.canvas_to_screen(bounds, Point::new(Px(cx0), Px(cy0)));
    let p1 = view.canvas_to_screen(bounds, Point::new(Px(cx1), Px(cy1)));
    let min_x = p0.x.0.min(p1.x.0);
    let max_x = p0.x.0.max(p1.x.0);
    let min_y = p0.y.0.min(p1.y.0);
    let max_y = p0.y.0.max(p1.y.0);
    rect_from_min_max(min_x, max_x, min_y, max_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_round_trips_through_screen_and_canvas() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let view = PanZoom2D {
            pan: Point::new(Px(-3.0), Px(5.0)),
            zoom: 2.0,
        };

        let screen = Rect::new(
            Point::new(Px(100.0), Px(120.0)),
            Size::new(Px(200.0), Px(50.0)),
        );
        let canvas = screen_rect_to_canvas_rect(bounds, view, screen);
        let screen2 = canvas_rect_to_screen_rect(bounds, view, canvas);

        assert!((screen.origin.x.0 - screen2.origin.x.0).abs() <= 1.0e-5);
        assert!((screen.origin.y.0 - screen2.origin.y.0).abs() <= 1.0e-5);
        assert!((screen.size.width.0 - screen2.size.width.0).abs() <= 1.0e-5);
        assert!((screen.size.height.0 - screen2.size.height.0).abs() <= 1.0e-5);
    }
}
