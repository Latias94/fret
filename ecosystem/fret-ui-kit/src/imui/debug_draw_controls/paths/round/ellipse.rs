use fret_core::{PathCommand, Point, Px, Size};

use super::super::super::DEFAULT_ELLIPSE_SEGMENTS;

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
