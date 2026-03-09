use fret_core::Point;
use fret_ui::UiHost;

use crate::ui::canvas::state::{MarqueeDrag, ViewSnapshot};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn update_active_marquee<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool {
    let Some(mut marquee) = canvas.interaction.marquee.take() else {
        return false;
    };

    marquee.pos = position;
    let (selected_nodes, selected_edges) =
        super::marquee_selection_query::collect_marquee_selection(
            canvas,
            cx.app,
            snapshot,
            marquee.start_pos,
            marquee.pos,
        );

    canvas.interaction.marquee = Some(marquee);
    super::focus_session::clear_edge_focus(&mut canvas.interaction);
    super::marquee_selection_apply::apply_marquee_selection(
        canvas,
        cx.app,
        selected_nodes,
        selected_edges,
    );
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

pub(super) fn activate_pending_marquee<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    start_pos: Point,
    position: Point,
) -> bool {
    canvas.interaction.pending_marquee = None;
    let marquee = MarqueeDrag {
        start_pos,
        pos: position,
    };
    canvas.interaction.marquee = Some(marquee.clone());

    let (selected_nodes, selected_edges) =
        super::marquee_selection_query::collect_marquee_selection(
            canvas,
            cx.app,
            snapshot,
            marquee.start_pos,
            marquee.pos,
        );
    super::marquee_selection_apply::apply_marquee_selection(
        canvas,
        cx.app,
        selected_nodes,
        selected_edges,
    );
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}
