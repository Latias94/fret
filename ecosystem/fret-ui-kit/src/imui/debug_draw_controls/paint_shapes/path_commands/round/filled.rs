use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::super::super::DebugDrawCommand;
use super::super::super::paths;

pub(super) fn paint_filled_round_path_shape_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
    scale: f32,
) -> bool {
    match command {
        DebugDrawCommand::CircleFilled {
            center,
            radius,
            color,
        } => {
            paths::paint_circle_filled(painter, key, order, *center, *radius, *color, scale);
            true
        }
        DebugDrawCommand::NgonFilled {
            center,
            radius,
            segments,
            color,
        } => {
            paths::paint_ngon_filled(
                painter, key, order, *center, *radius, *segments, *color, scale,
            );
            true
        }
        DebugDrawCommand::EllipseFilled {
            center,
            radius,
            rotation_radians,
            segments,
            color,
        } => {
            paths::paint_ellipse_filled(
                painter,
                key,
                order,
                *center,
                *radius,
                *rotation_radians,
                *segments,
                *color,
                scale,
            );
            true
        }
        _ => false,
    }
}
