use super::command_ui::finish_command_paint;
use super::*;

pub(super) fn cmd_create_group<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
) -> bool {
    let at = canvas.interaction.last_canvas_pos.unwrap_or_default();
    canvas.create_group_at(cx.app, cx.window, at);
    finish_command_paint(cx)
}

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

pub(super) fn cmd_group_rename<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    canvas.dismiss_command_transients();
    let Some(overlays) = canvas.overlays.clone() else {
        canvas.show_toast(
            cx.app,
            cx.window,
            DiagnosticSeverity::Info,
            "group rename overlay not configured",
        );
        return true;
    };
    let Some(group) = snapshot.selected_groups.last().copied() else {
        return true;
    };
    let invoked_at = canvas.command_invoked_at();
    let _ = overlays.update(cx.app, |state, _cx| {
        state.group_rename = Some(GroupRenameOverlay {
            group,
            invoked_at_window: invoked_at,
        });
    });
    finish_command_paint(cx)
}
