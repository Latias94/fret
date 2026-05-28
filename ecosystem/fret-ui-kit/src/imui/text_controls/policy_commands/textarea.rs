use std::sync::Arc;

use fret_core::{KeyCode, Modifiers};
use fret_ui::UiHost;

use crate::imui::{TextAreaOptions, TextAreaSubmitKey};

pub(in crate::imui::text_controls) fn install_textarea_policy_commands<H: UiHost>(
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
