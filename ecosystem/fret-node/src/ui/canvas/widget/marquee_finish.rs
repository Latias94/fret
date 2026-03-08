use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.marquee.take().is_some() {
        canvas.interaction.pending_marquee = None;
        canvas.interaction.snap_guides = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    if let Some(pending) = canvas.interaction.pending_marquee.take() {
        if pending.clear_selection_on_up {
            canvas.update_view_state(cx.app, |state| {
                state.selected_nodes.clear();
                state.selected_edges.clear();
                state.selected_groups.clear();
            });
        }
        canvas.interaction.snap_guides = None;
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    false
}
