use super::*;

pub(super) fn sync_pointer_move_modifier_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    modifiers: fret_core::Modifiers,
) {
    super::super::event_pointer_move_state::sync_pointer_move_modifier_state(
        canvas, snapshot, modifiers,
    );
}

pub(super) fn seed_or_update_last_pointer_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::super::event_pointer_move_state::seed_or_update_last_pointer_state(
        canvas, position, modifiers,
    )
}
