mod edge;
mod resize;

use fret_core::{CursorIcon, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn resolve_resize_handle_cursor<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> Option<CursorIcon> {
    resize::resolve_resize_handle_cursor(canvas, host, snapshot, position, zoom)
}

pub(super) fn resolve_edge_anchor_cursor<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> Option<CursorIcon> {
    edge::resolve_edge_anchor_cursor(canvas, snapshot, position)
}
