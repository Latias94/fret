mod state;

use fret_core::{Point, Rect};
use fret_ui::UiHost;

use super::super::LeftClickCx;
use crate::core::NodeId as GraphNodeId;
use crate::ui::canvas::state::{NodeResizeHandle, ViewSnapshot};
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_resize_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl LeftClickCx<H>,
    snapshot: &ViewSnapshot,
    position: Point,
    node: GraphNodeId,
    rect: Rect,
    handle: NodeResizeHandle,
    zoom: f32,
) {
    super::super::super::press_session::prepare_for_resize_hit(&mut canvas.interaction);

    if snapshot.interaction.elements_selectable {
        canvas.update_view_state(cx.left_click_host(), |s| {
            super::super::node_selection::apply_node_hit_selection(s, node)
        });
    }

    state::arm_pending_node_resize(canvas, cx, node, rect, handle, position, zoom);
}
