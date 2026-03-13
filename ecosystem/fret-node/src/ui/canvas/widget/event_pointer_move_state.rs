use super::*;
use crate::interaction::NodeGraphModifierKey;

pub(super) fn sync_pointer_move_modifier_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    modifiers: fret_core::Modifiers,
) {
    canvas.interaction.last_modifiers = modifiers;
    canvas.interaction.multi_selection_active =
        multi_selection_active(snapshot.interaction.multi_selection_key, modifiers);
}

pub(super) fn seed_or_update_last_pointer_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
    modifiers: fret_core::Modifiers,
) -> bool {
    super::pointer_move_pointer_state::seed_or_update_last_pointer_state(
        canvas, position, modifiers,
    )
}

fn multi_selection_active(key: NodeGraphModifierKey, modifiers: fret_core::Modifiers) -> bool {
    key.is_pressed(modifiers)
}

#[cfg(test)]
mod tests;
