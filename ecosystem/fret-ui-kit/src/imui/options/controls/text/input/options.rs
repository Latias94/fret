use std::sync::Arc;

use fret_core::SemanticsRole;

use super::super::filters::{InputTextCustomFilter, InputTextFilters};
use super::mode::InputTextMode;

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
