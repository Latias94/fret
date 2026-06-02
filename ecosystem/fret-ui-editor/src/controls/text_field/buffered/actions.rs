//! Buffered TextField commit/cancel action finalizers.

use std::sync::{Arc, Mutex};

use fret_runtime::{CommandId, Model};
use fret_ui::action::{ActionCx, UiActionHost, UiFocusActionHost};

use super::super::{OnTextFieldOutcome, TextFieldOutcome};
use super::BufferedTextFieldState;

fn clear_buffered_text_field_pending_blur(state: &mut BufferedTextFieldState) {
    state.blur_timer = None;
    state.pending_blur = None;
}

pub(in crate::controls::text_field) fn clear_buffered_text_field_state(
    state: &mut BufferedTextFieldState,
) {
    state.was_focused = false;
    clear_buffered_text_field_pending_blur(state);
    let _ = state.session.commit();
}

pub(in crate::controls::text_field) fn commit_buffered_text_field(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    model: &Model<String>,
    draft: &Model<String>,
    buffered_state: &Arc<Mutex<BufferedTextFieldState>>,
    on_outcome: Option<&OnTextFieldOutcome>,
    submit_command: Option<&CommandId>,
) -> bool {
    let next = host.models_mut().get_cloned(draft).unwrap_or_default();
    let should_emit_outcome = {
        let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
        let changed = state.session.changed_from(&next);
        clear_buffered_text_field_pending_blur(&mut state);
        let _ = state.session.commit();
        changed
    };

    {
        let next_for_update = next.clone();
        let _ = host.models_mut().update(model, |text| {
            if text.as_str() != next_for_update.as_str() {
                *text = next_for_update.clone();
            }
        });
    }
    if should_emit_outcome && let Some(cb) = on_outcome {
        cb(host, action_cx, TextFieldOutcome::Committed);
    }
    if let Some(command) = submit_command {
        host.dispatch_command(Some(action_cx.window), command.clone());
    }
    host.request_redraw(action_cx.window);
    true
}

pub(in crate::controls::text_field) fn commit_buffered_text_field_from_controller(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    model: &Model<String>,
    draft: &Model<String>,
    buffered_state: &Arc<Mutex<BufferedTextFieldState>>,
    submit_command: Option<&CommandId>,
) -> bool {
    let next = host.models_mut().get_cloned(draft).unwrap_or_default();
    {
        let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
        clear_buffered_text_field_pending_blur(&mut state);
        let _ = state.session.commit();
    }

    {
        let next_for_update = next.clone();
        let _ = host.models_mut().update(model, |text| {
            if text.as_str() != next_for_update.as_str() {
                *text = next_for_update.clone();
            }
        });
    }
    if let Some(command) = submit_command {
        host.dispatch_command(Some(action_cx.window), command.clone());
    }
    host.request_redraw(action_cx.window);
    true
}

pub(in crate::controls::text_field) fn cancel_buffered_text_field(
    host: &mut dyn UiFocusActionHost,
    action_cx: ActionCx,
    model: &Model<String>,
    draft: &Model<String>,
    buffered_state: &Arc<Mutex<BufferedTextFieldState>>,
    on_outcome: Option<&OnTextFieldOutcome>,
) -> bool {
    let current_draft = host.models_mut().get_cloned(draft).unwrap_or_default();
    let current_model = host.models_mut().get_cloned(model).unwrap_or_default();
    let (revert, should_emit_outcome) = {
        let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
        let changed = state.session.changed_from(&current_draft);
        clear_buffered_text_field_pending_blur(&mut state);
        let revert = state
            .session
            .cancel()
            .unwrap_or_else(|| current_model.clone());
        (revert, changed)
    };

    {
        let revert_for_draft = revert.clone();
        let _ = host.models_mut().update(draft, |text| {
            if text.as_str() != revert_for_draft.as_str() {
                *text = revert_for_draft.clone();
            }
        });
    }
    let _ = host.models_mut().update(model, |text| {
        if text.as_str() != revert.as_str() {
            *text = revert.clone();
        }
    });
    if should_emit_outcome && let Some(cb) = on_outcome {
        cb(host, action_cx, TextFieldOutcome::Canceled);
    }
    host.request_redraw(action_cx.window);
    true
}

pub(in crate::controls::text_field) fn cancel_buffered_text_field_from_controller(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    model: &Model<String>,
    draft: &Model<String>,
    buffered_state: &Arc<Mutex<BufferedTextFieldState>>,
) -> bool {
    let current_model = host.models_mut().get_cloned(model).unwrap_or_default();
    let revert = {
        let mut state = buffered_state.lock().unwrap_or_else(|e| e.into_inner());
        clear_buffered_text_field_pending_blur(&mut state);
        state
            .session
            .cancel()
            .unwrap_or_else(|| current_model.clone())
    };

    {
        let revert_for_draft = revert.clone();
        let _ = host.models_mut().update(draft, |text| {
            if text.as_str() != revert_for_draft.as_str() {
                *text = revert_for_draft.clone();
            }
        });
    }
    let _ = host.models_mut().update(model, |text| {
        if text.as_str() != revert.as_str() {
            *text = revert.clone();
        }
    });
    host.request_redraw(action_cx.window);
    true
}
