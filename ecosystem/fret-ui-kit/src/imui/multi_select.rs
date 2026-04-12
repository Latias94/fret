//! Immediate multi-select collection helpers.

use std::sync::Arc;

use fret_core::Modifiers;
use fret_runtime::Model;
use fret_ui::{ElementContext, Invalidation, UiHost};

use super::{ResponseExt, SelectableOptions, UiWriterImUiFacadeExt};

/// Model state for an immediate multi-select collection.
///
/// This is intentionally small:
/// - `selected` stores the currently selected keys,
/// - `anchor` stores the range-selection anchor used for shift-click expansion.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImUiMultiSelectState<K> {
    pub selected: Vec<K>,
    pub anchor: Option<K>,
}

impl<K: PartialEq> ImUiMultiSelectState<K> {
    pub fn is_selected(&self, key: &K) -> bool {
        self.selected.iter().any(|item| item == key)
    }
}

/// Returns a controllable selection model for an immediate multi-select collection.
pub fn multi_select_use_model<H: UiHost, K: Clone + 'static>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<ImUiMultiSelectState<K>>>,
    default_value: impl FnOnce() -> ImUiMultiSelectState<K>,
) -> crate::primitives::controllable_state::ControllableModel<ImUiMultiSelectState<K>> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

pub(super) fn multi_selectable_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    K: Clone + PartialEq + 'static,
>(
    ui: &mut W,
    label: Arc<str>,
    model: &Model<ImUiMultiSelectState<K>>,
    all_keys: &[K],
    key: K,
    options: SelectableOptions,
) -> ResponseExt {
    let model = model.clone();
    let key_for_read = key.clone();
    let selected = ui.with_cx_mut(|cx| {
        cx.read_model(&model, Invalidation::Paint, |_app, state| {
            state.is_selected(&key_for_read)
        })
        .unwrap_or(false)
    });

    let mut response = ui.selectable_with_options(
        label,
        SelectableOptions {
            selected,
            ..options
        },
    );

    if response.clicked() {
        let modifiers = response.pointer_click_modifiers().unwrap_or_default();
        let mut changed = false;
        let _ = ui.with_cx_mut(|cx| {
            cx.app.models_mut().update(&model, |state| {
                changed = apply_click(state, all_keys, &key, modifiers);
            })
        });
        response.core.changed = changed;
    }

    response
}

fn apply_click<K: Clone + PartialEq>(
    state: &mut ImUiMultiSelectState<K>,
    all_keys: &[K],
    key: &K,
    modifiers: Modifiers,
) -> bool {
    let previous = state.clone();

    if modifiers.shift {
        apply_range_click(state, all_keys, key);
    } else if primary_modifier_down(modifiers) {
        apply_toggle_click(state, all_keys, key);
    } else {
        state.selected = vec![key.clone()];
        state.anchor = Some(key.clone());
    }

    previous != *state
}

fn apply_range_click<K: Clone + PartialEq>(
    state: &mut ImUiMultiSelectState<K>,
    all_keys: &[K],
    key: &K,
) {
    let anchor = state.anchor.clone().unwrap_or_else(|| key.clone());
    let Some(anchor_index) = all_keys.iter().position(|item| item == &anchor) else {
        state.selected = vec![key.clone()];
        state.anchor = Some(key.clone());
        return;
    };
    let Some(key_index) = all_keys.iter().position(|item| item == key) else {
        state.selected = vec![key.clone()];
        state.anchor = Some(key.clone());
        return;
    };

    let (start, end) = if anchor_index <= key_index {
        (anchor_index, key_index)
    } else {
        (key_index, anchor_index)
    };
    state.selected = all_keys[start..=end].to_vec();
    state.anchor = Some(anchor);
}

fn apply_toggle_click<K: Clone + PartialEq>(
    state: &mut ImUiMultiSelectState<K>,
    all_keys: &[K],
    key: &K,
) {
    let mut selected = state.selected.clone();
    if let Some(index) = selected.iter().position(|item| item == key) {
        selected.remove(index);
    } else {
        selected.push(key.clone());
    }
    state.selected = normalize_selection_order(all_keys, selected);
    state.anchor = Some(key.clone());
}

fn normalize_selection_order<K: Clone + PartialEq>(all_keys: &[K], selected: Vec<K>) -> Vec<K> {
    let mut ordered = Vec::new();

    for key in all_keys {
        if selected.iter().any(|item| item == key) && !ordered.iter().any(|item| item == key) {
            ordered.push(key.clone());
        }
    }

    for key in selected {
        if !ordered.iter().any(|item| item == &key) {
            ordered.push(key);
        }
    }

    ordered
}

fn primary_modifier_down(modifiers: Modifiers) -> bool {
    modifiers.ctrl || modifiers.meta
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> Vec<Arc<str>> {
        vec![
            Arc::from("alpha"),
            Arc::from("beta"),
            Arc::from("gamma"),
            Arc::from("delta"),
        ]
    }

    #[test]
    fn plain_click_replaces_selection_and_resets_anchor() {
        let keys = keys();
        let mut state = ImUiMultiSelectState {
            selected: vec![keys[0].clone(), keys[2].clone()],
            anchor: Some(keys[2].clone()),
        };

        let changed = apply_click(&mut state, &keys, &keys[1], Modifiers::default());

        assert!(changed);
        assert_eq!(state.selected, vec![keys[1].clone()]);
        assert_eq!(state.anchor, Some(keys[1].clone()));
    }

    #[test]
    fn primary_modifier_click_toggles_membership_in_collection_order() {
        let keys = keys();
        let mut state = ImUiMultiSelectState {
            selected: vec![keys[0].clone(), keys[2].clone()],
            anchor: Some(keys[2].clone()),
        };

        let changed = apply_click(
            &mut state,
            &keys,
            &keys[1],
            Modifiers {
                meta: true,
                ..Default::default()
            },
        );

        assert!(changed);
        assert_eq!(
            state.selected,
            vec![keys[0].clone(), keys[1].clone(), keys[2].clone()]
        );
        assert_eq!(state.anchor, Some(keys[1].clone()));
    }

    #[test]
    fn shift_click_selects_range_from_anchor_without_moving_anchor() {
        let keys = keys();
        let mut state = ImUiMultiSelectState {
            selected: vec![keys[1].clone()],
            anchor: Some(keys[1].clone()),
        };

        let changed = apply_click(
            &mut state,
            &keys,
            &keys[3],
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );

        assert!(changed);
        assert_eq!(
            state.selected,
            vec![keys[1].clone(), keys[2].clone(), keys[3].clone()]
        );
        assert_eq!(state.anchor, Some(keys[1].clone()));
    }

    #[test]
    fn shift_click_without_anchor_falls_back_to_single_select() {
        let keys = keys();
        let mut state = ImUiMultiSelectState::<Arc<str>>::default();

        let changed = apply_click(
            &mut state,
            &keys,
            &keys[2],
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );

        assert!(changed);
        assert_eq!(state.selected, vec![keys[2].clone()]);
        assert_eq!(state.anchor, Some(keys[2].clone()));
    }
}
