use fret_core::{PathCommand, Point, Px, Rect, Size};

use crate::ui::presenter::EdgeRouteKind;

use super::wire_ctrl_points;

pub(super) fn path_bounds_rect(commands: &[PathCommand]) -> Option<Rect> {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    let mut saw_any = false;
    let mut subpath_start: Option<Point> = None;

    let mut push_point = |point: Point| {
        if !point.x.0.is_finite() || !point.y.0.is_finite() {
            return;
        }
        saw_any = true;
        min_x = min_x.min(point.x.0);
        min_y = min_y.min(point.y.0);
        max_x = max_x.max(point.x.0);
        max_y = max_y.max(point.y.0);
    };

    for command in commands {
        match *command {
            PathCommand::MoveTo(point) => {
                subpath_start = Some(point);
                push_point(point);
            }
            PathCommand::LineTo(point) => push_point(point),
            PathCommand::QuadTo { ctrl, to } => {
                push_point(ctrl);
                push_point(to);
            }
            PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                push_point(ctrl1);
                push_point(ctrl2);
                push_point(to);
            }
            PathCommand::Close => {
                if let Some(point) = subpath_start {
                    push_point(point);
                }
            }
        }
    }

    if !saw_any
        || !min_x.is_finite()
        || !min_y.is_finite()
        || !max_x.is_finite()
        || !max_y.is_finite()
        || min_x > max_x
        || min_y > max_y
    {
        return None;
    }

    Some(Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px(max_x - min_x), Px(max_y - min_y)),
    ))
}

pub(super) fn edge_bounds_rect(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    pad: f32,
) -> Rect {
    let mut min_x = from.x.0.min(to.x.0);
    let mut min_y = from.y.0.min(to.y.0);
    let mut max_x = from.x.0.max(to.x.0);
    let mut max_y = from.y.0.max(to.y.0);

    if route == EdgeRouteKind::Bezier {
        let (ctrl1, ctrl2) = wire_ctrl_points(from, to, zoom);
        min_x = min_x.min(ctrl1.x.0).min(ctrl2.x.0);
        min_y = min_y.min(ctrl1.y.0).min(ctrl2.y.0);
        max_x = max_x.max(ctrl1.x.0).max(ctrl2.x.0);
        max_y = max_y.max(ctrl1.y.0).max(ctrl2.y.0);
    }

    let pad = normalized_pad(pad);
    Rect::new(
        Point::new(Px(min_x - pad), Px(min_y - pad)),
        Size::new(
            Px((max_x - min_x) + 2.0 * pad),
            Px((max_y - min_y) + 2.0 * pad),
        ),
    )
}

fn normalized_pad(pad: f32) -> f32 {
    if pad.is_finite() { pad.max(0.0) } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_bounds_rect_includes_close_back_to_subpath_start() {
        let bounds = path_bounds_rect(&[
            PathCommand::MoveTo(Point::new(Px(10.0), Px(20.0))),
            PathCommand::LineTo(Point::new(Px(30.0), Px(25.0))),
            PathCommand::Close,
        ])
        .expect("expected bounds");

        assert_eq!(
            bounds,
            Rect::new(Point::new(Px(10.0), Px(20.0)), Size::new(Px(20.0), Px(5.0)))
        );
    }
}
