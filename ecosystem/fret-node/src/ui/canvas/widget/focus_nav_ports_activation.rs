mod commit;
mod preflight;
mod start;

use super::*;

pub(super) fn activate_focused_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let Some(target) = preflight::activation_target(canvas, cx.app, snapshot) else {
        return false;
    };

    if canvas.interaction.wire_drag.is_none() {
        start::arm_click_connect_wire_drag(canvas, target.port, target.position);
        return true;
    }

    commit::commit_click_connect_wire_drag(canvas, cx, snapshot, target.port, target.position);
    true
}
