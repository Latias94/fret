use fret_core::SemanticsRole;

use super::super::filters::InputTextFilters;
use super::mode::InputTextMode;
use super::options::InputTextOptions;

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
