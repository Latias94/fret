//! Shared text-entry policy helpers for editor-owned numeric controls.
//!
//! This keeps the editor baseline in one place:
//! - typed entry arms a "replace current value" mode on initial focus by default,
//! - Escape/Enter handling can stay control-local,
//! - and wrappers such as `DragValue` / `Slider` / `AxisDragValue` do not need to
//!   re-derive the same focus-entry rules independently.

use std::sync::{Arc, Mutex};

use fret_runtime::Model;
use fret_ui::action::{ActionCx, KeyDownCx, UiFocusActionHost};

mod focus;
mod replace;

pub use focus::NumericInputSelectionBehavior;
pub(crate) use focus::{
    NumericTextEntryFocusHandoffState, NumericTextEntryFocusState,
    arm_numeric_text_entry_focus_handoff, clear_numeric_error_when_draft_changes,
    numeric_text_entry_focus_state, sync_numeric_text_entry_focus,
    sync_numeric_text_entry_focus_handoff,
};
use replace::{NumericReplacementPlan, replacement_plan};

#[cfg(test)]
mod tests;

pub(crate) fn handle_numeric_text_entry_replace_key(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    down: KeyDownCx,
    focus_state: &Arc<Mutex<NumericTextEntryFocusState>>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
) -> Option<bool> {
    let plan = {
        let mut state = focus_state.lock().unwrap_or_else(|e| e.into_inner());
        if !state.replace_on_next_edit {
            return None;
        }

        let plan = replacement_plan(down);
        if !matches!(plan, NumericReplacementPlan::Ignore) {
            state.replace_on_next_edit = false;
        }
        plan
    };

    match plan {
        NumericReplacementPlan::Ignore | NumericReplacementPlan::Disarm => None,
        NumericReplacementPlan::ClearAndContinue => {
            clear_numeric_text_entry(host, draft, error);
            host.request_redraw(action_cx.window);
            Some(false)
        }
        NumericReplacementPlan::ClearAndConsume => {
            clear_numeric_text_entry(host, draft, error);
            host.request_redraw(action_cx.window);
            Some(true)
        }
    }
}

fn clear_numeric_text_entry(
    host: &mut dyn UiFocusActionHost,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
) {
    let _ = host.models_mut().update(draft, |text| text.clear());
    let _ = host.models_mut().update(error, |value| *value = None);
}
