//! Wire/curve geometry helpers for canvas-like editor widgets.
//!
//! This module is intentionally policy-light:
//! - It provides reusable geometry primitives (Bezier evaluation, tangents, default control points).
//! - It does not encode domain rules (snapping, connection validation, tool modes).
//!
//! The default control point heuristic matches common node-editor conventions (XyFlow/ImGui-style):
//! the curve bends primarily along the X axis with a screen-space clamp that is made zoom-safe.

use fret_core::{Point, Px, Rect, Size};

fn sanitize_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    }
}

/// Default subdivision count for approximate cubic Bezier hit-testing.
///
/// This matches the historical node-graph behavior and is a good conservative baseline.
pub const DEFAULT_BEZIER_HIT_TEST_STEPS: usize = 24;

/// Default cubic Bezier control points for a "wire" connecting `from` -> `to`.
///
/// The control points are chosen so that the curve is mostly horizontal with a zoom-safe
/// screen-space clamp, producing stable looking wires across zoom levels.
pub fn wire_ctrl_points(from: Point, to: Point, zoom: f32) -> (Point, Point) {
    let zoom = sanitize_zoom(zoom);
    let dx = to.x.0 - from.x.0;
    let ctrl = (dx.abs() * 0.5).clamp(40.0 / zoom, 160.0 / zoom);
    let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
    let c1 = Point::new(Px(from.x.0 + dir * ctrl), from.y);
    let c2 = Point::new(Px(to.x.0 - dir * ctrl), to.y);
    (c1, c2)
}

/// Evaluates a cubic Bezier curve at `t` (clamped to [0, 1]).
pub fn cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
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

/// Evaluates the derivative (tangent) of a cubic Bezier curve at `t` (clamped to [0, 1]).
pub fn cubic_bezier_derivative(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
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

/// Returns a unit-length normal vector for the given tangent (or a stable fallback).
pub fn normal_from_tangent(tangent: Point) -> Point {
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

fn dist2_point_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let apx = p.x.0 - a.x.0;
    let apy = p.y.0 - a.y.0;
    let abx = b.x.0 - a.x.0;
    let aby = b.y.0 - a.y.0;

    let ab2 = abx * abx + aby * aby;
    if ab2 <= 1.0e-9 {
        return apx * apx + apy * apy;
    }

    let t = ((apx * abx + apy * aby) / ab2).clamp(0.0, 1.0);
    let cx = a.x.0 + t * abx;
    let cy = a.y.0 + t * aby;
    let dx = p.x.0 - cx;
    let dy = p.y.0 - cy;
    dx * dx + dy * dy
}

fn closest_point_on_segment(p: Point, a: Point, b: Point) -> (Point, f32) {
    let apx = p.x.0 - a.x.0;
    let apy = p.y.0 - a.y.0;
    let abx = b.x.0 - a.x.0;
    let aby = b.y.0 - a.y.0;

    let ab2 = abx * abx + aby * aby;
    if ab2 <= 1.0e-9 {
        let d2 = apx * apx + apy * apy;
        return (a, d2);
    }

    let t = ((apx * abx + apy * aby) / ab2).clamp(0.0, 1.0);
    let cx = a.x.0 + t * abx;
    let cy = a.y.0 + t * aby;
    let dx = p.x.0 - cx;
    let dy = p.y.0 - cy;
    (Point::new(Px(cx), Px(cy)), dx * dx + dy * dy)
}

/// Approximate the squared distance from `p` to the default wire Bezier curve `from -> to`.
///
/// The curve is subdivided into line segments; higher `steps` improves accuracy but costs more.
pub fn bezier_wire_distance2(p: Point, from: Point, to: Point, zoom: f32, steps: usize) -> f32 {
    let steps = steps.max(1);
    let (c1, c2) = wire_ctrl_points(from, to, zoom);

    let mut best = f32::INFINITY;
    let mut prev = from;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let cur = cubic_bezier(from, c1, c2, to, t);
        best = best.min(dist2_point_to_segment(p, prev, cur));
        prev = cur;
    }

    best
}

/// Approximate the closest point from `p` to the default wire Bezier curve `from -> to`.
///
/// The curve is subdivided into line segments; higher `steps` improves accuracy but costs more.
pub fn closest_point_on_bezier_wire(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    steps: usize,
) -> Point {
    let steps = steps.max(1);
    let (c1, c2) = wire_ctrl_points(from, to, zoom);

    let mut best = (from, f32::INFINITY);
    let mut prev = from;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let cur = cubic_bezier(from, c1, c2, to, t);
        let cand = closest_point_on_segment(p, prev, cur);
        if cand.1 < best.1 {
            best = cand;
        }
        prev = cur;
    }

    best.0
}

/// Computes a conservative axis-aligned bounding box for the default wire curve `from -> to`.
///
/// This is intended for coarse culling and spatial indexing. The box is computed from the Bezier
/// end points and the default control points, then expanded by `pad` (canvas units).
pub fn wire_aabb(from: Point, to: Point, zoom: f32, pad: f32) -> Rect {
    let (c1, c2) = wire_ctrl_points(from, to, zoom);
    let mut min_x = from.x.0.min(to.x.0).min(c1.x.0).min(c2.x.0);
    let mut max_x = from.x.0.max(to.x.0).max(c1.x.0).max(c2.x.0);
    let mut min_y = from.y.0.min(to.y.0).min(c1.y.0).min(c2.y.0);
    let mut max_y = from.y.0.max(to.y.0).max(c1.y.0).max(c2.y.0);

    if !min_x.is_finite()
        || !max_x.is_finite()
        || !min_y.is_finite()
        || !max_y.is_finite()
        || min_x > max_x
        || min_y > max_y
    {
        return Rect::new(from, Size::new(Px(0.0), Px(0.0)));
    }

    let pad = if pad.is_finite() { pad.max(0.0) } else { 0.0 };
    min_x -= pad;
    min_y -= pad;
    max_x += pad;
    max_y += pad;

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px((max_x - min_x).max(0.0)), Px((max_y - min_y).max(0.0))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_ctrl_points_are_zoom_safe() {
        let from = Point::new(Px(0.0), Px(0.0));
        let to = Point::new(Px(200.0), Px(0.0));
        let (a1, b1) = wire_ctrl_points(from, to, 2.0);
        let (a2, b2) = wire_ctrl_points(from, to, f32::NAN);
        assert!(a1.x.0.is_finite() && b1.x.0.is_finite());
        assert!(a2.x.0.is_finite() && b2.x.0.is_finite());
    }

    #[test]
    fn bezier_hit_testing_handles_zero_steps() {
        let p = Point::new(Px(10.0), Px(20.0));
        let from = Point::new(Px(0.0), Px(0.0));
        let to = Point::new(Px(100.0), Px(0.0));
        let d2 = bezier_wire_distance2(p, from, to, 1.0, 0);
        assert!(d2.is_finite());
        let _ = closest_point_on_bezier_wire(p, from, to, 1.0, 0);
    }

    #[test]
    fn wire_aabb_is_conservative_and_pad_expands() {
        let from = Point::new(Px(0.0), Px(10.0));
        let to = Point::new(Px(200.0), Px(30.0));
        let a0 = wire_aabb(from, to, 1.0, 0.0);
        assert!(a0.size.width.0 >= 200.0);
        assert!(a0.origin.x.0 <= 0.0);
        assert!(a0.origin.y.0 <= 10.0_f32.min(30.0));

        let a1 = wire_aabb(from, to, 1.0, 10.0);
        assert!(a1.origin.x.0 <= a0.origin.x.0 - 9.9);
        assert!(a1.origin.y.0 <= a0.origin.y.0 - 9.9);
        assert!(a1.size.width.0 >= a0.size.width.0 + 19.9);
        assert!(a1.size.height.0 >= a0.size.height.0 + 19.9);
    }

    #[test]
    fn wire_aabb_handles_non_finite_inputs() {
        let from = Point::new(Px(f32::NAN), Px(0.0));
        let to = Point::new(Px(10.0), Px(10.0));
        let rect = wire_aabb(from, to, 1.0, 0.0);
        assert!(rect.origin.x.0.is_finite());
        assert!(rect.origin.y.0.is_finite());
        assert!(rect.size.width.0.is_finite());
        assert!(rect.size.height.0.is_finite());
    }
}
