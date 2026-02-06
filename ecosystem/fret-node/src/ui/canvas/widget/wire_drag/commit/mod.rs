use super::super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use super::commit_cx::WireCommitCx;

mod new_wire;
mod prelude;
mod reconnect;
mod reconnect_many;

use prelude::*;

pub(in super::super) fn handle_wire_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WireCommitCx<H>,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    handle_wire_left_up_with_forced_target(canvas, cx, snapshot, zoom, None)
}

pub(in super::super) fn handle_wire_left_up_with_forced_target<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WireCommitCx<H>,
    snapshot: &ViewSnapshot,
    zoom: f32,
    forced_target: Option<PortId>,
) -> bool {
    let Some(w) = canvas.interaction.wire_drag.take() else {
        return false;
    };
    let kind_for_callbacks = w.kind.clone();

    let bounds = cx.bounds(canvas.interaction.last_bounds);

    let (from_port, require_from_connectable_start) = match &w.kind {
        WireDragKind::New { from, .. } => (Some(*from), true),
        WireDragKind::Reconnect { fixed, .. } => (Some(*fixed), false),
        WireDragKind::ReconnectMany { edges } => (edges.first().map(|e| e.2), false),
    };

    let from_port_connectable = from_port
        .map(|port| {
            if !require_from_connectable_start {
                return true;
            }
            canvas
                .graph
                .read_ref(cx.host(), |graph| {
                    NodeGraphCanvasWith::<M>::port_is_connectable_start(
                        graph,
                        &snapshot.interaction,
                        port,
                    )
                })
                .ok()
                .unwrap_or(false)
        })
        .unwrap_or(false);
    let forced_target = forced_target.filter(|port| {
        canvas
            .graph
            .read_ref(cx.host(), |graph| {
                NodeGraphCanvasWith::<M>::port_is_connectable_end(
                    graph,
                    &snapshot.interaction,
                    *port,
                )
            })
            .ok()
            .unwrap_or(false)
    });
    let target = forced_target.or_else(|| {
        from_port.and_then(|from_port| {
            if !from_port_connectable {
                return None;
            }
            let (geom, index) = canvas.canvas_derived(&*cx.host(), snapshot);
            let this = &*canvas;
            let index = index.clone();
            this.graph
                .read_ref(cx.host(), |graph| {
                    let mut scratch = HitTestScratch::default();
                    let mut ctx =
                        HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
                    this.pick_target_port(
                        graph,
                        snapshot,
                        &mut ctx,
                        from_port,
                        require_from_connectable_start,
                        w.pos,
                    )
                })
                .ok()
                .flatten()
        })
    });
    canvas.interaction.hover_port = None;
    canvas.interaction.hover_port_valid = false;
    canvas.interaction.hover_port_convertible = false;
    canvas.interaction.hover_port_diagnostic = None;

    let pos = w.pos;
    let CommitEmit {
        target: connect_end_target,
        outcome: connect_end_outcome,
    } = match w.kind {
        WireDragKind::New { from, bundle } => new_wire::commit_new_wire(
            canvas, cx, snapshot, zoom, bounds, pos, from, bundle, target,
        ),
        WireDragKind::Reconnect {
            edge,
            endpoint,
            fixed: _fixed,
        } => reconnect::commit_reconnect(canvas, cx, snapshot, edge, endpoint, target),
        WireDragKind::ReconnectMany { edges } => {
            reconnect_many::commit_reconnect_many(canvas, cx, snapshot, edges, target)
        }
    };

    canvas.emit_connect_end(
        snapshot.interaction.connection_mode,
        &kind_for_callbacks,
        connect_end_target,
        connect_end_outcome,
    );

    cx.release_pointer_capture();
    cx.request_redraw();
    cx.invalidate_paint();

    true
}
