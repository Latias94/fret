use super::super::*;
use super::finish_command_paint;
use crate::ui::overlays::open_group_rename_session;

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
        open_group_rename_session(
            state,
            GroupRenameOverlay {
                group,
                invoked_at_window: invoked_at,
            },
        );
    });
    finish_command_paint(cx)
}
