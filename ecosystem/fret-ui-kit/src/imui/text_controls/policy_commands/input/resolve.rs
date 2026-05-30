use fret_core::KeyCode;
use fret_runtime::CommandId;
use fret_ui::action::KeyDownCx;

use crate::imui::InputTextOptions;

pub(in crate::imui::text_controls) struct InputTextPolicyCommands {
    completion: Option<CommandId>,
    history_previous: Option<CommandId>,
    history_next: Option<CommandId>,
    undo: Option<CommandId>,
    redo: Option<CommandId>,
    completion_repeat: bool,
    history_repeat: bool,
    undo_redo_repeat: bool,
}

impl InputTextPolicyCommands {
    pub(in crate::imui::text_controls) fn from_options(options: &InputTextOptions) -> Self {
        Self {
            completion: options.completion_command.clone(),
            history_previous: options.history_previous_command.clone(),
            history_next: options.history_next_command.clone(),
            undo: options.undo_command.clone(),
            redo: options.redo_command.clone(),
            completion_repeat: options.completion_command_repeat,
            history_repeat: options.history_command_repeat,
            undo_redo_repeat: options.undo_redo_command_repeat,
        }
    }

    pub(in crate::imui::text_controls) fn is_empty(&self) -> bool {
        self.completion.is_none()
            && self.history_previous.is_none()
            && self.history_next.is_none()
            && self.undo.is_none()
            && self.redo.is_none()
    }
}

pub(in crate::imui::text_controls) fn resolve_input_text_policy_command(
    commands: &InputTextPolicyCommands,
    down: KeyDownCx,
) -> Option<CommandId> {
    if down.ime_composing || down.modifiers.alt || down.modifiers.meta {
        return None;
    }

    if down.modifiers.ctrl {
        match down.key {
            KeyCode::KeyZ
                if !down.modifiers.shift && (!down.repeat || commands.undo_redo_repeat) =>
            {
                commands.undo.clone()
            }
            KeyCode::KeyY
                if !down.modifiers.shift && (!down.repeat || commands.undo_redo_repeat) =>
            {
                commands.redo.clone()
            }
            KeyCode::KeyZ
                if down.modifiers.shift && (!down.repeat || commands.undo_redo_repeat) =>
            {
                commands.redo.clone()
            }
            _ => None,
        }
    } else if !down.modifiers.shift {
        match down.key {
            KeyCode::Tab if !down.repeat || commands.completion_repeat => {
                commands.completion.clone()
            }
            KeyCode::ArrowUp if !down.repeat || commands.history_repeat => {
                commands.history_previous.clone()
            }
            KeyCode::ArrowDown if !down.repeat || commands.history_repeat => {
                commands.history_next.clone()
            }
            _ => None,
        }
    } else {
        None
    }
}
