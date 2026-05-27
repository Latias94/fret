use fret_ui::UiHost;

use crate::ui::canvas::widget::*;

pub(in super::super) fn handle_node_resize_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: pointer_up_commit_cx::PointerUpCommitCx<H>,
{
    let Some(resize) = super::super::super::pointer_up_session::take_active_release(
        &mut canvas.interaction.node_resize,
        &mut canvas.interaction.pending_node_resize,
    ) else {
        return false;
    };

    let ops = canvas
        .mirrors
        .graph
        .read_ref(cx.host(), |graph| {
            super::super::super::pointer_up_commit_resize::build_node_resize_ops(&resize, graph)
        })
        .ok()
        .unwrap_or_default();
    if !ops.is_empty() {
        let window = cx.window();
        let _ = canvas.commit_ops(cx.host(), window, Some("Resize Node"), ops);
    }

    super::super::super::pointer_up_finish::finish_pointer_up(cx);
    true
}
