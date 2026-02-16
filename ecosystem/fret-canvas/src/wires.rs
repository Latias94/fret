//! Wire/curve geometry helpers for canvas-like editor widgets.
//!
//! This module is intentionally policy-light:
//! - It provides reusable geometry primitives (Bezier evaluation, tangents, default control points).
//! - It does not encode domain rules (snapping, connection validation, tool modes).
//!
//! The default control point heuristic matches common node-editor conventions (XyFlow/ImGui-style):
//! the curve bends primarily along the X axis with a screen-space clamp that is made zoom-safe.

use fret_core::{Point, Px, Rect, Size};

#[cfg(feature = "kurbo")]
use kurbo::{CubicBez, ParamCurve, ParamCurveNearest};

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

/// Default screen-space tolerance (in px) for adaptive polyline flattening.
///
/// This is not a hit slop. It is the approximation error budget used when converting a Bezier
/// curve to line segments for distance checks.
pub const DEFAULT_BEZIER_FLATTEN_TOLERANCE_SCREEN_PX: f32 = 2.0;

#[cfg(feature = "kurbo")]
fn kurbo_point(p: Point) -> kurbo::Point {
    kurbo::Point::new(p.x.0 as f64, p.y.0 as f64)
}

#[cfg(feature = "kurbo")]
fn fret_point(p: kurbo::Point) -> Point {
    Point::new(Px(p.x as f32), Px(p.y as f32))
}

#[cfg(feature = "kurbo")]
fn kurbo_accuracy_canvas_units(from: Point, to: Point, zoom: f32, steps: usize) -> f64 {
    let z = sanitize_zoom(zoom).max(1.0e-6);
    let steps = steps.max(1) as f32;
    let base_steps = DEFAULT_BEZIER_HIT_TEST_STEPS as f32;

    let dx_screen = (to.x.0 - from.x.0) * z;
    let dy_screen = (to.y.0 - from.y.0) * z;
    let chord_len_screen = (dx_screen * dx_screen + dy_screen * dy_screen)
        .sqrt()
        .max(1.0);
    let segment_len_screen = (chord_len_screen / steps).max(1.0);

    // Kurbo's `nearest()` uses a subdivision scheme controlled by an `accuracy` parameter.
    // We map our historical "polyline subdivision steps" to a similar error budget in screen px.
    //
    // Heuristic:
    // - use a fraction of the implied segment length (so long wires can afford looser accuracy),
    // - scale by `sqrt(base_steps/steps)` so higher step counts request higher precision,
    // - clamp to a reasonable range so we don't under-refine near the hit threshold.
    let step_scale = (base_steps / steps).sqrt().clamp(0.5, 2.0);
    let accuracy_screen_px = (segment_len_screen * 0.35 * step_scale).clamp(0.75, 10.0);

    (accuracy_screen_px / z) as f64
}

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

/// Compute a simple arrowhead triangle at `end`, oriented by `tangent`.
///
/// Returns points in winding order `[tip, left, right]` so callers can fill a closed path:
/// `MoveTo(tip) -> LineTo(left) -> LineTo(right) -> Close`.
///
/// Inputs:
/// - `tangent`: direction at the end of the curve (does not need to be normalized)
/// - `length`: tip-to-base distance in the same units as `Point` (typically logical px)
/// - `width`: base width (left-to-right) in the same units as `Point`
pub fn arrowhead_triangle(end: Point, tangent: Point, length: f32, width: f32) -> [Point; 3] {
    let length = if length.is_finite() {
        length.max(0.0)
    } else {
        0.0
    };
    let width = if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    };

    let dx = tangent.x.0;
    let dy = tangent.y.0;
    let dlen = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = if dlen.is_finite() && dlen > 1.0e-6 {
        (dx / dlen, dy / dlen)
    } else {
        (1.0, 0.0)
    };

    // Unit normal (perp to direction).
    let nx = -uy;
    let ny = ux;
    let half_w = width * 0.5;

    let base = Point::new(Px(end.x.0 - ux * length), Px(end.y.0 - uy * length));
    let left = Point::new(Px(base.x.0 + nx * half_w), Px(base.y.0 + ny * half_w));
    let right = Point::new(Px(base.x.0 - nx * half_w), Px(base.y.0 - ny * half_w));

    [end, left, right]
}

/// Sample a cubic Bezier curve into a polyline point list.
///
/// The output always includes `p0` and `p3` and is cleared before writing.
pub fn cubic_bezier_polyline_points(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    steps: usize,
    out: &mut Vec<Point>,
) {
    out.clear();
    out.push(p0);

    let steps = steps.max(1);
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        out.push(cubic_bezier(p0, p1, p2, p3, t));
    }
}

/// Convert a polyline into dashed line segments.
///
/// This is a stroke-level helper for implementing SVG-like `strokeDasharray` behavior without
/// extending the renderer's path primitive. Callers can draw each returned segment as an
/// independent stroked path.
///
/// Conventions:
/// - `pattern` alternates ON/OFF lengths starting with ON (`pattern[0]`).
/// - Units are the same as the input points (typically logical px in screen space).
/// - `offset` advances the pattern start along the polyline (wraps by the pattern cycle length).
pub fn dash_polyline_segments(
    points: &[Point],
    pattern: &[f32],
    offset: f32,
    out: &mut Vec<(Point, Point)>,
) {
    out.clear();
    if points.len() < 2 {
        return;
    }

    let mut cycle = 0.0_f32;
    for &v in pattern {
        if !v.is_finite() || v <= 0.0 {
            return;
        }
        cycle += v;
    }
    if !cycle.is_finite() || cycle <= 1.0e-6 {
        return;
    }

    let mut phase_ix = 0usize;
    let mut phase_pos = if offset.is_finite() {
        offset.rem_euclid(cycle)
    } else {
        0.0
    };
    while phase_pos > 0.0 {
        let seg = pattern[phase_ix];
        if phase_pos < seg {
            break;
        }
        phase_pos -= seg;
        phase_ix = (phase_ix + 1) % pattern.len();
    }

    let mut on = phase_ix.is_multiple_of(2);
    let mut phase_len = pattern[phase_ix];

    for w in points.windows(2) {
        let a = w[0];
        let b = w[1];

        let dx = b.x.0 - a.x.0;
        let dy = b.y.0 - a.y.0;
        let seg_len = (dx * dx + dy * dy).sqrt();
        if !seg_len.is_finite() || seg_len <= 1.0e-6 {
            continue;
        }

        let ux = dx / seg_len;
        let uy = dy / seg_len;

        let mut dist = 0.0_f32;
        while dist + 1.0e-6 < seg_len {
            let remaining_in_seg = seg_len - dist;
            let remaining_in_phase = (phase_len - phase_pos).max(0.0);
            if remaining_in_phase <= 1.0e-6 {
                phase_ix = (phase_ix + 1) % pattern.len();
                phase_len = pattern[phase_ix];
                phase_pos = 0.0;
                on = phase_ix.is_multiple_of(2);
                continue;
            }

            let step = remaining_in_seg.min(remaining_in_phase);
            if on {
                let p0 = Point::new(Px(a.x.0 + ux * dist), Px(a.y.0 + uy * dist));
                let p1 = Point::new(
                    Px(a.x.0 + ux * (dist + step)),
                    Px(a.y.0 + uy * (dist + step)),
                );
                out.push((p0, p1));
            }

            dist += step;
            phase_pos += step;
            if phase_pos + 1.0e-6 >= phase_len {
                phase_ix = (phase_ix + 1) % pattern.len();
                phase_len = pattern[phase_ix];
                phase_pos = 0.0;
                on = phase_ix.is_multiple_of(2);
            }
        }
    }
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

fn dist_point_to_line(p: Point, a: Point, b: Point) -> f32 {
    let abx = b.x.0 - a.x.0;
    let aby = b.y.0 - a.y.0;
    let len = (abx * abx + aby * aby).sqrt();
    if !len.is_finite() || len <= 1.0e-6 {
        let dx = p.x.0 - a.x.0;
        let dy = p.y.0 - a.y.0;
        return (dx * dx + dy * dy).sqrt();
    }
    let apx = p.x.0 - a.x.0;
    let apy = p.y.0 - a.y.0;
    ((apx * aby - apy * abx).abs() / len).max(0.0)
}

fn midpoint(a: Point, b: Point) -> Point {
    Point::new(Px(0.5 * (a.x.0 + b.x.0)), Px(0.5 * (a.y.0 + b.y.0)))
}

fn cubic_flat_enough(p0: Point, p1: Point, p2: Point, p3: Point, tol: f32) -> bool {
    let tol = if tol.is_finite() { tol.max(0.0) } else { 0.0 };
    if tol <= 1.0e-6 {
        return false;
    }
    dist_point_to_line(p1, p0, p3) <= tol && dist_point_to_line(p2, p0, p3) <= tol
}

fn cubic_subdivide_half(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
) -> (Point, Point, Point, Point, Point, Point, Point, Point) {
    let p01 = midpoint(p0, p1);
    let p12 = midpoint(p1, p2);
    let p23 = midpoint(p2, p3);
    let p012 = midpoint(p01, p12);
    let p123 = midpoint(p12, p23);
    let p0123 = midpoint(p012, p123);

    (p0, p01, p012, p0123, p0123, p123, p23, p3)
}

fn bezier_distance2_polyline_adaptive(
    p: Point,
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    tol: f32,
    max_depth: u8,
) -> f32 {
    let mut best = f32::INFINITY;
    let mut stack: Vec<(Point, Point, Point, Point, u8)> = Vec::with_capacity(32);
    stack.push((p0, p1, p2, p3, max_depth));

    while let Some((a, b, c, d, depth)) = stack.pop() {
        if depth == 0 || cubic_flat_enough(a, b, c, d, tol) {
            best = best.min(dist2_point_to_segment(p, a, d));
            continue;
        }

        let (l0, l1, l2, l3, r0, r1, r2, r3) = cubic_subdivide_half(a, b, c, d);
        stack.push((r0, r1, r2, r3, depth - 1));
        stack.push((l0, l1, l2, l3, depth - 1));
    }

    best
}

fn bezier_closest_point_polyline_adaptive(
    p: Point,
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    tol: f32,
    max_depth: u8,
) -> Point {
    let mut best = (p0, f32::INFINITY);
    let mut stack: Vec<(Point, Point, Point, Point, u8)> = Vec::with_capacity(32);
    stack.push((p0, p1, p2, p3, max_depth));

    while let Some((a, b, c, d, depth)) = stack.pop() {
        if depth == 0 || cubic_flat_enough(a, b, c, d, tol) {
            let cand = closest_point_on_segment(p, a, d);
            if cand.1 < best.1 {
                best = cand;
            }
            continue;
        }

        let (l0, l1, l2, l3, r0, r1, r2, r3) = cubic_subdivide_half(a, b, c, d);
        stack.push((r0, r1, r2, r3, depth - 1));
        stack.push((l0, l1, l2, l3, depth - 1));
    }

    best.0
}

/// Approximate the squared distance from `p` to the default wire Bezier curve `from -> to`.
///
/// The curve is subdivided into line segments; higher `steps` improves accuracy but costs more.
pub fn bezier_wire_distance2_polyline(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    steps: usize,
) -> f32 {
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

/// Approximate the squared distance from `p` to the default wire curve using adaptive flattening.
///
/// `tolerance_screen_px` is an approximation error budget (not hit slop) in screen pixels.
pub fn bezier_wire_distance2_polyline_adaptive(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    tolerance_screen_px: f32,
) -> f32 {
    let zoom = sanitize_zoom(zoom);
    let tol_screen = if tolerance_screen_px.is_finite() {
        tolerance_screen_px.max(0.0)
    } else {
        DEFAULT_BEZIER_FLATTEN_TOLERANCE_SCREEN_PX
    };
    let tol_canvas = if zoom > 1.0e-6 {
        tol_screen / zoom
    } else {
        tol_screen
    };

    let (c1, c2) = wire_ctrl_points(from, to, zoom);
    bezier_distance2_polyline_adaptive(p, from, c1, c2, to, tol_canvas, 10)
}

#[cfg(feature = "kurbo")]
pub fn bezier_wire_distance2_kurbo(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    steps: usize,
) -> f32 {
    let (c1, c2) = wire_ctrl_points(from, to, zoom);
    let curve = CubicBez::new(
        kurbo_point(from),
        kurbo_point(c1),
        kurbo_point(c2),
        kurbo_point(to),
    );
    let accuracy = kurbo_accuracy_canvas_units(from, to, zoom, steps);
    let nearest = curve.nearest(kurbo_point(p), accuracy);
    let d2 = nearest.distance_sq as f32;
    if d2.is_finite() { d2 } else { f32::INFINITY }
}

pub fn bezier_wire_distance2(p: Point, from: Point, to: Point, zoom: f32, steps: usize) -> f32 {
    #[cfg(feature = "kurbo")]
    {
        bezier_wire_distance2_kurbo(p, from, to, zoom, steps)
    }

    #[cfg(not(feature = "kurbo"))]
    bezier_wire_distance2_polyline(p, from, to, zoom, steps)
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
    #[cfg(feature = "kurbo")]
    {
        closest_point_on_bezier_wire_kurbo(p, from, to, zoom, steps)
    }

    #[cfg(not(feature = "kurbo"))]
    closest_point_on_bezier_wire_polyline(p, from, to, zoom, steps)
}

pub fn closest_point_on_bezier_wire_polyline(
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

/// Approximate the closest point from `p` to the default wire curve using adaptive flattening.
///
/// `tolerance_screen_px` is an approximation error budget (not hit slop) in screen pixels.
pub fn closest_point_on_bezier_wire_polyline_adaptive(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    tolerance_screen_px: f32,
) -> Point {
    let zoom = sanitize_zoom(zoom);
    let tol_screen = if tolerance_screen_px.is_finite() {
        tolerance_screen_px.max(0.0)
    } else {
        DEFAULT_BEZIER_FLATTEN_TOLERANCE_SCREEN_PX
    };
    let tol_canvas = if zoom > 1.0e-6 {
        tol_screen / zoom
    } else {
        tol_screen
    };

    let (c1, c2) = wire_ctrl_points(from, to, zoom);
    bezier_closest_point_polyline_adaptive(p, from, c1, c2, to, tol_canvas, 10)
}

#[cfg(feature = "kurbo")]
pub fn closest_point_on_bezier_wire_kurbo(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    steps: usize,
) -> Point {
    let (c1, c2) = wire_ctrl_points(from, to, zoom);
    let curve = CubicBez::new(
        kurbo_point(from),
        kurbo_point(c1),
        kurbo_point(c2),
        kurbo_point(to),
    );
    let accuracy = kurbo_accuracy_canvas_units(from, to, zoom, steps);
    let nearest = curve.nearest(kurbo_point(p), accuracy);
    fret_point(curve.eval(nearest.t))
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
    fn adaptive_polyline_hit_testing_is_finite() {
        let p = Point::new(Px(10.0), Px(20.0));
        let from = Point::new(Px(0.0), Px(0.0));
        let to = Point::new(Px(100.0), Px(50.0));

        let d2 = bezier_wire_distance2_polyline_adaptive(p, from, to, 1.0, 2.0);
        assert!(d2.is_finite());

        let q = closest_point_on_bezier_wire_polyline_adaptive(p, from, to, 1.0, 2.0);
        assert!(q.x.0.is_finite() && q.y.0.is_finite());
    }

    #[test]
    fn dash_polyline_segments_respects_pattern() {
        let a = Point::new(Px(0.0), Px(0.0));
        let b = Point::new(Px(20.0), Px(0.0));
        let mut out = Vec::new();
        dash_polyline_segments(&[a, b], &[5.0, 5.0], 0.0, &mut out);
        assert_eq!(out.len(), 2);
        assert!((out[0].0.x.0 - 0.0).abs() <= 1.0e-6);
        assert!((out[0].1.x.0 - 5.0).abs() <= 1.0e-6);
        assert!((out[1].0.x.0 - 10.0).abs() <= 1.0e-6);
        assert!((out[1].1.x.0 - 15.0).abs() <= 1.0e-6);
    }

    #[test]
    fn dash_polyline_segments_supports_offset() {
        let a = Point::new(Px(0.0), Px(0.0));
        let b = Point::new(Px(20.0), Px(0.0));
        let mut out = Vec::new();
        // Offset by 2px: ON covers [0..3], then [8..13], etc.
        dash_polyline_segments(&[a, b], &[5.0, 5.0], 2.0, &mut out);
        assert!(!out.is_empty());
        assert!((out[0].0.x.0 - 0.0).abs() <= 1.0e-6);
        assert!((out[0].1.x.0 - 3.0).abs() <= 1.0e-6);
    }

    #[test]
    fn arrowhead_triangle_is_finite() {
        let end = Point::new(Px(10.0), Px(5.0));
        let tangent = Point::new(Px(2.0), Px(0.0));
        let tri = arrowhead_triangle(end, tangent, 10.0, 6.0);

        for p in tri {
            assert!(p.x.0.is_finite() && p.y.0.is_finite());
        }

        // Base center should be behind the tip along the tangent direction.
        let base = Point::new(
            Px((tri[1].x.0 + tri[2].x.0) * 0.5),
            Px((tri[1].y.0 + tri[2].y.0) * 0.5),
        );
        assert!(base.x.0 < end.x.0);
        assert!((base.y.0 - end.y.0).abs() <= 1.0e-4);
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
