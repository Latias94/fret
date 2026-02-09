//! Combobox interaction semantics (Base UI shaped).
//!
//! This module is intentionally **outcome/state-machine** oriented:
//! - open/close reasons mapping
//! - callback gating helpers ("changed" vs "completed")
//! - value change gating (emit only on actual changes)
//! - reason-aware focus restore policies

use std::sync::Arc;
use std::sync::Mutex;

use crate::prelude::Model;
use fret_ui::action::{DismissReason, OnActivate, OnCloseAutoFocus, OnDismissRequest};
use fret_ui::elements::GlobalElementId;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComboboxCloseAutoFocusDecision {
    /// Do nothing and allow the primitive's default behavior.
    Default,
    /// Prevent the primitive's default behavior.
    PreventDefault,
    /// Restore focus to the combobox trigger (and prevent default).
    RestoreTrigger,
}

/// Reason-aware focus-restore policy for combobox-like overlays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComboboxCloseAutoFocusPolicy {
    pub on_item_press: ComboboxCloseAutoFocusDecision,
    pub on_escape: ComboboxCloseAutoFocusDecision,
    pub on_trigger_press: ComboboxCloseAutoFocusDecision,
    pub on_outside_press: ComboboxCloseAutoFocusDecision,
    pub on_focus_out: ComboboxCloseAutoFocusDecision,
    pub on_none: ComboboxCloseAutoFocusDecision,
}

impl Default for ComboboxCloseAutoFocusPolicy {
    fn default() -> Self {
        // shadcn/ui-like expectations:
        // - commit restores focus to trigger (asserted by diag gates)
        // - Escape restores focus to trigger
        // - outside press / focus-out should not steal focus back to trigger
        Self {
            on_item_press: ComboboxCloseAutoFocusDecision::RestoreTrigger,
            on_escape: ComboboxCloseAutoFocusDecision::RestoreTrigger,
            on_trigger_press: ComboboxCloseAutoFocusDecision::RestoreTrigger,
            on_outside_press: ComboboxCloseAutoFocusDecision::PreventDefault,
            on_focus_out: ComboboxCloseAutoFocusDecision::PreventDefault,
            on_none: ComboboxCloseAutoFocusDecision::Default,
        }
    }
}

pub fn close_auto_focus_decision_for_reason(
    policy: ComboboxCloseAutoFocusPolicy,
    reason: ComboboxOpenChangeReason,
) -> ComboboxCloseAutoFocusDecision {
    match reason {
        ComboboxOpenChangeReason::ItemPress => policy.on_item_press,
        ComboboxOpenChangeReason::EscapeKey => policy.on_escape,
        ComboboxOpenChangeReason::TriggerPress => policy.on_trigger_press,
        ComboboxOpenChangeReason::OutsidePress => policy.on_outside_press,
        ComboboxOpenChangeReason::FocusOut => policy.on_focus_out,
        ComboboxOpenChangeReason::None => policy.on_none,
    }
}

/// A small listbox policy helper: clear the query when transitioning from open -> closed.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClearQueryOnCloseState {
    was_open: bool,
}

pub fn should_clear_query_on_close(state: &mut ClearQueryOnCloseState, open: bool) -> bool {
    let should_clear = state.was_open && !open;
    state.was_open = open;
    should_clear
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

/// A selection-commit policy for Combobox (Base UI shaped, Fret semantics).
#[derive(Debug, Clone, Copy)]
pub struct SelectionCommitPolicy {
    /// If the user selects the already-selected item again, clear the value (`None`).
    pub toggle_selected_to_none: bool,
    /// Close the listbox after committing a selection.
    pub close_on_commit: bool,
    /// Clear the query after committing.
    pub clear_query_on_commit: bool,
}

impl Default for SelectionCommitPolicy {
    fn default() -> Self {
        Self {
            toggle_selected_to_none: true,
            close_on_commit: true,
            clear_query_on_commit: true,
        }
    }
}

pub fn set_open_change_reason_on_activate(
    open_change_reason: Model<Option<ComboboxOpenChangeReason>>,
    reason: ComboboxOpenChangeReason,
) -> OnActivate {
    Arc::new(move |host, action_cx, _activate_reason| {
        let _ = host
            .models_mut()
            .update(&open_change_reason, |v| *v = Some(reason));
        host.request_redraw(action_cx.window);
    })
}

pub fn set_open_change_reason_on_dismiss_request(
    open_change_reason: Model<Option<ComboboxOpenChangeReason>>,
) -> OnDismissRequest {
    Arc::new(move |host, action_cx, req| {
        let reason = open_change_reason_from_dismiss_reason(req.reason);
        let _ = host
            .models_mut()
            .update(&open_change_reason, |v| *v = Some(reason));
        host.request_redraw(action_cx.window);
    })
}

pub fn commit_selection_on_activate<T: Clone + PartialEq + 'static>(
    policy: SelectionCommitPolicy,
    value: Model<Option<T>>,
    open: Model<bool>,
    query: Model<String>,
    open_change_reason: Model<Option<ComboboxOpenChangeReason>>,
    selected_value: T,
) -> OnActivate {
    Arc::new(move |host, action_cx, _activate_reason| {
        let _ = host.models_mut().update(&value, |v| {
            if policy.toggle_selected_to_none
                && v.as_ref().is_some_and(|cur| cur == &selected_value)
            {
                *v = None;
            } else {
                *v = Some(selected_value.clone());
            }
        });
        let _ = host.models_mut().update(&open_change_reason, |v| {
            *v = Some(ComboboxOpenChangeReason::ItemPress);
        });
        if policy.close_on_commit {
            let _ = host.models_mut().update(&open, |v| *v = false);
        }
        if policy.clear_query_on_commit {
            let _ = host.models_mut().update(&query, |v| v.clear());
        }
        host.request_redraw(action_cx.window);
    })
}

pub fn on_close_auto_focus_with_reason(
    open_change_reason: Model<Option<ComboboxOpenChangeReason>>,
    trigger_id: Arc<Mutex<Option<GlobalElementId>>>,
    policy: ComboboxCloseAutoFocusPolicy,
) -> OnCloseAutoFocus {
    Arc::new(move |host, _action_cx, req| {
        let reason = host
            .models_mut()
            .get_copied(&open_change_reason)
            .unwrap_or(None)
            .unwrap_or(ComboboxOpenChangeReason::None);
        // Avoid leaking a stale reason across programmatic open/close.
        let _ = host.models_mut().update(&open_change_reason, |v| *v = None);

        match close_auto_focus_decision_for_reason(policy, reason) {
            ComboboxCloseAutoFocusDecision::Default => {}
            ComboboxCloseAutoFocusDecision::PreventDefault => {
                req.prevent_default();
            }
            ComboboxCloseAutoFocusDecision::RestoreTrigger => {
                req.prevent_default();
                let target = trigger_id.lock().unwrap_or_else(|e| e.into_inner()).clone();
                if let Some(target) = target {
                    host.request_focus(target);
                }
            }
        }
    })
}

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

    #[test]
    fn should_clear_query_on_close_emits_only_on_open_to_closed() {
        let mut state = ClearQueryOnCloseState::default();

        assert_eq!(should_clear_query_on_close(&mut state, false), false);
        assert_eq!(should_clear_query_on_close(&mut state, true), false);
        assert_eq!(should_clear_query_on_close(&mut state, true), false);
        assert_eq!(should_clear_query_on_close(&mut state, false), true);
        assert_eq!(should_clear_query_on_close(&mut state, false), false);
    }

    #[test]
    fn close_auto_focus_decision_maps_reasons() {
        let policy = ComboboxCloseAutoFocusPolicy {
            on_item_press: ComboboxCloseAutoFocusDecision::RestoreTrigger,
            on_escape: ComboboxCloseAutoFocusDecision::RestoreTrigger,
            on_trigger_press: ComboboxCloseAutoFocusDecision::RestoreTrigger,
            on_outside_press: ComboboxCloseAutoFocusDecision::PreventDefault,
            on_focus_out: ComboboxCloseAutoFocusDecision::PreventDefault,
            on_none: ComboboxCloseAutoFocusDecision::Default,
        };

        assert_eq!(
            close_auto_focus_decision_for_reason(policy, ComboboxOpenChangeReason::ItemPress),
            ComboboxCloseAutoFocusDecision::RestoreTrigger
        );
        assert_eq!(
            close_auto_focus_decision_for_reason(policy, ComboboxOpenChangeReason::EscapeKey),
            ComboboxCloseAutoFocusDecision::RestoreTrigger
        );
        assert_eq!(
            close_auto_focus_decision_for_reason(policy, ComboboxOpenChangeReason::OutsidePress),
            ComboboxCloseAutoFocusDecision::PreventDefault
        );
        assert_eq!(
            close_auto_focus_decision_for_reason(policy, ComboboxOpenChangeReason::FocusOut),
            ComboboxCloseAutoFocusDecision::PreventDefault
        );
        assert_eq!(
            close_auto_focus_decision_for_reason(policy, ComboboxOpenChangeReason::None),
            ComboboxCloseAutoFocusDecision::Default
        );
    }
}
