mod capabilities;
mod drag;

use fret_core::{Point, Px, Rect};
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_node_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    node: GraphNodeId,
    rect: Rect,
    multi_selection_pressed: bool,
    zoom: f32,
) {
    super::super::super::press_session::prepare_for_node_hit(&mut canvas.interaction);
    let offset = Point::new(
        Px(position.x.0 - rect.origin.x.0),
        Px(position.y.0 - rect.origin.y.0),
    );
    let already_selected = snapshot.selected_nodes.iter().any(|id| *id == node);
    let hit_caps = capabilities::node_hit_capabilities(canvas, cx.app, snapshot, node);
    let select_action = super::super::node_selection::pending_node_select_action(
        hit_caps.selectable,
        multi_selection_pressed,
    );

    if hit_caps.selectable && !multi_selection_pressed {
        canvas.update_view_state(cx.app, |s| {
            super::super::node_selection::apply_node_hit_selection(s, node)
        });
    }

    let nodes_for_drag = drag::drag_nodes_for_hit(
        canvas,
        cx.app,
        snapshot,
        node,
        hit_caps.selectable,
        hit_caps.draggable,
        already_selected,
    );
    let header_hit = super::super::node_header_hit(
        rect,
        canvas.style.geometry.node_header_height,
        zoom,
        position,
    );
    let drag_enabled = drag::drag_enabled_for_node_hit(
        snapshot.interaction.node_drag_handle_mode,
        header_hit,
        hit_caps.draggable,
    );
    drag::arm_pending_node_drag(
        canvas,
        cx,
        node,
        nodes_for_drag,
        offset,
        position,
        select_action,
        drag_enabled,
    );
}
