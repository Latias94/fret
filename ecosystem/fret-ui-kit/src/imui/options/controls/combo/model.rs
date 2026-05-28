use std::sync::Arc;

use super::super::super::menus::PopupMenuOptions;

#[derive(Debug, Clone)]
pub struct ComboModelOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub placeholder: Option<Arc<str>>,
    pub popup: PopupMenuOptions,
    /// Exact key chord that activates the combo trigger while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for ComboModelOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            placeholder: Some(Arc::from("Select...")),
            popup: PopupMenuOptions::default(),
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}
