//! Shared text-entry policy helpers for editor-owned numeric controls.
//!
//! This keeps the editor baseline in one place:
//! - typed entry arms a "replace current value" mode on initial focus by default,
//! - Escape/Enter handling can stay control-local,
//! - and wrappers such as `DragValue` / `Slider` / `AxisDragValue` do not need to
//!   re-derive the same focus-entry rules independently.

use std::sync::{Arc, Mutex};

use fret_core::{KeyCode, keycode_to_ascii_lowercase};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, KeyDownCx, UiFocusActionHost};
use fret_ui::{ElementContext, Invalidation, UiHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumericInputSelectionBehavior {
    PreserveDraft,
    #[default]
    ReplaceAllOnFocus,
}

#[derive(Debug, Default)]
pub(crate) struct NumericTextEntryFocusState {
    was_focused: bool,
    replace_on_next_edit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericReplacementPlan {
    Ignore,
    Disarm,
    ClearAndContinue,
    ClearAndConsume,
}

pub(crate) fn numeric_text_entry_focus_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Arc<Mutex<NumericTextEntryFocusState>> {
    cx.with_state(
        || Arc::new(Mutex::new(NumericTextEntryFocusState::default())),
        |state| state.clone(),
    )
}

pub(crate) fn sync_numeric_text_entry_focus<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    focus_state: &Arc<Mutex<NumericTextEntryFocusState>>,
    is_focused: bool,
    current_text: &Arc<str>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    selection_behavior: NumericInputSelectionBehavior,
) {
    let mut state = focus_state.lock().unwrap_or_else(|e| e.into_inner());

    if is_focused && !state.was_focused {
        state.replace_on_next_edit = matches!(
            selection_behavior,
            NumericInputSelectionBehavior::ReplaceAllOnFocus
        ) && !current_text.is_empty();
    } else if !is_focused {
        let _ = cx
            .app
            .models_mut()
            .update(draft, |text| *text = current_text.as_ref().to_string());
        let _ = cx.app.models_mut().update(error, |value| *value = None);
        state.replace_on_next_edit = false;
    }

    state.was_focused = is_focused;
}

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

pub(crate) fn clear_numeric_error_when_draft_changes<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    is_focused: bool,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    last_draft_text: &Arc<Mutex<String>>,
) {
    if !is_focused {
        return;
    }

    let draft_text = cx
        .get_model_cloned(draft, Invalidation::Paint)
        .unwrap_or_default();
    let changed = {
        let mut last = last_draft_text.lock().unwrap_or_else(|e| e.into_inner());
        if *last == draft_text {
            false
        } else {
            *last = draft_text;
            true
        }
    };

    if changed {
        let _ = cx.app.models_mut().update(error, |value| *value = None);
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

fn replacement_plan(down: KeyDownCx) -> NumericReplacementPlan {
    if down.ime_composing {
        return NumericReplacementPlan::Ignore;
    }

    if down.repeat {
        return NumericReplacementPlan::Disarm;
    }

    if down.modifiers.alt {
        return NumericReplacementPlan::Disarm;
    }

    if down.modifiers.ctrl || down.modifiers.meta {
        return match down.key {
            KeyCode::KeyV => NumericReplacementPlan::ClearAndContinue,
            _ => NumericReplacementPlan::Disarm,
        };
    }

    match down.key {
        KeyCode::Backspace | KeyCode::Delete => NumericReplacementPlan::ClearAndConsume,
        KeyCode::Enter
        | KeyCode::NumpadEnter
        | KeyCode::Escape
        | KeyCode::Tab
        | KeyCode::ArrowUp
        | KeyCode::ArrowDown
        | KeyCode::ArrowLeft
        | KeyCode::ArrowRight
        | KeyCode::Home
        | KeyCode::End
        | KeyCode::PageUp
        | KeyCode::PageDown => NumericReplacementPlan::Disarm,
        _ if is_text_insertion_key(down.key) => NumericReplacementPlan::ClearAndContinue,
        _ => NumericReplacementPlan::Disarm,
    }
}

fn is_text_insertion_key(key: KeyCode) -> bool {
    keycode_to_ascii_lowercase(key).is_some()
        || matches!(
            key,
            KeyCode::Space
                | KeyCode::Minus
                | KeyCode::Equal
                | KeyCode::BracketLeft
                | KeyCode::BracketRight
                | KeyCode::Backslash
                | KeyCode::Semicolon
                | KeyCode::Quote
                | KeyCode::Backquote
                | KeyCode::Comma
                | KeyCode::Period
                | KeyCode::Slash
                | KeyCode::Numpad0
                | KeyCode::Numpad1
                | KeyCode::Numpad2
                | KeyCode::Numpad3
                | KeyCode::Numpad4
                | KeyCode::Numpad5
                | KeyCode::Numpad6
                | KeyCode::Numpad7
                | KeyCode::Numpad8
                | KeyCode::Numpad9
                | KeyCode::NumpadAdd
                | KeyCode::NumpadSubtract
                | KeyCode::NumpadMultiply
                | KeyCode::NumpadDivide
                | KeyCode::NumpadDecimal
        )
}

#[cfg(test)]
mod tests {
    use super::{NumericReplacementPlan, replacement_plan};
    use fret_core::{KeyCode, Modifiers};
    use fret_ui::action::KeyDownCx;

    fn key(key: KeyCode) -> KeyDownCx {
        KeyDownCx {
            key,
            modifiers: Modifiers::default(),
            repeat: false,
            ime_composing: false,
        }
    }

    #[test]
    fn replacement_plan_clears_on_plain_character_keys() {
        assert_eq!(
            replacement_plan(key(KeyCode::Digit2)),
            NumericReplacementPlan::ClearAndContinue
        );
        assert_eq!(
            replacement_plan(key(KeyCode::Period)),
            NumericReplacementPlan::ClearAndContinue
        );
        assert_eq!(
            replacement_plan(key(KeyCode::KeyE)),
            NumericReplacementPlan::ClearAndContinue
        );
    }

    #[test]
    fn replacement_plan_consumes_delete_keys() {
        assert_eq!(
            replacement_plan(key(KeyCode::Backspace)),
            NumericReplacementPlan::ClearAndConsume
        );
        assert_eq!(
            replacement_plan(key(KeyCode::Delete)),
            NumericReplacementPlan::ClearAndConsume
        );
    }

    #[test]
    fn replacement_plan_disarms_on_navigation_keys() {
        assert_eq!(
            replacement_plan(key(KeyCode::ArrowLeft)),
            NumericReplacementPlan::Disarm
        );
        assert_eq!(
            replacement_plan(key(KeyCode::Enter)),
            NumericReplacementPlan::Disarm
        );
    }

    #[test]
    fn replacement_plan_clears_on_platform_paste_shortcut() {
        assert_eq!(
            replacement_plan(KeyDownCx {
                key: KeyCode::KeyV,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                repeat: false,
                ime_composing: false,
            }),
            NumericReplacementPlan::ClearAndContinue
        );
    }
}
