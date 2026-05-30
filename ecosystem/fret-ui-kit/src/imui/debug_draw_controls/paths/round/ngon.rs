use fret_core::{PathCommand, Point, Px};

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
