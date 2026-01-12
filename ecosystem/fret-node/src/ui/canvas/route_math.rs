//! Shared edge routing math for the node graph canvas.
//!
//! This module intentionally contains only pure geometry helpers (no UI state).

use fret_core::{Point, Px};

use crate::ui::presenter::EdgeRouteKind;

pub(super) fn wire_ctrl_points(from: Point, to: Point, zoom: f32) -> (Point, Point) {
    let dx = to.x.0 - from.x.0;
    let ctrl = (dx.abs() * 0.5).clamp(40.0 / zoom, 160.0 / zoom);
    let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
    let c1 = Point::new(Px(from.x.0 + dir * ctrl), from.y);
    let c2 = Point::new(Px(to.x.0 - dir * ctrl), to.y);
    (c1, c2)
}

pub(super) fn cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    let t = t.clamp(0.0, 1.0);
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

    let w0 = mt2 * mt;
    let w1 = 3.0 * mt2 * t;
    let w2 = 3.0 * mt * t2;
    let w3 = t2 * t;

    Point::new(
        Px(w0 * p0.x.0 + w1 * p1.x.0 + w2 * p2.x.0 + w3 * p3.x.0),
        Px(w0 * p0.y.0 + w1 * p1.y.0 + w2 * p2.y.0 + w3 * p3.y.0),
    )
}

pub(super) fn cubic_bezier_derivative(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    let t = t.clamp(0.0, 1.0);
    let mt = 1.0 - t;

    let w0 = 3.0 * mt * mt;
    let w1 = 6.0 * mt * t;
    let w2 = 3.0 * t * t;

    Point::new(
        Px(w0 * (p1.x.0 - p0.x.0) + w1 * (p2.x.0 - p1.x.0) + w2 * (p3.x.0 - p2.x.0)),
        Px(w0 * (p1.y.0 - p0.y.0) + w1 * (p2.y.0 - p1.y.0) + w2 * (p3.y.0 - p2.y.0)),
    )
}

pub(super) fn edge_route_start_tangent(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
) -> Point {
    match route {
        EdgeRouteKind::Bezier => {
            let (c1, c2) = wire_ctrl_points(from, to, zoom);
            cubic_bezier_derivative(from, c1, c2, to, 0.0)
        }
        EdgeRouteKind::Straight => Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0)),
        EdgeRouteKind::Step => {
            let mx = 0.5 * (from.x.0 + to.x.0);
            let p1 = Point::new(Px(mx), from.y);
            let p2 = Point::new(Px(mx), to.y);
            let d0 = Point::new(Px(p1.x.0 - from.x.0), Px(p1.y.0 - from.y.0));
            if (d0.x.0 * d0.x.0 + d0.y.0 * d0.y.0) > 1.0e-12 {
                return d0;
            }
            let d1 = Point::new(Px(p2.x.0 - p1.x.0), Px(p2.y.0 - p1.y.0));
            if (d1.x.0 * d1.x.0 + d1.y.0 * d1.y.0) > 1.0e-12 {
                return d1;
            }
            Point::new(Px(to.x.0 - p2.x.0), Px(to.y.0 - p2.y.0))
        }
    }
}

pub(super) fn edge_route_end_tangent(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
) -> Point {
    match route {
        EdgeRouteKind::Bezier => {
            let (c1, c2) = wire_ctrl_points(from, to, zoom);
            cubic_bezier_derivative(from, c1, c2, to, 1.0)
        }
        EdgeRouteKind::Straight => Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0)),
        EdgeRouteKind::Step => {
            let mx = 0.5 * (from.x.0 + to.x.0);
            let p1 = Point::new(Px(mx), from.y);
            let p2 = Point::new(Px(mx), to.y);
            let d2 = Point::new(Px(to.x.0 - p2.x.0), Px(to.y.0 - p2.y.0));
            if (d2.x.0 * d2.x.0 + d2.y.0 * d2.y.0) > 1.0e-12 {
                return d2;
            }
            let d1 = Point::new(Px(p2.x.0 - p1.x.0), Px(p2.y.0 - p1.y.0));
            if (d1.x.0 * d1.x.0 + d1.y.0 * d1.y.0) > 1.0e-12 {
                return d1;
            }
            Point::new(Px(p1.x.0 - from.x.0), Px(p1.y.0 - from.y.0))
        }
    }
}

pub(super) fn normal_from_tangent(tangent: Point) -> Point {
    let dx = tangent.x.0;
    let dy = tangent.y.0;
    let len = (dx * dx + dy * dy).sqrt();
    if !len.is_finite() || len <= 1.0e-6 {
        return Point::new(Px(0.0), Px(-1.0));
    }
    let nx = -dy / len;
    let ny = dx / len;
    Point::new(Px(nx), Px(ny))
}
