mod overlay;

use super::command_ui::finish_command_paint;
use super::*;

pub(super) fn cmd_open_conversion_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    overlay::cmd_open_conversion_picker(canvas, cx, snapshot)
}
