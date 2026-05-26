//! Immediate multi-select collection helpers.

mod state;

use std::sync::Arc;

use fret_core::Modifiers;
use fret_runtime::Model;
use fret_ui::{ElementContext, Invalidation, UiHost};

use super::{ResponseExt, SelectableOptions, UiWriterImUiFacadeExt};

pub use state::ImUiMultiSelectState;

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
        response.set_core_changed(changed);
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

#[cfg(test)]
mod tests;
