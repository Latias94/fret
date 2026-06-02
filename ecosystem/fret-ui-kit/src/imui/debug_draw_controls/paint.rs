use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::DebugDrawCommand;
use super::paint_shapes::paint_debug_draw_shape_command;

mod clip;
mod media;

pub(super) fn paint_debug_draw_commands(
    painter: &mut CanvasPainter<'_>,
    commands: &[DebugDrawCommand],
) {
    let scale = painter.scale_factor().max(1.0);
    let mut open_clip_depth = 0usize;
    for (index, command) in commands.iter().enumerate() {
        let order = DrawOrder(index as u32);
        let key = painter.key(&("fret-ui-kit.imui.debug_draw.command", index));
        if clip::paint_debug_draw_clip_command(painter, command, &mut open_clip_depth) {
            continue;
        }

        match command {
            DebugDrawCommand::Media(_) => {
                media::paint_debug_draw_media_command(painter, key, order, command);
            }
            DebugDrawCommand::Line { .. }
            | DebugDrawCommand::Polyline { .. }
            | DebugDrawCommand::ConvexPolyFilled { .. }
            | DebugDrawCommand::ConcavePolyFilled { .. }
            | DebugDrawCommand::Rect { .. }
            | DebugDrawCommand::RectFilled { .. }
            | DebugDrawCommand::RectFilledMultiColor { .. }
            | DebugDrawCommand::Quad { .. }
            | DebugDrawCommand::QuadFilled { .. }
            | DebugDrawCommand::Triangle { .. }
            | DebugDrawCommand::TriangleFilled { .. }
            | DebugDrawCommand::Mesh(_)
            | DebugDrawCommand::Circle { .. }
            | DebugDrawCommand::CircleFilled { .. }
            | DebugDrawCommand::Ngon { .. }
            | DebugDrawCommand::NgonFilled { .. }
            | DebugDrawCommand::Ellipse { .. }
            | DebugDrawCommand::EllipseFilled { .. }
            | DebugDrawCommand::BezierQuadratic { .. }
            | DebugDrawCommand::BezierCubic { .. }
            | DebugDrawCommand::Text { .. } => {
                paint_debug_draw_shape_command(painter, index, command, scale);
            }
            DebugDrawCommand::PushClipRect { .. } | DebugDrawCommand::PopClipRect => {}
        }
    }

    clip::close_debug_draw_clip_stack(painter, open_clip_depth);
}
