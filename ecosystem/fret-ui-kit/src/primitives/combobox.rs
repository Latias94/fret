//! Combobox interaction semantics (Base UI shaped).
//!
//! This module is intentionally **outcome/state-machine** oriented:
//! - open/close reasons mapping
//! - callback gating helpers ("changed" vs "completed")
//! - value change gating (emit only on actual changes)

use std::sync::Arc;

use fret_ui::action::DismissReason;

/// Open-change reasons aligned with Base UI combobox semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComboboxOpenChangeReason {
    TriggerPress,
    OutsidePress,
    ItemPress,
    EscapeKey,
    FocusOut,
    None,
}

pub fn open_change_reason_from_dismiss_reason(reason: DismissReason) -> ComboboxOpenChangeReason {
    match reason {
        DismissReason::Escape => ComboboxOpenChangeReason::EscapeKey,
        DismissReason::OutsidePress { .. } => ComboboxOpenChangeReason::OutsidePress,
        DismissReason::FocusOutside => ComboboxOpenChangeReason::FocusOut,
        DismissReason::Scroll => ComboboxOpenChangeReason::None,
    }
}

/// Tracks open-change callbacks so we can emit:
/// - `changed` immediately on open-state change
/// - `completed` only once presence has settled and any motion is done
#[derive(Debug, Default, Clone)]
pub struct OpenChangeCallbackState {
    initialized: bool,
    last_open: bool,
    pending_complete: Option<bool>,
}

pub fn open_change_events(
    state: &mut OpenChangeCallbackState,
    open: bool,
    present: bool,
    animating: bool,
) -> (Option<bool>, Option<bool>) {
    let mut changed = None;
    let mut completed = None;

    if !state.initialized {
        state.initialized = true;
        state.last_open = open;
    } else if state.last_open != open {
        state.last_open = open;
        state.pending_complete = Some(open);
        changed = Some(open);
    }

    if state.pending_complete == Some(open) && present == open && !animating {
        state.pending_complete = None;
        completed = Some(open);
    }

    (changed, completed)
}

/// Tracks value changes for `onValueChange` so we don't emit the initial value or repeats.
#[derive(Debug, Default, Clone)]
pub struct ValueChangeCallbackState<T> {
    initialized: bool,
    last_value: Option<T>,
}

pub fn value_change_event<T: Clone + PartialEq>(
    state: &mut ValueChangeCallbackState<T>,
    value: Option<T>,
) -> Option<Option<T>> {
    if !state.initialized {
        state.initialized = true;
        state.last_value = value;
        return None;
    }

    if state.last_value != value {
        state.last_value = value.clone();
        return Some(value);
    }

    None
}

pub type OnOpenChange = Arc<dyn Fn(bool) + Send + Sync + 'static>;
pub type OnOpenChangeWithReason =
    Arc<dyn Fn(bool, ComboboxOpenChangeReason) + Send + Sync + 'static>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_change_events_emit_change_and_complete_after_settle() {
        let mut state = OpenChangeCallbackState::default();

        let (changed, completed) = open_change_events(&mut state, false, false, false);
        assert_eq!((changed, completed), (None, None));

        let (changed, completed) = open_change_events(&mut state, true, true, true);
        assert_eq!((changed, completed), (Some(true), None));

        let (changed, completed) = open_change_events(&mut state, true, true, false);
        assert_eq!((changed, completed), (None, Some(true)));

        let (changed, completed) = open_change_events(&mut state, false, true, true);
        assert_eq!((changed, completed), (Some(false), None));

        let (changed, completed) = open_change_events(&mut state, false, false, false);
        assert_eq!((changed, completed), (None, Some(false)));
    }

    #[test]
    fn open_change_events_complete_without_animation() {
        let mut state = OpenChangeCallbackState::default();

        let _ = open_change_events(&mut state, false, false, false);
        let (changed, completed) = open_change_events(&mut state, true, true, false);
        assert_eq!((changed, completed), (Some(true), Some(true)));

        let (changed, completed) = open_change_events(&mut state, false, false, false);
        assert_eq!((changed, completed), (Some(false), Some(false)));
    }

    #[test]
    fn open_change_reason_maps_dismiss_reasons() {
        assert_eq!(
            open_change_reason_from_dismiss_reason(DismissReason::Escape),
            ComboboxOpenChangeReason::EscapeKey
        );
        assert_eq!(
            open_change_reason_from_dismiss_reason(DismissReason::OutsidePress { pointer: None }),
            ComboboxOpenChangeReason::OutsidePress
        );
        assert_eq!(
            open_change_reason_from_dismiss_reason(DismissReason::FocusOutside),
            ComboboxOpenChangeReason::FocusOut
        );
        assert_eq!(
            open_change_reason_from_dismiss_reason(DismissReason::Scroll),
            ComboboxOpenChangeReason::None
        );
    }

    #[test]
    fn value_change_event_emits_only_on_state_change() {
        let mut state: ValueChangeCallbackState<Arc<str>> = ValueChangeCallbackState::default();

        let changed = value_change_event(&mut state, None);
        assert_eq!(changed, None);

        let changed = value_change_event(&mut state, Some(Arc::from("beta")));
        assert_eq!(changed, Some(Some(Arc::from("beta"))));

        let changed = value_change_event(&mut state, Some(Arc::from("beta")));
        assert_eq!(changed, None);

        let changed = value_change_event(&mut state, Some(Arc::from("alpha")));
        assert_eq!(changed, Some(Some(Arc::from("alpha"))));

        let changed = value_change_event(&mut state, None);
        assert_eq!(changed, Some(None));
    }
}
