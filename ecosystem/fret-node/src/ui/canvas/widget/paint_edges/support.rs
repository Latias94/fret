use fret_core::{PathCommand, Point, Px, Rect, Size};

use crate::ui::canvas::widget::EdgeRouteKind;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub(super) fn stable_hash_u64<T: Hash>(tag: u8, key: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    tag.hash(&mut hasher);
    key.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn glow_bounds_for_edge_route(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    width_screen_px: f32,
    blur_radius_screen_px: f32,
) -> Option<Rect> {
    let z = zoom.max(1.0e-6);
    let mut points: [Point; 6] = [from, to, from, to, from, to];
    let mut point_count = 2usize;
    match route {
        EdgeRouteKind::Bezier => {
            let dx = to.x.0 - from.x.0;
            let ctrl = (dx.abs() * 0.5).clamp(40.0 / z, 160.0 / z);
            let dir = if dx >= 0.0 { 1.0 } else { -1.0 };
            let c1 = Point::new(Px(from.x.0 + dir * ctrl), from.y);
            let c2 = Point::new(Px(to.x.0 - dir * ctrl), to.y);
            points[2] = c1;
            points[3] = c2;
            point_count = 4;
        }
        EdgeRouteKind::Step => {
            let mx = 0.5 * (from.x.0 + to.x.0);
            points[2] = Point::new(Px(mx), from.y);
            points[3] = Point::new(Px(mx), to.y);
            point_count = 4;
        }
        EdgeRouteKind::Straight => {}
    }

    let mut rect = bounds_from_points(&points[..point_count])?;
    let pad_screen = (blur_radius_screen_px.max(0.0) + 0.5 * width_screen_px.max(0.0)).max(0.0);
    let pad = pad_screen / z;
    rect = inflate_rect(rect, pad);
    Some(rect)
}

pub(super) fn glow_bounds_for_custom_path(
    commands: &[PathCommand],
    zoom: f32,
    width_screen_px: f32,
    blur_radius_screen_px: f32,
) -> Option<Rect> {
    let z = zoom.max(1.0e-6);
    let mut points: Vec<Point> = Vec::with_capacity(commands.len().saturating_mul(2));
    for command in commands {
        match *command {
            PathCommand::MoveTo(point) => points.push(point),
            PathCommand::LineTo(point) => points.push(point),
            PathCommand::QuadTo { ctrl, to } => {
                points.push(ctrl);
                points.push(to);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                points.push(ctrl1);
                points.push(ctrl2);
                points.push(to);
            }
            PathCommand::Close => {}
        }
    }
    let mut rect = bounds_from_points(&points)?;
    let pad_screen = (blur_radius_screen_px.max(0.0) + 0.5 * width_screen_px.max(0.0)).max(0.0);
    let pad = pad_screen / z;
    rect = inflate_rect(rect, pad);
    Some(rect)
}

fn bounds_from_points(points: &[Point]) -> Option<Rect> {
    if points.is_empty() {
        return None;
    }
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for point in points {
        if !point.x.0.is_finite() || !point.y.0.is_finite() {
            continue;
        }
        min_x = min_x.min(point.x.0);
        min_y = min_y.min(point.y.0);
        max_x = max_x.max(point.x.0);
        max_y = max_y.max(point.y.0);
    }
    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }
    Some(Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px((max_x - min_x).max(0.0)), Px((max_y - min_y).max(0.0))),
    ))
}

fn inflate_rect(rect: Rect, pad: f32) -> Rect {
    let pad = if pad.is_finite() && pad > 0.0 {
        pad
    } else {
        0.0
    };
    Rect::new(
        Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
        Size::new(
            Px(rect.size.width.0 + 2.0 * pad),
            Px(rect.size.height.0 + 2.0 * pad),
        ),
    )
}
