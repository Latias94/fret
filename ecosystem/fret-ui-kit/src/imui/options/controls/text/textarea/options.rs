use std::sync::Arc;

use fret_core::Px;

use super::submit_key::TextAreaSubmitKey;

#[derive(Debug, Clone)]
pub struct TextAreaOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub read_only: bool,
    pub select_all_on_focus: bool,
    /// Dear ImGui-style multiline Tab policy. When false, focused Tab is left for focus traversal
    /// or higher-level shortcut handling instead of mutating the text model.
    pub allow_tab_input: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub min_height: Px,
    /// Command dispatched from the focused text area when the configured submit key is pressed.
    ///
    /// This is the policy-layer equivalent of Dear ImGui's multiline submit behavior. The
    /// default key is Ctrl+Enter so ordinary Enter keeps inserting newlines.
    pub submit_command: Option<fret_runtime::CommandId>,
    /// Command dispatched from the focused text area on unmodified Escape.
    pub cancel_command: Option<fret_runtime::CommandId>,
    /// Key policy used for `submit_command`.
    pub submit_key: TextAreaSubmitKey,
    /// Whether submit/cancel commands should fire for repeated keydown events.
    pub submit_cancel_command_repeat: bool,
    /// If true, opt into a stable multiline line-box policy suitable for UI/form text areas.
    ///
    /// This is expected to reduce baseline jitter across mixed-script / emoji lines, at the cost
    /// of potentially clipping glyph ink that exceeds the chosen line box.
    pub stable_line_boxes: bool,
}

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
