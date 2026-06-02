use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::super::super::{DebugDrawCommand, DebugDrawRoundCommand};
use super::super::super::paths;

pub(super) fn paint_stroked_round_path_shape_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
    scale: f32,
) -> bool {
    match command {
        DebugDrawCommand::Round(DebugDrawRoundCommand::Circle {
            center,
            radius,
            color,
            style,
        }) => {
            paths::paint_circle(painter, key, order, *center, *radius, *color, *style, scale);
            true
        }
        DebugDrawCommand::Round(DebugDrawRoundCommand::Ngon {
            center,
            radius,
            segments,
            color,
            style,
        }) => {
            paths::paint_ngon(
                painter, key, order, *center, *radius, *segments, *color, *style, scale,
            );
            true
        }
        DebugDrawCommand::Round(DebugDrawRoundCommand::Ellipse {
            center,
            radius,
            rotation_radians,
            segments,
            color,
            style,
        }) => {
            paths::paint_ellipse(
                painter,
                key,
                order,
                *center,
                *radius,
                *rotation_radians,
                *segments,
                *color,
                *style,
                scale,
            );
            true
        }
        _ => false,
    }
}
