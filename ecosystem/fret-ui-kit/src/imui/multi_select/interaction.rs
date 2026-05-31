use fret_core::Modifiers;

use super::ImUiMultiSelectState;

pub(in crate::imui::multi_select) fn apply_click<K: Clone + PartialEq>(
    state: &mut ImUiMultiSelectState<K>,
    all_keys: &[K],
    key: &K,
    modifiers: Modifiers,
) -> bool {
    let previous = state.clone();

    if modifiers.shift {
        state.range_select_from_anchor_or_single(all_keys, key);
    } else if primary_modifier_down(modifiers) {
        state.toggle_in_order(all_keys, key);
    } else {
        state.replace_with_single(key);
    }

    previous != *state
}

fn primary_modifier_down(modifiers: Modifiers) -> bool {
    modifiers.ctrl || modifiers.meta
}
