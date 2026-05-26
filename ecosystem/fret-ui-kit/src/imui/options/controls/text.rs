use std::sync::Arc;

use fret_core::{Px, SemanticsRole, Size};

use super::super::menus::PopupMenuOptions;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputTextMode {
    /// Render the model value directly.
    #[default]
    PlainText,
    /// Obscure the painted text while preserving the underlying model value.
    Password,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InputTextFilters {
    pub decimal: bool,
    pub hexadecimal: bool,
    pub scientific: bool,
    pub uppercase: bool,
    pub no_blank: bool,
}

impl InputTextFilters {
    pub const fn none() -> Self {
        Self {
            decimal: false,
            hexadecimal: false,
            scientific: false,
            uppercase: false,
            no_blank: false,
        }
    }

    pub const fn decimal() -> Self {
        Self {
            decimal: true,
            ..Self::none()
        }
    }

    pub const fn hexadecimal() -> Self {
        Self {
            hexadecimal: true,
            ..Self::none()
        }
    }

    pub const fn scientific() -> Self {
        Self {
            scientific: true,
            ..Self::none()
        }
    }

    pub const fn uppercase() -> Self {
        Self {
            uppercase: true,
            ..Self::none()
        }
    }

    pub const fn no_blank() -> Self {
        Self {
            no_blank: true,
            ..Self::none()
        }
    }

    pub const fn with_decimal(mut self) -> Self {
        self.decimal = true;
        self
    }

    pub const fn with_hexadecimal(mut self) -> Self {
        self.hexadecimal = true;
        self
    }

    pub const fn with_scientific(mut self) -> Self {
        self.scientific = true;
        self
    }

    pub const fn with_uppercase(mut self) -> Self {
        self.uppercase = true;
        self
    }

    pub const fn with_no_blank(mut self) -> Self {
        self.no_blank = true;
        self
    }

    pub const fn is_empty(self) -> bool {
        !self.decimal && !self.hexadecimal && !self.scientific && !self.uppercase && !self.no_blank
    }

    pub fn filter_text(self, text: &str) -> String {
        if self.is_empty() {
            return text.to_string();
        }

        text.chars().filter_map(|c| self.filter_char(c)).collect()
    }

    fn filter_char(self, mut c: char) -> Option<char> {
        if self.decimal && !is_decimal_input_char(c) {
            return None;
        }
        if self.scientific && !is_scientific_input_char(c) {
            return None;
        }
        if self.hexadecimal && !c.is_ascii_hexdigit() {
            return None;
        }
        if self.uppercase && c.is_ascii_lowercase() {
            c = c.to_ascii_uppercase();
        }
        if self.no_blank && matches!(c, ' ' | '\t') {
            return None;
        }
        Some(c)
    }
}

fn is_decimal_input_char(c: char) -> bool {
    c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | '*' | '/')
}

fn is_scientific_input_char(c: char) -> bool {
    is_decimal_input_char(c) || matches!(c, 'e' | 'E')
}

#[derive(Clone)]
pub struct InputTextCustomFilter {
    filter: Arc<dyn Fn(&str) -> String + 'static>,
}

impl InputTextCustomFilter {
    pub fn new(filter: impl Fn(&str) -> String + 'static) -> Self {
        Self {
            filter: Arc::new(filter),
        }
    }

    pub fn filter_text(&self, text: &str) -> String {
        (self.filter)(text)
    }
}

impl std::fmt::Debug for InputTextCustomFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputTextCustomFilter")
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputTextPickerFilter {
    #[default]
    ContainsCaseInsensitive,
    PrefixCaseInsensitive,
    None,
}

impl InputTextPickerFilter {
    pub fn matches(self, query: &str, candidate: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        match self {
            Self::None => true,
            Self::PrefixCaseInsensitive => candidate
                .to_lowercase()
                .starts_with(query.to_lowercase().as_str()),
            Self::ContainsCaseInsensitive => candidate
                .to_lowercase()
                .contains(query.to_lowercase().as_str()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputTextPickerOptions {
    pub input: InputTextOptions,
    pub popup: PopupMenuOptions,
    pub filter: InputTextPickerFilter,
    pub max_items: usize,
    pub open_on_focus: bool,
    pub open_when_empty: bool,
    pub hide_when_exact_match: bool,
    pub keyboard_navigation: bool,
    pub keyboard_repeat: bool,
    pub test_id: Option<Arc<str>>,
}

impl Default for InputTextPickerOptions {
    fn default() -> Self {
        Self {
            input: InputTextOptions::default(),
            popup: PopupMenuOptions {
                modal: false,
                auto_focus: false,
                estimated_size: Size::new(Px(220.0), Px(160.0)),
                ..PopupMenuOptions::default()
            },
            filter: InputTextPickerFilter::ContainsCaseInsensitive,
            max_items: 8,
            open_on_focus: true,
            open_when_empty: false,
            hide_when_exact_match: true,
            keyboard_navigation: true,
            keyboard_repeat: false,
            test_id: None,
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAreaSubmitKey {
    /// Dispatch submit on Ctrl+Enter and leave unmodified Enter for multiline insertion.
    #[default]
    CtrlEnter,
    /// Dispatch submit on unmodified Enter before the textarea inserts a newline.
    Enter,
}

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
