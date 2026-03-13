use super::super::*;

pub(super) fn emit_move_start<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    kind: ViewportMoveKind,
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    callbacks.on_move_start(ViewportMoveStart {
        kind,
        pan: snapshot.pan,
        zoom: snapshot.zoom,
    });
}

pub(super) fn emit_move_end<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    kind: ViewportMoveKind,
    outcome: ViewportMoveEndOutcome,
) {
    let Some(callbacks) = canvas.callbacks.as_mut() else {
        return;
    };
    callbacks.on_move_end(ViewportMoveEnd {
        kind,
        pan: snapshot.pan,
        zoom: snapshot.zoom,
        outcome,
    });
}
