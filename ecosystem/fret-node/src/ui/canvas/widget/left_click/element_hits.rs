use fret_core::{Modifiers, Point, Px, Rect};
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;
use crate::interaction::NodeGraphDragHandleMode;
use crate::ui::canvas::state::{
    EdgeDrag, NodeResizeHandle, PendingEdgeInsertDrag, PendingNodeDrag, PendingNodeResize,
    PendingNodeSelectAction, ViewSnapshot,
};
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_resize_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    node: GraphNodeId,
    rect: Rect,
    handle: NodeResizeHandle,
    zoom: f32,
) {
    super::super::press_session::prepare_for_resize_hit(&mut canvas.interaction);

    if snapshot.interaction.elements_selectable {
        canvas.update_view_state(cx.app, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            if !s.selected_nodes.iter().any(|id| *id == node) {
                s.selected_nodes.clear();
                s.selected_nodes.push(node);
            }
            s.draw_order.retain(|id| *id != node);
            s.draw_order.push(node);
        });
    }

    let start_size = crate::core::CanvasSize {
        width: rect.size.width.0 * zoom,
        height: rect.size.height.0 * zoom,
    };
    let start_size_opt = canvas
        .graph
        .read_ref(cx.app, |g| g.nodes.get(&node).and_then(|n| n.size))
        .ok()
        .flatten();
    let start_node_pos = canvas
        .graph
        .read_ref(cx.app, |g| g.nodes.get(&node).map(|n| n.pos))
        .ok()
        .flatten()
        .unwrap_or_default();

    canvas.interaction.pending_node_resize = Some(PendingNodeResize {
        node,
        handle,
        start_pos: position,
        start_node_pos,
        start_size,
        start_size_opt,
    });
    cx.capture_pointer(cx.node);
    super::super::paint_invalidation::invalidate_paint(cx);
}

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
    super::super::press_session::prepare_for_node_hit(&mut canvas.interaction);
    let offset = Point::new(
        Px(position.x.0 - rect.origin.x.0),
        Px(position.y.0 - rect.origin.y.0),
    );
    let already_selected = snapshot.selected_nodes.iter().any(|id| *id == node);
    let node_selectable = canvas
        .graph
        .read_ref(cx.app, |g| {
            NodeGraphCanvasWith::<M>::node_is_selectable(g, &snapshot.interaction, node)
        })
        .ok()
        .unwrap_or(false);
    let node_draggable = canvas
        .graph
        .read_ref(cx.app, |g| {
            NodeGraphCanvasWith::<M>::node_is_draggable(g, &snapshot.interaction, node)
        })
        .ok()
        .unwrap_or(false);
    let select_action = if node_selectable && multi_selection_pressed {
        PendingNodeSelectAction::Toggle
    } else {
        PendingNodeSelectAction::None
    };

    if node_selectable && !multi_selection_pressed {
        canvas.update_view_state(cx.app, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            if !s.selected_nodes.iter().any(|id| *id == node) {
                s.selected_nodes.clear();
                s.selected_nodes.push(node);
            }
            s.draw_order.retain(|id| *id != node);
            s.draw_order.push(node);
        });
    }

    let nodes_for_drag = if node_draggable
        && node_selectable
        && already_selected
        && snapshot.selected_nodes.len() > 1
    {
        snapshot.selected_nodes.clone()
    } else {
        vec![node]
    };
    let nodes_for_drag = canvas
        .graph
        .read_ref(cx.app, |g| {
            nodes_for_drag
                .iter()
                .copied()
                .filter(|id| {
                    NodeGraphCanvasWith::<M>::node_is_draggable(g, &snapshot.interaction, *id)
                })
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_else(|| vec![node]);
    let drag_enabled = match snapshot.interaction.node_drag_handle_mode {
        NodeGraphDragHandleMode::Any => true,
        NodeGraphDragHandleMode::Header => super::node_header_hit(
            rect,
            canvas.style.geometry.node_header_height,
            zoom,
            position,
        ),
    };
    let drag_enabled = drag_enabled && node_draggable;
    canvas.interaction.pending_node_drag = Some(PendingNodeDrag {
        primary: node,
        nodes: nodes_for_drag,
        grab_offset: offset,
        start_pos: position,
        select_action,
        drag_enabled,
    });
    cx.capture_pointer(cx.node);

    super::super::paint_invalidation::invalidate_paint(cx);
}

pub(super) fn handle_edge_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    edge: crate::core::EdgeId,
    multi_selection_pressed: bool,
) {
    super::super::press_session::prepare_for_edge_hit(&mut canvas.interaction);
    let edge_selectable = canvas
        .graph
        .read_ref(cx.app, |g| {
            NodeGraphCanvasWith::<M>::edge_is_selectable(g, &snapshot.interaction, edge)
        })
        .ok()
        .unwrap_or(false);
    if edge_selectable {
        canvas.update_view_state(cx.app, |s| {
            if multi_selection_pressed {
                if let Some(ix) = s.selected_edges.iter().position(|id| *id == edge) {
                    s.selected_edges.remove(ix);
                } else {
                    s.selected_edges.push(edge);
                }
            } else {
                s.selected_nodes.clear();
                s.selected_groups.clear();
                s.selected_edges.clear();
                s.selected_edges.push(edge);
            }
        });
    }
    canvas.interaction.focused_edge =
        (snapshot.interaction.edges_focusable && edge_selectable).then_some(edge);

    if snapshot.interaction.edge_insert_on_alt_drag && (modifiers.alt || modifiers.alt_gr) {
        canvas.interaction.pending_edge_insert_drag = Some(PendingEdgeInsertDrag {
            edge,
            start_pos: position,
        });
        canvas.interaction.edge_insert_drag = None;
        canvas.interaction.edge_drag = None;
    } else {
        canvas.interaction.pending_edge_insert_drag = None;
        canvas.interaction.edge_insert_drag = None;
        canvas.interaction.edge_drag = Some(EdgeDrag {
            edge,
            start_pos: position,
        });
    }
    cx.capture_pointer(cx.node);
    super::super::paint_invalidation::invalidate_paint(cx);
}
