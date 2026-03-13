use super::*;
use crate::interaction::NodeGraphModifierKey;

pub(super) fn sync_pointer_wheel_modifier_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    modifiers: fret_core::Modifiers,
) {
    canvas.interaction.last_modifiers = modifiers;
    canvas.interaction.multi_selection_active =
        multi_selection_active(snapshot.interaction.multi_selection_key, modifiers);
}

fn multi_selection_active(key: NodeGraphModifierKey, modifiers: fret_core::Modifiers) -> bool {
    key.is_pressed(modifiers)
}

#[cfg(test)]
mod tests;
