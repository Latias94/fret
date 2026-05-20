use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, marquee_cx::MarqueeCx};

pub(super) fn handle_left_up<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: MarqueeCx<H>,
{
    if canvas.interaction.marquee.take().is_some() {
        canvas.interaction.pending_marquee = None;
        super::pointer_up_session::finish_pointer_up_with_snap_guide_cleanup(
            &mut canvas.interaction,
            cx,
        );
        return true;
    }

    if let Some(pending) = canvas.interaction.pending_marquee.take() {
        if pending.clear_selection_on_up {
            canvas.update_view_state(cx.host(), |state| {
                state.selected_nodes.clear();
                state.selected_edges.clear();
                state.selected_groups.clear();
            });
        }
        super::pointer_up_session::finish_pointer_up_with_snap_guide_cleanup(
            &mut canvas.interaction,
            cx,
        );
        return true;
    }

    false
}
