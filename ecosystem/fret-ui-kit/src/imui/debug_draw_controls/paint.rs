use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::DebugDrawCommand;
use super::geometry::rect_is_empty;
use super::paint_shapes::paint_debug_draw_shape_command;

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
        match command {
            DebugDrawCommand::PushClipRect { rect } => {
                if rect_is_empty(*rect) {
                    continue;
                }
                painter
                    .scene()
                    .push(fret_core::SceneOp::PushClipRect { rect: *rect });
                open_clip_depth += 1;
            }
            DebugDrawCommand::PopClipRect => {
                if open_clip_depth == 0 {
                    continue;
                }
                painter.scene().push(fret_core::SceneOp::PopClip);
                open_clip_depth -= 1;
            }
            DebugDrawCommand::Image { .. }
            | DebugDrawCommand::ImageRegion { .. }
            | DebugDrawCommand::ImageQuad { .. }
            | DebugDrawCommand::ImageRounded { .. }
            | DebugDrawCommand::ImageRegionRounded { .. }
            | DebugDrawCommand::SvgImage { .. }
            | DebugDrawCommand::SvgMaskIcon { .. } => {
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
            | DebugDrawCommand::TriangleMesh { .. }
            | DebugDrawCommand::ImageTriangleMesh { .. }
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
        }
    }

    for _ in 0..open_clip_depth {
        painter.scene().push(fret_core::SceneOp::PopClip);
    }
}
