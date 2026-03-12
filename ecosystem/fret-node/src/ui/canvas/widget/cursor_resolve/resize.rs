use fret_core::{CursorIcon, Point};
use fret_ui::UiHost;

use crate::ui::canvas::state::{NodeResizeHandle, ViewSnapshot};
use crate::ui::canvas::widget::*;

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

fn cursor_icon_for_resize_handle(handle: NodeResizeHandle) -> CursorIcon {
    match handle {
        NodeResizeHandle::Top | NodeResizeHandle::Bottom => CursorIcon::RowResize,
        NodeResizeHandle::Left | NodeResizeHandle::Right => CursorIcon::ColResize,
        _ => CursorIcon::ColResize,
    }
}

#[cfg(test)]
mod tests;
