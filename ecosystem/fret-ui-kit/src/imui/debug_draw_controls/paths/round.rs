use fret_core::{PathCommand, Point, Px, Size};

use super::super::DEFAULT_ELLIPSE_SEGMENTS;

pub(in crate::imui::debug_draw_controls) fn circle_path(
    center: Point,
    radius: Px,
) -> [PathCommand; 6] {
    let r = radius.0;
    let k = 0.552_284_8_f32 * r;
    let cx = center.x.0;
    let cy = center.y.0;
    [
        PathCommand::MoveTo(Point::new(Px(cx + r), Px(cy))),
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + r), Px(cy + k)),
            ctrl2: Point::new(Px(cx + k), Px(cy + r)),
            to: Point::new(Px(cx), Px(cy + r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - k), Px(cy + r)),
            ctrl2: Point::new(Px(cx - r), Px(cy + k)),
            to: Point::new(Px(cx - r), Px(cy)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx - r), Px(cy - k)),
            ctrl2: Point::new(Px(cx - k), Px(cy - r)),
            to: Point::new(Px(cx), Px(cy - r)),
        },
        PathCommand::CubicTo {
            ctrl1: Point::new(Px(cx + k), Px(cy - r)),
            ctrl2: Point::new(Px(cx + r), Px(cy - k)),
            to: Point::new(Px(cx + r), Px(cy)),
        },
        PathCommand::Close,
    ]
}

pub(in crate::imui::debug_draw_controls) fn ngon_path(
    center: Point,
    radius: Px,
    segments: usize,
) -> Option<Vec<PathCommand>> {
    if segments < 3 || radius.0 <= 0.0 || !radius.0.is_finite() {
        return None;
    }

    let mut commands = Vec::with_capacity(segments.checked_add(1)?);
    for index in 0..segments {
        let angle = std::f32::consts::TAU * index as f32 / segments as f32;
        let (sin, cos) = angle.sin_cos();
        let point = Point::new(
            Px(center.x.0 + cos * radius.0),
            Px(center.y.0 + sin * radius.0),
        );
        if index == 0 {
            commands.push(PathCommand::MoveTo(point));
        } else {
            commands.push(PathCommand::LineTo(point));
        }
    }
    commands.push(PathCommand::Close);
    Some(commands)
}

pub(in crate::imui::debug_draw_controls) fn ellipse_path(
    center: Point,
    radius: Size,
    rotation_radians: f32,
    segments: usize,
) -> Option<Vec<PathCommand>> {
    let segments = if segments == 0 {
        DEFAULT_ELLIPSE_SEGMENTS
    } else {
        segments
    };
    if segments < 3
        || radius.width.0 <= 0.0
        || radius.height.0 <= 0.0
        || !radius.width.0.is_finite()
        || !radius.height.0.is_finite()
        || !rotation_radians.is_finite()
    {
        return None;
    }

    let (rot_sin, rot_cos) = rotation_radians.sin_cos();
    let mut commands = Vec::with_capacity(segments.checked_add(1)?);
    for index in 0..segments {
        let angle = std::f32::consts::TAU * index as f32 / segments as f32;
        let (angle_sin, angle_cos) = angle.sin_cos();
        let x = angle_cos * radius.width.0;
        let y = angle_sin * radius.height.0;
        let point = Point::new(
            Px(center.x.0 + x * rot_cos - y * rot_sin),
            Px(center.y.0 + x * rot_sin + y * rot_cos),
        );
        if index == 0 {
            commands.push(PathCommand::MoveTo(point));
        } else {
            commands.push(PathCommand::LineTo(point));
        }
    }
    commands.push(PathCommand::Close);
    Some(commands)
}
