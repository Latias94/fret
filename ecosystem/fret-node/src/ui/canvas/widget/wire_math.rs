use super::*;
use fret_core::PathCommand;

pub(super) fn wire_distance2(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    bezier_steps: usize,
) -> f32 {
    fret_canvas::wires::bezier_wire_distance2(p, from, to, zoom, bezier_steps)
}

pub(super) fn closest_point_on_edge_route(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    bezier_steps: usize,
    p: Point,
) -> Point {
    match route {
        EdgeRouteKind::Bezier => closest_point_on_wire_bezier(p, from, to, zoom, bezier_steps),
        EdgeRouteKind::Straight => closest_point_on_segment(p, from, to).0,
        EdgeRouteKind::Step => closest_point_on_step_wire(p, from, to).0,
    }
}

pub(super) fn wire_distance2_path(p: Point, commands: &[PathCommand], bezier_steps: usize) -> f32 {
    let mut best = f32::INFINITY;
    let mut cur: Option<Point> = None;
    let mut subpath_start: Option<Point> = None;

    let steps = bezier_steps.max(1);
    for cmd in commands {
        match *cmd {
            PathCommand::MoveTo(p0) => {
                cur = Some(p0);
                subpath_start = Some(p0);
            }
            PathCommand::LineTo(p1) => {
                if let Some(p0) = cur {
                    best = best.min(dist2_point_to_segment(p, p0, p1));
                }
                cur = Some(p1);
            }
            PathCommand::QuadTo { ctrl, to } => {
                let Some(p0) = cur else {
                    cur = Some(to);
                    subpath_start.get_or_insert(to);
                    continue;
                };
                let mut prev = p0;
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let pt = quad_bezier(p0, ctrl, to, t);
                    best = best.min(dist2_point_to_segment(p, prev, pt));
                    prev = pt;
                }
                cur = Some(to);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                let Some(p0) = cur else {
                    cur = Some(to);
                    subpath_start.get_or_insert(to);
                    continue;
                };
                let mut prev = p0;
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let pt = cubic_bezier(p0, ctrl1, ctrl2, to, t);
                    best = best.min(dist2_point_to_segment(p, prev, pt));
                    prev = pt;
                }
                cur = Some(to);
            }
            PathCommand::Close => {
                if let (Some(p0), Some(p1)) = (cur, subpath_start) {
                    best = best.min(dist2_point_to_segment(p, p0, p1));
                    cur = Some(p1);
                }
            }
        }
    }

    if best.is_finite() {
        best
    } else {
        f32::INFINITY
    }
}

pub(super) fn closest_point_on_path(
    commands: &[PathCommand],
    bezier_steps: usize,
    p: Point,
) -> Point {
    let mut best = (Point::new(Px(0.0), Px(0.0)), f32::INFINITY);
    let mut cur: Option<Point> = None;
    let mut subpath_start: Option<Point> = None;

    let steps = bezier_steps.max(1);
    for cmd in commands {
        match *cmd {
            PathCommand::MoveTo(p0) => {
                cur = Some(p0);
                subpath_start = Some(p0);
            }
            PathCommand::LineTo(p1) => {
                if let Some(p0) = cur {
                    let cand = closest_point_on_segment(p, p0, p1);
                    if cand.1 < best.1 {
                        best = cand;
                    }
                }
                cur = Some(p1);
            }
            PathCommand::QuadTo { ctrl, to } => {
                let Some(p0) = cur else {
                    cur = Some(to);
                    subpath_start.get_or_insert(to);
                    continue;
                };
                let mut prev = p0;
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let pt = quad_bezier(p0, ctrl, to, t);
                    let cand = closest_point_on_segment(p, prev, pt);
                    if cand.1 < best.1 {
                        best = cand;
                    }
                    prev = pt;
                }
                cur = Some(to);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                let Some(p0) = cur else {
                    cur = Some(to);
                    subpath_start.get_or_insert(to);
                    continue;
                };
                let mut prev = p0;
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let pt = cubic_bezier(p0, ctrl1, ctrl2, to, t);
                    let cand = closest_point_on_segment(p, prev, pt);
                    if cand.1 < best.1 {
                        best = cand;
                    }
                    prev = pt;
                }
                cur = Some(to);
            }
            PathCommand::Close => {
                if let (Some(p0), Some(p1)) = (cur, subpath_start) {
                    let cand = closest_point_on_segment(p, p0, p1);
                    if cand.1 < best.1 {
                        best = cand;
                    }
                    cur = Some(p1);
                }
            }
        }
    }

    best.0
}

pub(super) fn path_start_end_tangents(commands: &[PathCommand]) -> Option<(Point, Point)> {
    let mut cur: Option<Point> = None;
    let mut subpath_start: Option<Point> = None;

    let mut start: Option<Point> = None;
    let mut end: Option<Point> = None;

    for cmd in commands {
        match *cmd {
            PathCommand::MoveTo(p0) => {
                cur = Some(p0);
                subpath_start = Some(p0);
            }
            PathCommand::LineTo(p1) => {
                if let Some(p0) = cur {
                    let t = Point::new(Px(p1.x.0 - p0.x.0), Px(p1.y.0 - p0.y.0));
                    if start.is_none() {
                        start = Some(t);
                    }
                    end = Some(t);
                }
                cur = Some(p1);
            }
            PathCommand::QuadTo { ctrl, to } => {
                if let Some(p0) = cur {
                    let t0 = Point::new(Px(ctrl.x.0 - p0.x.0), Px(ctrl.y.0 - p0.y.0));
                    let t1 = Point::new(Px(to.x.0 - ctrl.x.0), Px(to.y.0 - ctrl.y.0));
                    if start.is_none() {
                        start = Some(t0);
                    }
                    end = Some(t1);
                }
                cur = Some(to);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                if let Some(p0) = cur {
                    let t0 = Point::new(Px(ctrl1.x.0 - p0.x.0), Px(ctrl1.y.0 - p0.y.0));
                    let t1 = Point::new(Px(to.x.0 - ctrl2.x.0), Px(to.y.0 - ctrl2.y.0));
                    if start.is_none() {
                        start = Some(t0);
                    }
                    end = Some(t1);
                }
                cur = Some(to);
            }
            PathCommand::Close => {
                if let (Some(p0), Some(p1)) = (cur, subpath_start) {
                    let t = Point::new(Px(p1.x.0 - p0.x.0), Px(p1.y.0 - p0.y.0));
                    if start.is_none() {
                        start = Some(t);
                    }
                    end = Some(t);
                    cur = Some(p1);
                }
            }
        }
    }

    Some((start?, end?))
}

pub(super) fn path_midpoint_and_normal(
    commands: &[PathCommand],
    bezier_steps: usize,
) -> Option<(Point, Point)> {
    let steps = bezier_steps.max(1);
    let mut points: Vec<Point> = Vec::new();

    let mut cur: Option<Point> = None;
    let mut subpath_start: Option<Point> = None;

    for cmd in commands {
        match *cmd {
            PathCommand::MoveTo(p0) => {
                cur = Some(p0);
                subpath_start = Some(p0);
                if points.is_empty() {
                    points.push(p0);
                } else {
                    points.push(p0);
                }
            }
            PathCommand::LineTo(p1) => {
                if cur.is_some() {
                    points.push(p1);
                }
                cur = Some(p1);
            }
            PathCommand::QuadTo { ctrl, to } => {
                let Some(p0) = cur else {
                    cur = Some(to);
                    subpath_start.get_or_insert(to);
                    points.push(to);
                    continue;
                };
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    points.push(quad_bezier(p0, ctrl, to, t));
                }
                cur = Some(to);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                let Some(p0) = cur else {
                    cur = Some(to);
                    subpath_start.get_or_insert(to);
                    points.push(to);
                    continue;
                };
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    points.push(cubic_bezier(p0, ctrl1, ctrl2, to, t));
                }
                cur = Some(to);
            }
            PathCommand::Close => {
                if let Some(p0) = subpath_start {
                    points.push(p0);
                    cur = Some(p0);
                }
            }
        }
    }

    if points.len() < 2 {
        return None;
    }

    let mut total = 0.0f32;
    for w in points.windows(2) {
        let dx = w[1].x.0 - w[0].x.0;
        let dy = w[1].y.0 - w[0].y.0;
        total += (dx * dx + dy * dy).sqrt();
    }
    if !total.is_finite() || total <= 1.0e-6 {
        return None;
    }

    let target = 0.5 * total;
    let mut acc = 0.0f32;
    for w in points.windows(2) {
        let a = w[0];
        let b = w[1];
        let dx = b.x.0 - a.x.0;
        let dy = b.y.0 - a.y.0;
        let seg = (dx * dx + dy * dy).sqrt();
        if !seg.is_finite() || seg <= 1.0e-6 {
            continue;
        }
        if acc + seg >= target {
            let t = ((target - acc) / seg).clamp(0.0, 1.0);
            let p = Point::new(Px(a.x.0 + t * dx), Px(a.y.0 + t * dy));
            let tangent = Point::new(Px(dx), Px(dy));
            let normal = normal_from_tangent(tangent);
            return Some((p, normal));
        }
        acc += seg;
    }

    let a = points[points.len() - 2];
    let b = points[points.len() - 1];
    let tangent = Point::new(Px(b.x.0 - a.x.0), Px(b.y.0 - a.y.0));
    Some((b, normal_from_tangent(tangent)))
}

fn quad_bezier(p0: Point, p1: Point, p2: Point, t: f32) -> Point {
    let t = t.clamp(0.0, 1.0);
    let mt = 1.0 - t;
    let w0 = mt * mt;
    let w1 = 2.0 * mt * t;
    let w2 = t * t;
    Point::new(
        Px(w0 * p0.x.0 + w1 * p1.x.0 + w2 * p2.x.0),
        Px(w0 * p0.y.0 + w1 * p1.y.0 + w2 * p2.y.0),
    )
}

fn closest_point_on_wire_bezier(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    bezier_steps: usize,
) -> Point {
    fret_canvas::wires::closest_point_on_bezier_wire(p, from, to, zoom, bezier_steps)
}

fn closest_point_on_step_wire(p: Point, from: Point, to: Point) -> (Point, f32) {
    let mx = 0.5 * (from.x.0 + to.x.0);
    let p1 = Point::new(Px(mx), from.y);
    let p2 = Point::new(Px(mx), to.y);
    let c0 = closest_point_on_segment(p, from, p1);
    let c1 = closest_point_on_segment(p, p1, p2);
    let c2 = closest_point_on_segment(p, p2, to);
    [c0, c1, c2]
        .into_iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap_or((from, f32::INFINITY))
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

pub(super) fn step_wire_distance2(p: Point, from: Point, to: Point) -> f32 {
    let mx = 0.5 * (from.x.0 + to.x.0);
    let p1 = Point::new(Px(mx), from.y);
    let p2 = Point::new(Px(mx), to.y);
    let d0 = dist2_point_to_segment(p, from, p1);
    let d1 = dist2_point_to_segment(p, p1, p2);
    let d2 = dist2_point_to_segment(p, p2, to);
    d0.min(d1).min(d2)
}

pub(super) fn dist2_point_to_segment(p: Point, a: Point, b: Point) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_distance2_on_line_is_zeroish() {
        let commands = [
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
        ];

        let p = Point::new(Px(5.0), Px(0.0));
        let d2 = wire_distance2_path(p, &commands, 8);
        assert!(d2.is_finite() && d2 <= 1.0e-6);
    }

    #[test]
    fn path_midpoint_and_normal_is_finite() {
        let commands = [
            PathCommand::MoveTo(Point::new(Px(0.0), Px(0.0))),
            PathCommand::LineTo(Point::new(Px(10.0), Px(0.0))),
        ];

        let (mid, normal) = path_midpoint_and_normal(&commands, 8).expect("midpoint exists");
        assert!((mid.x.0 - 5.0).abs() <= 1.0e-3);
        assert!(mid.y.0.abs() <= 1.0e-3);
        assert!(normal.x.0.is_finite() && normal.y.0.is_finite());
    }

    #[test]
    fn path_start_end_tangents_follow_control_points() {
        let from = Point::new(Px(0.0), Px(0.0));
        let ctrl1 = Point::new(Px(5.0), Px(1.0));
        let ctrl2 = Point::new(Px(6.0), Px(2.0));
        let to = Point::new(Px(10.0), Px(0.0));

        let commands = [
            PathCommand::MoveTo(from),
            PathCommand::CubicTo { ctrl1, ctrl2, to },
        ];

        let (t0, t1) = path_start_end_tangents(&commands).expect("tangents exist");
        assert_eq!(
            t0,
            Point::new(Px(ctrl1.x.0 - from.x.0), Px(ctrl1.y.0 - from.y.0))
        );
        assert_eq!(
            t1,
            Point::new(Px(to.x.0 - ctrl2.x.0), Px(to.y.0 - ctrl2.y.0))
        );
    }
}
