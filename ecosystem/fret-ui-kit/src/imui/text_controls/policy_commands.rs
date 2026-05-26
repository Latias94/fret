use std::sync::Arc;

use fret_core::{KeyCode, Modifiers};
use fret_ui::UiHost;

use super::super::{InputTextOptions, TextAreaOptions, TextAreaSubmitKey};

pub(super) fn install_input_text_policy_commands<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    options: &InputTextOptions,
) {
    let completion_command = options.completion_command.clone();
    let history_previous_command = options.history_previous_command.clone();
    let history_next_command = options.history_next_command.clone();
    let undo_command = options.undo_command.clone();
    let redo_command = options.redo_command.clone();
    let completion_command_repeat = options.completion_command_repeat;
    let history_command_repeat = options.history_command_repeat;
    let undo_redo_command_repeat = options.undo_redo_command_repeat;

    if completion_command.is_none()
        && history_previous_command.is_none()
        && history_next_command.is_none()
        && undo_command.is_none()
        && redo_command.is_none()
    {
        return;
    }

    cx.key_add_on_key_down_for(
        id,
        Arc::new(move |host, action_cx, down| {
            if down.ime_composing || down.modifiers.alt || down.modifiers.meta {
                return false;
            }

            let command = if down.modifiers.ctrl {
                match down.key {
                    KeyCode::KeyZ
                        if !down.modifiers.shift && (!down.repeat || undo_redo_command_repeat) =>
                    {
                        undo_command.clone()
                    }
                    KeyCode::KeyY
                        if !down.modifiers.shift && (!down.repeat || undo_redo_command_repeat) =>
                    {
                        redo_command.clone()
                    }
                    KeyCode::KeyZ
                        if down.modifiers.shift && (!down.repeat || undo_redo_command_repeat) =>
                    {
                        redo_command.clone()
                    }
                    _ => None,
                }
            } else if !down.modifiers.shift {
                match down.key {
                    KeyCode::Tab if !down.repeat || completion_command_repeat => {
                        completion_command.clone()
                    }
                    KeyCode::ArrowUp if !down.repeat || history_command_repeat => {
                        history_previous_command.clone()
                    }
                    KeyCode::ArrowDown if !down.repeat || history_command_repeat => {
                        history_next_command.clone()
                    }
                    _ => None,
                }
            } else {
                None
            };

            let Some(command) = command else {
                return false;
            };

            host.dispatch_command(Some(action_cx.window), command);
            true
        }),
    );
}

pub(super) fn install_textarea_policy_commands<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    id: fret_ui::GlobalElementId,
    options: &TextAreaOptions,
) {
    let submit_command = options.submit_command.clone();
    let cancel_command = options.cancel_command.clone();
    let submit_key = options.submit_key;
    let command_repeat = options.submit_cancel_command_repeat;

    if submit_command.is_none() && cancel_command.is_none() {
        return;
    }

    cx.key_add_on_key_down_capture_for(
        id,
        Arc::new(move |host, action_cx, down| {
            if down.ime_composing || down.modifiers.alt || down.modifiers.meta {
                return false;
            }

            let command = match down.key {
                KeyCode::Enter | KeyCode::NumpadEnter => match submit_key {
                    TextAreaSubmitKey::CtrlEnter
                        if down.modifiers
                            == (Modifiers {
                                ctrl: true,
                                ..Default::default()
                            }) =>
                    {
                        submit_command.clone()
                    }
                    TextAreaSubmitKey::Enter if down.modifiers == Modifiers::default() => {
                        submit_command.clone()
                    }
                    _ => None,
                },
                KeyCode::Escape if down.modifiers == Modifiers::default() => cancel_command.clone(),
                _ => None,
            };

            let Some(command) = command else {
                return false;
            };

            if down.repeat && !command_repeat {
                return true;
            }

            host.dispatch_command(Some(action_cx.window), command);
            true
        }),
    );
}
