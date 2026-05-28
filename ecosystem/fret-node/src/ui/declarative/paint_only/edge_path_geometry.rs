use fret_canvas::wires as canvas_wires;
use fret_core::{PathCommand, Point, Px};

pub(super) const EDGE_PATH_ANCHOR_STEPS: usize = canvas_wires::DEFAULT_BEZIER_HIT_TEST_STEPS;

fn point_lerp(from: Point, to: Point, t: f32) -> Point {
    let t = t.clamp(0.0, 1.0);
    Point::new(
        Px(from.x.0 + (to.x.0 - from.x.0) * t),
        Px(from.y.0 + (to.y.0 - from.y.0) * t),
    )
}

fn quad_bezier(p0: Point, p1: Point, p2: Point, t: f32) -> Point {
    let t = t.clamp(0.0, 1.0);
    let mt = 1.0 - t;
    Point::new(
        Px(mt * mt * p0.x.0 + 2.0 * mt * t * p1.x.0 + t * t * p2.x.0),
        Px(mt * mt * p0.y.0 + 2.0 * mt * t * p1.y.0 + t * t * p2.y.0),
    )
}

fn segment_length(from: Point, to: Point) -> Option<f32> {
    let dx = to.x.0 - from.x.0;
    let dy = to.y.0 - from.y.0;
    let len = (dx * dx + dy * dy).sqrt();
    (len.is_finite() && len > 1.0e-6).then_some(len)
}

fn for_each_flattened_segment(
    commands: &[PathCommand],
    steps: usize,
    mut f: impl FnMut(Point, Point),
) {
    let steps = steps.max(1);
    let mut current = None::<Point>;
    let mut subpath_start = None::<Point>;

    for command in commands {
        match *command {
            PathCommand::MoveTo(to) => {
                current = Some(to);
                subpath_start = Some(to);
            }
            PathCommand::LineTo(to) => {
                if let Some(from) = current {
                    f(from, to);
                }
                current = Some(to);
            }
            PathCommand::QuadTo { ctrl, to } => {
                if let Some(from) = current {
                    let mut prev = from;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let next = quad_bezier(from, ctrl, to, t);
                        f(prev, next);
                        prev = next;
                    }
                }
                current = Some(to);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                if let Some(from) = current {
                    let mut prev = from;
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let next = canvas_wires::cubic_bezier(from, ctrl1, ctrl2, to, t);
                        f(prev, next);
                        prev = next;
                    }
                }
                current = Some(to);
            }
            PathCommand::Close => {
                if let (Some(from), Some(to)) = (current, subpath_start) {
                    f(from, to);
                    current = Some(to);
                }
            }
        }
    }
}

pub(super) fn path_midpoint_and_normal(
    commands: &[PathCommand],
    steps: usize,
) -> Option<(Point, Point)> {
    let mut total = 0.0f32;
    for_each_flattened_segment(commands, steps, |from, to| {
        if let Some(len) = segment_length(from, to) {
            total += len;
        }
    });

    if !total.is_finite() || total <= 1.0e-6 {
        return None;
    }

    let target = total * 0.5;
    let mut accumulated = 0.0f32;
    let mut out = None::<(Point, Point)>;

    for_each_flattened_segment(commands, steps, |from, to| {
        if out.is_some() {
            return;
        }
        let Some(len) = segment_length(from, to) else {
            return;
        };
        if accumulated + len >= target {
            let t = ((target - accumulated) / len).clamp(0.0, 1.0);
            let point = point_lerp(from, to, t);
            let tangent = Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0));
            out = Some((point, canvas_wires::normal_from_tangent(tangent)));
        } else {
            accumulated += len;
        }
    });

    out
}
