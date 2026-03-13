mod drag;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_edge_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    edge: crate::core::EdgeId,
    multi_selection_pressed: bool,
) {
    super::super::super::press_session::prepare_for_edge_hit(&mut canvas.interaction);
    let edge_selectable =
        super::super::edge_selection::edge_is_selectable(canvas, cx.app, snapshot, edge);
    if edge_selectable {
        canvas.update_view_state(cx.app, |s| {
            super::super::edge_selection::apply_edge_selection(s, edge, multi_selection_pressed)
        });
    }
    canvas.interaction.focused_edge = super::super::edge_selection::focused_edge_after_hit(
        snapshot.interaction.edges_focusable,
        edge_selectable,
        edge,
    );

    drag::arm_edge_hit_drag(
        canvas,
        cx,
        snapshot.interaction.edge_insert_on_alt_drag,
        modifiers,
        edge,
        position,
    );
}
