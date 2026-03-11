mod fallback;

use super::command_ui::finish_command_paint;
use super::*;

pub(super) fn cmd_open_insert_node<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let at = canvas
        .interaction
        .last_canvas_pos
        .or_else(|| {
            fallback::insert_picker_fallback_canvas_point::<M>(
                snapshot,
                canvas.interaction.last_bounds,
            )
        })
        .unwrap_or_default();
    canvas.open_insert_node_picker(cx.app, at);
    finish_command_paint(cx)
}
