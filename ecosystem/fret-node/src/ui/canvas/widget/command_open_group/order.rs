use super::super::*;
use super::finish_command_paint;

pub(super) fn cmd_group_bring_to_front<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    canvas.dismiss_command_transients();
    let groups = snapshot.selected_groups.clone();
    if groups.is_empty() {
        return true;
    }
    canvas.update_view_state(cx.app, |state| {
        group_draw_order::bring_selected_groups_to_front_in_view_state(state, &groups);
    });
    finish_command_paint(cx)
}

pub(super) fn cmd_group_send_to_back<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    canvas.dismiss_command_transients();
    let groups = snapshot.selected_groups.clone();
    if groups.is_empty() {
        return true;
    }
    canvas.update_view_state(cx.app, |state| {
        group_draw_order::send_selected_groups_to_back_in_view_state(state, &groups);
    });
    finish_command_paint(cx)
}
