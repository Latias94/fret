use fret_runtime::CommandId;

use crate::imui::{TextAreaOptions, TextAreaSubmitKey};

pub(in crate::imui::text_controls) struct TextAreaPolicyCommands {
    pub(super) submit: Option<CommandId>,
    pub(super) cancel: Option<CommandId>,
    pub(super) submit_key: TextAreaSubmitKey,
    pub(super) command_repeat: bool,
}

impl TextAreaPolicyCommands {
    pub(in crate::imui::text_controls) fn from_options(options: &TextAreaOptions) -> Self {
        Self {
            submit: options.submit_command.clone(),
            cancel: options.cancel_command.clone(),
            submit_key: options.submit_key,
            command_repeat: options.submit_cancel_command_repeat,
        }
    }

    pub(in crate::imui::text_controls) fn is_empty(&self) -> bool {
        self.submit.is_none() && self.cancel.is_none()
    }
}
