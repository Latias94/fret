use fret_ui::UiHost;

use crate::ui::canvas::widget::*;

pub(in super::super) fn handle_group_resize_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: pointer_up_commit_cx::PointerUpCommitCx<H>,
{
    let Some(resize) = super::super::super::pointer_up_session::take_active_release(
        &mut canvas.interaction.group_resize,
        &mut canvas.interaction.pending_group_resize,
    ) else {
        return false;
    };

    let ops = super::super::super::pointer_up_commit_resize::build_group_resize_ops(&resize);
    if !ops.is_empty() {
        let window = cx.window();
        let _ = canvas.commit_ops(cx.host(), window, Some("Resize Group"), ops);
    }

    super::super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}
