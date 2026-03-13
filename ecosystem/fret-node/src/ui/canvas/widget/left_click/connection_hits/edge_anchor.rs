mod arm;

use fret_core::Point;
use fret_ui::UiHost;

use crate::core::{EdgeId, PortId};
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_edge_anchor_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    edge: EdgeId,
    endpoint: EdgeEndpoint,
    fixed: PortId,
    multi_selection_pressed: bool,
) {
    let edge_selectable =
        super::super::edge_selection::edge_is_selectable(canvas, cx.app, snapshot, edge);

    super::super::super::press_session::prepare_for_edge_anchor_hit(&mut canvas.interaction);
    canvas.interaction.focused_edge = super::super::edge_selection::focused_edge_after_hit(
        snapshot.interaction.edges_focusable,
        edge_selectable,
        edge,
    );

    if edge_selectable {
        canvas.update_view_state(cx.app, |s| {
            super::super::edge_selection::apply_edge_selection(s, edge, multi_selection_pressed)
        });
    }

    arm::arm_edge_anchor_reconnect(canvas, cx, edge, endpoint, fixed, position);
}
