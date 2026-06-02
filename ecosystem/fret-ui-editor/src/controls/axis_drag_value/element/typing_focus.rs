//! AxisDragValue typing focus lifecycle owner.

use std::sync::{Arc, Mutex};

use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::primitives::numeric_text_entry::{
    NumericInputSelectionBehavior, NumericTextEntryFocusHandoffState, NumericTextEntryFocusState,
    clear_numeric_error_when_draft_changes, sync_numeric_text_entry_focus,
    sync_numeric_text_entry_focus_handoff,
};

use super::super::model::{AxisDragValueMode, AxisDragValueState};

pub(super) struct AxisDragValueTypingFocusArgs {
    pub(super) state: Arc<Mutex<AxisDragValueState>>,
    pub(super) focus_state: Arc<Mutex<NumericTextEntryFocusState>>,
    pub(super) focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) last_draft_text: Arc<Mutex<String>>,
    pub(super) value_text: Arc<str>,
    pub(super) typing: bool,
    pub(super) input_id: GlobalElementId,
    pub(super) is_focused: bool,
    pub(super) selection_behavior: NumericInputSelectionBehavior,
}

pub(super) fn axis_drag_value_sync_typing_focus<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: AxisDragValueTypingFocusArgs,
) {
    let AxisDragValueTypingFocusArgs {
        state,
        focus_state,
        focus_handoff,
        draft,
        error,
        last_draft_text,
        value_text,
        typing,
        input_id,
        is_focused,
        selection_behavior,
    } = args;

    if typing {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        if is_focused {
            st.seen_input_focus = true;
        } else if st.seen_input_focus {
            st.mode = AxisDragValueMode::Scrub;
        }
    }

    sync_numeric_text_entry_focus(
        cx,
        &focus_state,
        is_focused,
        &value_text,
        &draft,
        &error,
        selection_behavior,
    );
    sync_numeric_text_entry_focus_handoff(
        cx,
        input_id,
        &focus_handoff,
        typing,
        input_id,
        is_focused,
    );

    if !is_focused {
        let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
        *last = value_text.as_ref().to_string();
    }
}

pub(super) fn axis_drag_value_clear_typing_error_when_draft_changes<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    is_focused: bool,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    last_draft_text: &Arc<Mutex<String>>,
) {
    clear_numeric_error_when_draft_changes(cx, is_focused, draft, error, last_draft_text);
}
