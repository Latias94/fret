mod edge_anchor;
mod port;

use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::LeftClickCx;
use crate::core::{EdgeId, PortId};
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_port_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl LeftClickCx<H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
    port: PortId,
) {
    port::handle_port_hit(canvas, cx, snapshot, position, modifiers, zoom, port)
}

pub(super) fn handle_edge_anchor_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl LeftClickCx<H>,
    snapshot: &ViewSnapshot,
    position: Point,
    edge: EdgeId,
    endpoint: EdgeEndpoint,
    fixed: PortId,
    multi_selection_pressed: bool,
) {
    edge_anchor::handle_edge_anchor_hit(
        canvas,
        cx,
        snapshot,
        position,
        edge,
        endpoint,
        fixed,
        multi_selection_pressed,
    )
}
