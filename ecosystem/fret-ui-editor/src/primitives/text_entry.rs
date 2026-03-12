//! Shared text-entry policy helpers for editor-owned text-like controls.
//!
//! This keeps light editor interaction policy out of individual controls:
//! - search-like fields can select all on focus without runtime changes,
//! - Escape behavior stays explicit and editor-owned,
//! - and `TextField` / `MiniSearchBox` do not need to hand-roll focus timers.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use fret_runtime::{CommandId, TimerToken};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTextCancelBehavior {
    #[default]
    None,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTextSelectionBehavior {
    #[default]
    PreserveSelection,
    SelectAllOnFocus,
}

#[derive(Debug, Default)]
pub(crate) struct EditorTextEntryFocusState {
    was_focused: bool,
    pending_select_all: bool,
    timer: Option<TimerToken>,
}

pub(crate) fn editor_text_entry_focus_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Arc<Mutex<EditorTextEntryFocusState>> {
    cx.with_state(
        || Arc::new(Mutex::new(EditorTextEntryFocusState::default())),
        |state| state.clone(),
    )
}

pub(crate) fn sync_editor_text_entry_focus_selection<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    focus_state: &Arc<Mutex<EditorTextEntryFocusState>>,
    input_id: GlobalElementId,
    is_focused: bool,
    has_text: bool,
    selection_behavior: EditorTextSelectionBehavior,
) {
    let (cancel_token, arm_token) = {
        let mut state = focus_state.lock().unwrap_or_else(|e| e.into_inner());

        let mut cancel_token = None;
        let mut arm_token = None;

        if is_focused && !state.was_focused {
            state.pending_select_all = matches!(
                selection_behavior,
                EditorTextSelectionBehavior::SelectAllOnFocus
            ) && has_text;
            if state.pending_select_all {
                let token = cx.app.next_timer_token();
                state.timer = Some(token);
                arm_token = Some(token);
            }
        } else if !is_focused {
            cancel_token = state.timer.take();
            state.pending_select_all = false;
        }

        state.was_focused = is_focused;
        (cancel_token, arm_token)
    };

    if let Some(token) = cancel_token {
        cx.cancel_timer(token);
    }
    if let Some(token) = arm_token {
        cx.set_timer_for(input_id, token, Duration::ZERO);
    }

    let focus_state_for_timer = focus_state.clone();
    cx.timer_on_timer_for(
        input_id,
        Arc::new(move |host, action_cx, token| {
            let mut state = focus_state_for_timer
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if state.timer != Some(token) {
                return false;
            }
            state.timer = None;
            if !state.pending_select_all {
                return false;
            }
            state.pending_select_all = false;
            host.dispatch_command(Some(action_cx.window), CommandId::from("edit.select_all"));
            host.request_redraw(action_cx.window);
            false
        }),
    );
}
