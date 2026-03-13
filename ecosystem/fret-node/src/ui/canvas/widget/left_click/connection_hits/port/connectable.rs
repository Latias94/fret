use fret_ui::UiHost;

use crate::core::PortId;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct PortConnectability {
    pub(super) base: bool,
    pub(super) start: bool,
    pub(super) end: bool,
}

pub(super) fn port_connectability<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    port: PortId,
) -> PortConnectability {
    canvas
        .graph
        .read_ref(host, |graph| {
            let base = NodeGraphCanvasWith::<M>::port_is_connectable_base(
                graph,
                &snapshot.interaction,
                port,
            );
            PortConnectability {
                base,
                start: base
                    && NodeGraphCanvasWith::<M>::port_is_connectable_start(
                        graph,
                        &snapshot.interaction,
                        port,
                    ),
                end: base
                    && NodeGraphCanvasWith::<M>::port_is_connectable_end(
                        graph,
                        &snapshot.interaction,
                        port,
                    ),
            }
        })
        .ok()
        .unwrap_or_default()
}
