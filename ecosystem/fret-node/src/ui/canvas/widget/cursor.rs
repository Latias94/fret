use fret_core::{CursorIcon, Point};
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{NodeResizeHandle, ViewSnapshot};

pub(super) fn update_cursors<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    update_close_button_cursor(canvas, cx, snapshot, position, zoom);
    update_resize_handle_cursor(canvas, cx, snapshot, position, zoom);
    update_edge_anchor_cursor(canvas, cx, snapshot, position, zoom);
}

fn update_close_button_cursor<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    if canvas.close_command.is_none()
        || canvas.interaction.node_drag.is_some()
        || canvas.interaction.node_resize.is_some()
        || canvas.interaction.wire_drag.is_some()
        || canvas.interaction.pending_edge_insert_drag.is_some()
        || canvas.interaction.edge_insert_drag.is_some()
        || canvas.interaction.edge_drag.is_some()
        || canvas.interaction.panning
    {
        return;
    }

    let rect = NodeGraphCanvasWith::<M>::close_button_rect(snapshot.pan, zoom);
    if NodeGraphCanvasWith::<M>::rect_contains(rect, position) {
        cx.set_cursor_icon(CursorIcon::Pointer);
    }
}

fn update_resize_handle_cursor<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) {
    if canvas.interaction.node_drag.is_some()
        || canvas.interaction.node_resize.is_some()
        || canvas.interaction.wire_drag.is_some()
        || canvas.interaction.pending_edge_insert_drag.is_some()
        || canvas.interaction.edge_insert_drag.is_some()
        || canvas.interaction.edge_drag.is_some()
        || canvas.interaction.panning
        || canvas.interaction.marquee.is_some()
        || canvas.interaction.context_menu.is_some()
        || canvas.interaction.searcher.is_some()
    {
        return;
    }

    if snapshot.selected_nodes.is_empty() {
        return;
    }

    let geom = canvas.canvas_geometry(&*cx.app, snapshot);
    let presenter = &*canvas.presenter;
    let style = &canvas.style;
    let icon = canvas
        .graph
        .read_ref(cx.app, |graph| {
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
                        return Some(match handle {
                            NodeResizeHandle::Top | NodeResizeHandle::Bottom => {
                                CursorIcon::RowResize
                            }
                            NodeResizeHandle::Left | NodeResizeHandle::Right => {
                                CursorIcon::ColResize
                            }
                            _ => CursorIcon::ColResize,
                        });
                    }
                }
            }
            None
        })
        .ok()
        .flatten();
    if let Some(icon) = icon {
        cx.set_cursor_icon(icon);
    }
}

fn update_edge_anchor_cursor<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    _position: Point,
    _zoom: f32,
) {
    if canvas.interaction.node_drag.is_some()
        || canvas.interaction.node_resize.is_some()
        || canvas.interaction.wire_drag.is_some()
        || canvas.interaction.pending_edge_insert_drag.is_some()
        || canvas.interaction.edge_insert_drag.is_some()
        || canvas.interaction.edge_drag.is_some()
        || canvas.interaction.panning
        || canvas.interaction.marquee.is_some()
        || canvas.interaction.context_menu.is_some()
        || canvas.interaction.searcher.is_some()
    {
        return;
    }

    let target_edge = canvas
        .interaction
        .focused_edge
        .or_else(|| (snapshot.selected_edges.len() == 1).then(|| snapshot.selected_edges[0]));
    let Some(edge_id) = target_edge else {
        return;
    };

    if canvas
        .interaction
        .hover_edge_anchor
        .is_some_and(|(id, _)| id == edge_id)
    {
        cx.set_cursor_icon(CursorIcon::Pointer);
    }
}
