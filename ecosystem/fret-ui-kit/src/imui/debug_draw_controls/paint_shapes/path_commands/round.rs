use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::super::super::DebugDrawCommand;
use super::super::paths;

pub(super) fn paint_round_path_shape_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
    scale: f32,
) -> bool {
    match command {
        DebugDrawCommand::Circle {
            center,
            radius,
            color,
            style,
        } => {
            paths::paint_circle(painter, key, order, *center, *radius, *color, *style, scale);
            true
        }
        DebugDrawCommand::CircleFilled {
            center,
            radius,
            color,
        } => {
            paths::paint_circle_filled(painter, key, order, *center, *radius, *color, scale);
            true
        }
        DebugDrawCommand::Ngon {
            center,
            radius,
            segments,
            color,
            style,
        } => {
            paths::paint_ngon(
                painter, key, order, *center, *radius, *segments, *color, *style, scale,
            );
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
        DebugDrawCommand::Ellipse {
            center,
            radius,
            rotation_radians,
            segments,
            color,
            style,
        } => {
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
