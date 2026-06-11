use fret_core::Px;

use super::options::TextAreaOptions;
use super::submit_key::TextAreaSubmitKey;

impl Default for TextAreaOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            read_only: false,
            select_all_on_focus: false,
            allow_tab_input: false,
            a11y_label: None,
            test_id: None,
            min_height: Px(80.0),
            submit_command: None,
            cancel_command: None,
            submit_key: TextAreaSubmitKey::default(),
            submit_cancel_command_repeat: false,
            stable_line_boxes: false,
        }
    }
}
