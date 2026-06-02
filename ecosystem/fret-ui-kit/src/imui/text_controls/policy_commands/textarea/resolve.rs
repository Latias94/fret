use fret_core::{KeyCode, Modifiers};
use fret_runtime::CommandId;
use fret_ui::action::KeyDownCx;

use crate::imui::TextAreaSubmitKey;

mod snapshot;

pub(in crate::imui::text_controls) use snapshot::TextAreaPolicyCommands;

pub(in crate::imui::text_controls) enum TextAreaPolicyCommandAction {
    Dispatch(CommandId),
    Consume,
    Ignore,
}

pub(in crate::imui::text_controls) fn resolve_textarea_policy_command(
    commands: &TextAreaPolicyCommands,
    down: KeyDownCx,
) -> TextAreaPolicyCommandAction {
    if down.ime_composing || down.modifiers.alt || down.modifiers.meta {
        return TextAreaPolicyCommandAction::Ignore;
    }

    let command = match down.key {
        KeyCode::Enter | KeyCode::NumpadEnter => match commands.submit_key {
            TextAreaSubmitKey::CtrlEnter
                if down.modifiers
                    == (Modifiers {
                        ctrl: true,
                        ..Default::default()
                    }) =>
            {
                commands.submit.clone()
            }
            TextAreaSubmitKey::Enter if down.modifiers == Modifiers::default() => {
                commands.submit.clone()
            }
            _ => None,
        },
        KeyCode::Escape if down.modifiers == Modifiers::default() => commands.cancel.clone(),
        _ => None,
    };

    let Some(command) = command else {
        return TextAreaPolicyCommandAction::Ignore;
    };

    if down.repeat && !commands.command_repeat {
        return TextAreaPolicyCommandAction::Consume;
    }

    TextAreaPolicyCommandAction::Dispatch(command)
}
