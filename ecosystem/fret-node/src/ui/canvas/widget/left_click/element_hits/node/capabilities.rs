use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct NodeHitCapabilities {
    pub(super) selectable: bool,
    pub(super) draggable: bool,
}

pub(super) fn node_hit_capabilities<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    node: GraphNodeId,
) -> NodeHitCapabilities {
    canvas
        .graph
        .read_ref(host, |graph| NodeHitCapabilities {
            selectable: NodeGraphCanvasWith::<M>::node_is_selectable(
                graph,
                &snapshot.interaction,
                node,
            ),
            draggable: NodeGraphCanvasWith::<M>::node_is_draggable(
                graph,
                &snapshot.interaction,
                node,
            ),
        })
        .ok()
        .unwrap_or_default()
}
