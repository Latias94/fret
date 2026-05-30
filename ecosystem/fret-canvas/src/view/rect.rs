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

/// Builds a normalized rect from two corner points.
pub fn rect_from_points(a: Point, b: Point) -> Rect {
    let min_x = a.x.0.min(b.x.0);
    let max_x = a.x.0.max(b.x.0);
    let min_y = a.y.0.min(b.y.0);
    let max_y = a.y.0.max(b.y.0);
    rect_from_min_max(min_x, max_x, min_y, max_y)
}

/// Returns whether `p` is inside `rect`, including the rect edges.
pub fn rect_contains_point(rect: Rect, p: Point) -> bool {
    let (x0, x1, y0, y1) = rect_min_max(rect);
    p.x.0 >= x0 && p.x.0 <= x1 && p.y.0 >= y0 && p.y.0 <= y1
}

/// Returns whether `outer` fully contains `inner`, including matching edges.
pub fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
    let (ox0, ox1, oy0, oy1) = rect_min_max(outer);
    let (ix0, ix1, iy0, iy1) = rect_min_max(inner);
    ix0 >= ox0 && ix1 <= ox1 && iy0 >= oy0 && iy1 <= oy1
}

/// Returns the union of two axis-aligned rects.
pub fn rect_union(a: Rect, b: Rect) -> Rect {
    let (ax0, ax1, ay0, ay1) = rect_min_max(a);
    let (bx0, bx1, by0, by1) = rect_min_max(b);
    rect_from_min_max(ax0.min(bx0), ax1.max(bx1), ay0.min(by0), ay1.max(by1))
}

/// Returns whether two rects intersect, counting touching edges as an intersection.
pub fn rects_intersect(a: Rect, b: Rect) -> bool {
    let (ax0, ax1, ay0, ay1) = rect_min_max(a);
    let (bx0, bx1, by0, by1) = rect_min_max(b);
    ax0 <= bx1 && ax1 >= bx0 && ay0 <= by1 && ay1 >= by0
}

/// Inflates a rect by a finite positive margin on all sides.
pub fn inflate_rect(rect: Rect, margin: f32) -> Rect {
    if !margin.is_finite() || margin <= 0.0 {
        return rect;
    }
    let (x0, x1, y0, y1) = rect_min_max(rect);
    rect_from_min_max(x0 - margin, x1 + margin, y0 - margin, y1 + margin)
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

    #[test]
    fn rect_helpers_normalize_and_intersect() {
        let rect = rect_from_points(
            Point::new(Px(20.0), Px(40.0)),
            Point::new(Px(5.0), Px(10.0)),
        );
        assert_eq!(rect.origin.x.0, 5.0);
        assert_eq!(rect.origin.y.0, 10.0);
        assert_eq!(rect.size.width.0, 15.0);
        assert_eq!(rect.size.height.0, 30.0);

        assert!(rect_contains_point(rect, Point::new(Px(20.0), Px(40.0))));
        assert!(rect_contains_rect(
            rect,
            Rect::new(Point::new(Px(10.0), Px(15.0)), Size::new(Px(2.0), Px(3.0)))
        ));
        assert!(rects_intersect(
            rect,
            Rect::new(
                Point::new(Px(20.0), Px(40.0)),
                Size::new(Px(10.0), Px(10.0))
            )
        ));
    }

    #[test]
    fn rect_union_and_inflate_are_stable_for_bad_margin() {
        let a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let b = Rect::new(Point::new(Px(8.0), Px(-2.0)), Size::new(Px(10.0), Px(4.0)));
        let union = rect_union(a, b);
        assert_eq!(union.origin.x.0, 0.0);
        assert_eq!(union.origin.y.0, -2.0);
        assert_eq!(union.size.width.0, 18.0);
        assert_eq!(union.size.height.0, 12.0);

        assert_eq!(inflate_rect(a, f32::NAN), a);
        let inflated = inflate_rect(a, 2.0);
        assert_eq!(inflated.origin.x.0, -2.0);
        assert_eq!(inflated.origin.y.0, -2.0);
        assert_eq!(inflated.size.width.0, 14.0);
        assert_eq!(inflated.size.height.0, 14.0);
    }
}
