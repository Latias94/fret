use std::sync::Arc;

use fret_core::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonArrowDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ButtonVariant {
    #[default]
    Default,
    Small,
    Arrow(ButtonArrowDirection),
    Invisible {
        size: Size,
    },
}

#[derive(Debug, Clone)]
pub struct ButtonOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub variant: ButtonVariant,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    /// Exact key chord that activates the button while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for ButtonOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            variant: ButtonVariant::Default,
            a11y_label: None,
            test_id: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}
