use fret_ui::UiHost;

use super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, pointer_up_commit_cx::PointerUpCommitCx,
};

pub(in super::super) fn handle_group_drag_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: PointerUpCommitCx<H>,
{
    let Some(drag) = super::super::pointer_up_session::take_active_release(
        &mut canvas.interaction.group_drag,
        &mut canvas.interaction.pending_group_drag,
    ) else {
        return false;
    };

    let ops = super::super::pointer_up_commit_group_drag::build_group_drag_ops(&drag);
    if !ops.is_empty() {
        let window = cx.window();
        let _ = canvas.commit_ops(cx.host(), window, Some("Move Group"), ops);
    }

    super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}
