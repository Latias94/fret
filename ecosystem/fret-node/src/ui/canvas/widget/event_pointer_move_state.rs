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
mod tests {
    use super::*;

    #[test]
    fn multi_selection_active_respects_modifier_key_policy() {
        assert!(multi_selection_active(
            NodeGraphModifierKey::None,
            fret_core::Modifiers::default()
        ));
        assert!(multi_selection_active(
            NodeGraphModifierKey::CtrlOrMeta,
            fret_core::Modifiers {
                ctrl: true,
                ..fret_core::Modifiers::default()
            }
        ));
        assert!(multi_selection_active(
            NodeGraphModifierKey::Alt,
            fret_core::Modifiers {
                alt_gr: true,
                ..fret_core::Modifiers::default()
            }
        ));
        assert!(!multi_selection_active(
            NodeGraphModifierKey::Shift,
            fret_core::Modifiers::default()
        ));
    }
}
