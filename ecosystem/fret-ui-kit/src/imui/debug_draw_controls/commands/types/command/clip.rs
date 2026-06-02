use fret_core::Rect;

// This file owns clip-stack debug-draw command payload variants.

#[derive(Debug, Clone)]
pub(in crate::imui::debug_draw_controls) enum DebugDrawClipCommand {
    PushClipRect { rect: Rect },
    PopClipRect,
}
