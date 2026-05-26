use std::sync::Arc;

use fret_core::SemanticsRole;

use super::filters::{InputTextCustomFilter, InputTextFilters};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputTextMode {
    /// Render the model value directly.
    #[default]
    PlainText,
    /// Obscure the painted text while preserving the underlying model value.
    Password,
}

#[derive(Debug, Clone)]
pub struct InputTextOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub read_only: bool,
    pub select_all_on_focus: bool,
    pub mode: InputTextMode,
    pub filters: InputTextFilters,
    /// Optional Fret-native equivalent of Dear ImGui's `CallbackCharFilter`.
    ///
    /// Named filters run first; this filter receives the named-filtered insertion text and may
    /// replace or discard it. It intentionally does not expose mutable buffer callbacks.
    pub custom_filter: Option<InputTextCustomFilter>,
    pub a11y_label: Option<Arc<str>>,
    pub a11y_role: Option<SemanticsRole>,
    pub placeholder: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub submit_command: Option<fret_runtime::CommandId>,
    pub cancel_command: Option<fret_runtime::CommandId>,
    /// Command dispatched when an unmodified Tab key is pressed while the field is focused.
    ///
    /// This is the Fret policy-layer equivalent of Dear ImGui's completion callback flag. The
    /// command target owns the completion behavior; the IMUI helper only arbitrates the key.
    pub completion_command: Option<fret_runtime::CommandId>,
    /// Command dispatched when an unmodified Up key is pressed while the field is focused.
    pub history_previous_command: Option<fret_runtime::CommandId>,
    /// Command dispatched when an unmodified Down key is pressed while the field is focused.
    pub history_next_command: Option<fret_runtime::CommandId>,
    /// Command dispatched when Ctrl+Z is pressed while the field is focused.
    ///
    /// Fret text input does not own an internal undo stack. This is the app-owned command policy
    /// equivalent of Dear ImGui's undo/redo shortcuts; leaving it unset is the Fret-native
    /// `NoUndoRedo` behavior.
    pub undo_command: Option<fret_runtime::CommandId>,
    /// Command dispatched when Ctrl+Y or Ctrl+Shift+Z is pressed while the field is focused.
    pub redo_command: Option<fret_runtime::CommandId>,
    /// Whether `completion_command` should fire for repeated Tab keydown events.
    pub completion_command_repeat: bool,
    /// Whether history commands should fire for repeated Up/Down keydown events.
    pub history_command_repeat: bool,
    /// Whether undo/redo commands should fire for repeated keydown events.
    pub undo_redo_command_repeat: bool,
}

impl Default for InputTextOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            read_only: false,
            select_all_on_focus: false,
            mode: InputTextMode::PlainText,
            filters: InputTextFilters::default(),
            custom_filter: None,
            a11y_label: None,
            a11y_role: Some(SemanticsRole::TextField),
            placeholder: None,
            test_id: None,
            submit_command: None,
            cancel_command: None,
            completion_command: None,
            history_previous_command: None,
            history_next_command: None,
            undo_command: None,
            redo_command: None,
            completion_command_repeat: false,
            history_command_repeat: false,
            undo_redo_command_repeat: false,
        }
    }
}
