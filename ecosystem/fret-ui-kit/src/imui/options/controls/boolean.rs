use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CheckboxOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    /// Exact key chord that activates the checkbox while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for CheckboxOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RadioOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    /// Exact key chord that activates the radio button while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for RadioOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwitchOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    /// Exact key chord that activates the switch while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for SwitchOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}
