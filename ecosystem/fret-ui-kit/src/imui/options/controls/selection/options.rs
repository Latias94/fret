use std::sync::Arc;

use fret_core::SemanticsRole;

#[derive(Debug, Clone)]
pub struct SelectableOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub selected: bool,
    /// Render the row with hovered-style emphasis without changing selected semantics.
    ///
    /// This is the Fret policy-layer equivalent of Dear ImGui's `SelectableFlags_Highlight`.
    /// It is useful for keyboard-active candidates in popup/list recipes where selection remains
    /// app-owned and should not be conflated with navigation highlight.
    pub highlighted: bool,
    pub close_popup: Option<fret_runtime::Model<bool>>,
    pub a11y_label: Option<Arc<str>>,
    pub a11y_role: Option<SemanticsRole>,
    pub test_id: Option<Arc<str>>,
    /// Exact key chord that activates the selectable while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for SelectableOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            selected: false,
            highlighted: false,
            close_popup: None,
            a11y_label: None,
            a11y_role: Some(SemanticsRole::ListBoxOption),
            test_id: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}
