use fret_runtime::CommandId;

use crate::imui::InputTextOptions;

pub(in crate::imui::text_controls) struct InputTextPolicyCommands {
    pub(super) completion: Option<CommandId>,
    pub(super) history_previous: Option<CommandId>,
    pub(super) history_next: Option<CommandId>,
    pub(super) undo: Option<CommandId>,
    pub(super) redo: Option<CommandId>,
    pub(super) completion_repeat: bool,
    pub(super) history_repeat: bool,
    pub(super) undo_redo_repeat: bool,
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
