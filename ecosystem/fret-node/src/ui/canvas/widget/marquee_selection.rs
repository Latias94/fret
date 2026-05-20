use fret_core::Point;
use fret_ui::UiHost;

use crate::ui::canvas::state::{MarqueeDrag, ViewSnapshot};

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, marquee_cx::MarqueeCx};

pub(super) fn update_active_marquee<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: MarqueeCx<H>,
{
    let Some(mut marquee) = canvas.interaction.marquee.take() else {
        return false;
    };

    marquee.pos = position;
    let (selected_nodes, selected_edges) =
        super::marquee_selection_query::collect_marquee_selection(
            canvas,
            cx.host(),
            snapshot,
            marquee.start_pos,
            marquee.pos,
        );

    canvas.interaction.marquee = Some(marquee);
    super::focus_session::clear_edge_focus(&mut canvas.interaction);
    super::marquee_selection_apply::apply_marquee_selection(
        canvas,
        cx.host(),
        selected_nodes,
        selected_edges,
    );
    super::paint_invalidation::invalidate_paint(cx);
    true
}

pub(super) fn activate_pending_marquee<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    start_pos: Point,
    position: Point,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: MarqueeCx<H>,
{
    canvas.interaction.pending_marquee = None;
    let marquee = MarqueeDrag {
        start_pos,
        pos: position,
    };
    canvas.interaction.marquee = Some(marquee.clone());

    let (selected_nodes, selected_edges) =
        super::marquee_selection_query::collect_marquee_selection(
            canvas,
            cx.host(),
            snapshot,
            marquee.start_pos,
            marquee.pos,
        );
    super::marquee_selection_apply::apply_marquee_selection(
        canvas,
        cx.host(),
        selected_nodes,
        selected_edges,
    );
    super::paint_invalidation::invalidate_paint(cx);
    true
}
