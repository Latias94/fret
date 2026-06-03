use fret_core::{KeyCode, keycode_to_ascii_lowercase};
use fret_ui::action::KeyDownCx;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NumericReplacementPlan {
    Ignore,
    Disarm,
    ClearAndContinue,
    ClearAndConsume,
}

pub(super) fn replacement_plan(down: KeyDownCx) -> NumericReplacementPlan {
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
