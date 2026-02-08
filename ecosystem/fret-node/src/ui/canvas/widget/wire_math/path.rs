use crate::ui::canvas::widget::*;
use fret_core::PathCommand;

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
                    best = best.min(super::dist2_point_to_segment(p, p0, p1));
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
                    best = best.min(super::dist2_point_to_segment(p, prev, pt));
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
                    best = best.min(super::dist2_point_to_segment(p, prev, pt));
                    prev = pt;
                }
                cur = Some(to);
            }
            PathCommand::Close => {
                if let (Some(p0), Some(p1)) = (cur, subpath_start) {
                    best = best.min(super::dist2_point_to_segment(p, p0, p1));
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
                    let cand = super::closest_point_on_segment(p, p0, p1);
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
                    let cand = super::closest_point_on_segment(p, prev, pt);
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
                    let cand = super::closest_point_on_segment(p, prev, pt);
                    if cand.1 < best.1 {
                        best = cand;
                    }
                    prev = pt;
                }
                cur = Some(to);
            }
            PathCommand::Close => {
                if let (Some(p0), Some(p1)) = (cur, subpath_start) {
                    let cand = super::closest_point_on_segment(p, p0, p1);
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
                points.push(p0);
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
