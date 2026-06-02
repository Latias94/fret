use fret_ui::canvas::CanvasPainter;

use super::super::DebugDrawCommand;
use super::super::geometry::rect_is_empty;

pub(super) fn paint_debug_draw_clip_command(
    painter: &mut CanvasPainter<'_>,
    command: &DebugDrawCommand,
    open_clip_depth: &mut usize,
) -> bool {
    match command {
        DebugDrawCommand::PushClipRect { rect } => {
            if rect_is_empty(*rect) {
                return true;
            }
            painter
                .scene()
                .push(fret_core::SceneOp::PushClipRect { rect: *rect });
            *open_clip_depth += 1;
            true
        }
        DebugDrawCommand::PopClipRect => {
            if *open_clip_depth == 0 {
                return true;
            }
            painter.scene().push(fret_core::SceneOp::PopClip);
            *open_clip_depth -= 1;
            true
        }
        _ => false,
    }
}

pub(super) fn close_debug_draw_clip_stack(painter: &mut CanvasPainter<'_>, open_clip_depth: usize) {
    for _ in 0..open_clip_depth {
        painter.scene().push(fret_core::SceneOp::PopClip);
    }
}
