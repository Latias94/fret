use std::sync::{Arc, Mutex};

use fret_runtime::{CommandId, Model};
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::{ElementContextAccess, Invalidation, UiHost};

use super::{
    BufferedTextFieldState, cancel_buffered_text_field_from_controller,
    cancel_buffered_text_field_from_controller_if_dirty,
    commit_buffered_text_field_from_controller,
    commit_buffered_text_field_from_controller_if_dirty,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextFieldDraftSnapshot {
    Unbound,
    Clean { committed: String },
    Dirty { committed: String, draft: String },
}

impl TextFieldDraftSnapshot {
    pub fn is_dirty(&self) -> bool {
        matches!(self, Self::Dirty { .. })
    }

    pub fn committed(&self) -> Option<&str> {
        match self {
            Self::Unbound => None,
            Self::Clean { committed } | Self::Dirty { committed, .. } => Some(committed),
        }
    }
}

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

    /// Read the current draft outside declarative rendering.
    ///
    /// Render code should use [`Self::snapshot_in`] so model changes invalidate the owning view.
    pub fn snapshot(&self, host: &impl fret_runtime::ModelHost) -> TextFieldDraftSnapshot {
        let binding = self.bound_binding();
        let Some(binding) = binding else {
            return TextFieldDraftSnapshot::Unbound;
        };

        let committed = host.models().get_cloned(&binding.model).unwrap_or_default();
        let draft = host
            .models()
            .get_cloned(&binding.draft)
            .unwrap_or_else(|| committed.clone());
        binding.snapshot(committed, draft)
    }

    /// Read the current draft through the declarative context so view-cache invalidation tracks
    /// both the committed value and the private draft model.
    pub fn snapshot_in<'a, H, Cx>(
        &self,
        cx: &mut Cx,
        invalidation: Invalidation,
    ) -> TextFieldDraftSnapshot
    where
        H: UiHost + 'a,
        Cx: ElementContextAccess<'a, H>,
    {
        let Some(binding) = self.bound_binding() else {
            return TextFieldDraftSnapshot::Unbound;
        };

        let committed = cx
            .elements()
            .get_model_cloned(&binding.model, invalidation)
            .unwrap_or_default();
        let draft = cx
            .elements()
            .get_model_cloned(&binding.draft, invalidation)
            .unwrap_or_else(|| committed.clone());
        binding.snapshot(committed, draft)
    }

    pub fn commit(&self, host: &mut dyn UiActionHost, action_cx: ActionCx) -> bool {
        let Some(binding) = self.bound_binding() else {
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

    pub fn commit_if_dirty(&self, host: &mut dyn UiActionHost, action_cx: ActionCx) -> bool {
        let Some(binding) = self.bound_binding() else {
            return false;
        };

        commit_buffered_text_field_from_controller_if_dirty(
            host,
            action_cx,
            &binding.model,
            &binding.draft,
            &binding.buffered_state,
            binding.submit_command.as_ref(),
        )
    }

    pub fn discard(&self, host: &mut dyn UiActionHost, action_cx: ActionCx) -> bool {
        let Some(binding) = self.bound_binding() else {
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

    pub fn discard_if_dirty(&self, host: &mut dyn UiActionHost, action_cx: ActionCx) -> bool {
        let Some(binding) = self.bound_binding() else {
            return false;
        };

        cancel_buffered_text_field_from_controller_if_dirty(
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

    fn bound_binding(&self) -> Option<BufferedTextFieldDraftBinding> {
        self.binding
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

impl BufferedTextFieldDraftBinding {
    fn snapshot(&self, committed: String, draft: String) -> TextFieldDraftSnapshot {
        let session_active = self
            .buffered_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .session
            .is_active();

        if session_active && draft != committed {
            TextFieldDraftSnapshot::Dirty { committed, draft }
        } else {
            TextFieldDraftSnapshot::Clean { committed }
        }
    }
}

impl std::fmt::Debug for TextFieldDraftController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextFieldDraftController")
            .field("is_bound", &self.is_bound())
            .finish()
    }
}
