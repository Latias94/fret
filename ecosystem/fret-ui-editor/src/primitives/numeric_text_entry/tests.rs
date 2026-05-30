use fret_core::{KeyCode, Modifiers};
use fret_ui::action::KeyDownCx;

use super::{NumericReplacementPlan, replacement_plan};

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
