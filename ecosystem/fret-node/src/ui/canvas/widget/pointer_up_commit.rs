mod group_drag;
mod resize;

use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) use group_drag::handle_group_drag_release;
pub(super) use resize::{handle_group_resize_release, handle_node_resize_release};

pub(super) fn handle_node_drag_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    super::pointer_up_node_drag::handle_node_drag_release(canvas, cx, snapshot)
}
