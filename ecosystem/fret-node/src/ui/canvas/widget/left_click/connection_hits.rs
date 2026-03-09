use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use crate::core::{EdgeId, PortId};
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::state::{PendingWireDrag, ViewSnapshot, WireDragKind};
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_port_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
    port: PortId,
) {
    super::super::focus_session::clear_edge_focus(&mut canvas.interaction);
    let port_base_connectable = canvas
        .graph
        .read_ref(cx.app, |graph| {
            NodeGraphCanvasWith::<M>::port_is_connectable_base(graph, &snapshot.interaction, port)
        })
        .ok()
        .unwrap_or(false);
    let port_connectable_start = port_base_connectable
        && canvas
            .graph
            .read_ref(cx.app, |graph| {
                NodeGraphCanvasWith::<M>::port_is_connectable_start(
                    graph,
                    &snapshot.interaction,
                    port,
                )
            })
            .ok()
            .unwrap_or(false);
    let port_connectable_end = port_base_connectable
        && canvas
            .graph
            .read_ref(cx.app, |graph| {
                NodeGraphCanvasWith::<M>::port_is_connectable_end(
                    graph,
                    &snapshot.interaction,
                    port,
                )
            })
            .ok()
            .unwrap_or(false);

    if snapshot.interaction.connect_on_click
        && canvas.interaction.click_connect
        && canvas.interaction.wire_drag.is_some()
    {
        if !port_connectable_end {
            return;
        }
        if let Some(mut w) = canvas.interaction.wire_drag.take() {
            w.pos = position;
            canvas.interaction.wire_drag = Some(w);
            canvas.interaction.click_connect = false;
            canvas.interaction.pending_wire_drag = None;
            let _ = crate::ui::canvas::widget::wire_drag::handle_wire_left_up_with_forced_target(
                canvas,
                cx,
                snapshot,
                zoom,
                Some(port),
            );
            return;
        }
    }

    if !port_base_connectable {
        canvas.interaction.click_connect = false;
        return;
    }

    super::super::press_session::prepare_for_port_hit(&mut canvas.interaction);
    let yank = (modifiers.ctrl || modifiers.meta)
        .then(|| canvas.yank_reconnectable_edges_from_port(cx.app, &snapshot.interaction, port));

    let kind = match yank {
        Some(edges) if edges.len() > 1 => WireDragKind::ReconnectMany { edges },
        Some(mut edges) if edges.len() == 1 => {
            let (edge, endpoint, fixed) = edges.remove(0);
            WireDragKind::Reconnect {
                edge,
                endpoint,
                fixed,
            }
        }
        _ => WireDragKind::New {
            from: port,
            bundle: vec![port],
        },
    };

    if matches!(kind, WireDragKind::New { .. }) && !port_connectable_start {
        return;
    }

    canvas.interaction.pending_wire_drag = Some(PendingWireDrag {
        kind,
        start_pos: position,
    });
    cx.capture_pointer(cx.node);
    super::super::paint_invalidation::invalidate_paint(cx);
}

pub(super) fn handle_edge_anchor_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    edge: EdgeId,
    endpoint: EdgeEndpoint,
    fixed: PortId,
    multi_selection_pressed: bool,
) {
    let edge_selectable = canvas
        .graph
        .read_ref(cx.app, |g| {
            NodeGraphCanvasWith::<M>::edge_is_selectable(g, &snapshot.interaction, edge)
        })
        .ok()
        .unwrap_or(false);

    super::super::press_session::prepare_for_edge_anchor_hit(&mut canvas.interaction);
    canvas.interaction.focused_edge =
        (snapshot.interaction.edges_focusable && edge_selectable).then_some(edge);

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

    canvas.interaction.pending_wire_drag = Some(PendingWireDrag {
        kind: WireDragKind::Reconnect {
            edge,
            endpoint,
            fixed,
        },
        start_pos: position,
    });
    cx.capture_pointer(cx.node);
    super::super::paint_invalidation::invalidate_paint(cx);
}
