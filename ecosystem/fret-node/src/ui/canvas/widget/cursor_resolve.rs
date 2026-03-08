use fret_core::{CursorIcon, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{NodeResizeHandle, ViewSnapshot};

pub(super) fn resolve_resize_handle_cursor<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> Option<CursorIcon> {
    let geom = canvas.canvas_geometry(&*host, snapshot);
    let presenter = &*canvas.presenter;
    let style = &canvas.style;
    canvas
        .graph
        .read_ref(host, |graph| {
            for node_id in &snapshot.selected_nodes {
                let Some(node_geom) = geom.nodes.get(node_id) else {
                    continue;
                };
                let handles = presenter.node_resize_handles(graph, *node_id, style);
                for handle in NodeResizeHandle::ALL {
                    if !handles.contains(handle) {
                        continue;
                    }
                    let rect = canvas.node_resize_handle_rect(node_geom.rect, handle, zoom);
                    if NodeGraphCanvasWith::<M>::rect_contains(rect, position) {
                        return Some(cursor_icon_for_resize_handle(handle));
                    }
                }
            }
            None
        })
        .ok()
        .flatten()
}

pub(super) fn resolve_edge_anchor_cursor<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    _position: Point,
) -> Option<CursorIcon> {
    let target_edge = canvas
        .interaction
        .focused_edge
        .or_else(|| (snapshot.selected_edges.len() == 1).then(|| snapshot.selected_edges[0]));
    let edge_id = target_edge?;
    canvas
        .interaction
        .hover_edge_anchor
        .is_some_and(|(id, _)| id == edge_id)
        .then_some(CursorIcon::Pointer)
}

fn cursor_icon_for_resize_handle(handle: NodeResizeHandle) -> CursorIcon {
    match handle {
        NodeResizeHandle::Top | NodeResizeHandle::Bottom => CursorIcon::RowResize,
        NodeResizeHandle::Left | NodeResizeHandle::Right => CursorIcon::ColResize,
        _ => CursorIcon::ColResize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_handle_cursor_icon_matches_axis_handles() {
        assert_eq!(
            cursor_icon_for_resize_handle(NodeResizeHandle::Top),
            CursorIcon::RowResize
        );
        assert_eq!(
            cursor_icon_for_resize_handle(NodeResizeHandle::Bottom),
            CursorIcon::RowResize
        );
        assert_eq!(
            cursor_icon_for_resize_handle(NodeResizeHandle::Left),
            CursorIcon::ColResize
        );
        assert_eq!(
            cursor_icon_for_resize_handle(NodeResizeHandle::Right),
            CursorIcon::ColResize
        );
    }
}
