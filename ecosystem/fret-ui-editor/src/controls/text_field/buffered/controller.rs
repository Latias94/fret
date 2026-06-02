use std::sync::{Arc, Mutex};

use fret_runtime::{CommandId, Model};
use fret_ui::action::{ActionCx, UiActionHost};

use super::{
    BufferedTextFieldState, cancel_buffered_text_field_from_controller,
    commit_buffered_text_field_from_controller,
};

#[derive(Clone, Default)]
pub struct TextFieldDraftController {
    binding: Arc<Mutex<Option<BufferedTextFieldDraftBinding>>>,
}

#[derive(Clone)]
struct BufferedTextFieldDraftBinding {
    model: Model<String>,
    draft: Model<String>,
    buffered_state: Arc<Mutex<BufferedTextFieldState>>,
    submit_command: Option<CommandId>,
}

impl TextFieldDraftController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_bound(&self) -> bool {
        self.binding
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some()
    }

    pub fn commit(&self, host: &mut dyn UiActionHost, action_cx: ActionCx) -> bool {
        let binding = self
            .binding
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let Some(binding) = binding else {
            return false;
        };

        commit_buffered_text_field_from_controller(
            host,
            action_cx,
            &binding.model,
            &binding.draft,
            &binding.buffered_state,
            binding.submit_command.as_ref(),
        )
    }

    pub fn discard(&self, host: &mut dyn UiActionHost, action_cx: ActionCx) -> bool {
        let binding = self
            .binding
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let Some(binding) = binding else {
            return false;
        };

        cancel_buffered_text_field_from_controller(
            host,
            action_cx,
            &binding.model,
            &binding.draft,
            &binding.buffered_state,
        )
    }

    pub(in crate::controls::text_field) fn bind(
        &self,
        model: Model<String>,
        draft: Model<String>,
        buffered_state: Arc<Mutex<BufferedTextFieldState>>,
        submit_command: Option<CommandId>,
    ) {
        *self.binding.lock().unwrap_or_else(|e| e.into_inner()) =
            Some(BufferedTextFieldDraftBinding {
                model,
                draft,
                buffered_state,
                submit_command,
            });
    }

    pub(in crate::controls::text_field) fn unbind(&self) {
        *self.binding.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

impl std::fmt::Debug for TextFieldDraftController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextFieldDraftController")
            .field("is_bound", &self.is_bound())
            .finish()
    }
}
