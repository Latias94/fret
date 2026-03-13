use super::*;

pub(super) fn sync_pointer_up_modifier_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    modifiers: fret_core::Modifiers,
) {
    canvas.interaction.last_modifiers = modifiers;
    canvas.interaction.multi_selection_active = snapshot
        .interaction
        .multi_selection_key
        .is_pressed(modifiers);
}
