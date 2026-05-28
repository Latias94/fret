use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MenuItemOptions {
    pub enabled: bool,
    pub close_popup: Option<fret_runtime::Model<bool>>,
    pub shortcut: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub shortcut_test_id: Option<Arc<str>>,
    pub submenu: bool,
    pub expanded: Option<bool>,
    /// Exact key chord that activates the menu item while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for MenuItemOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            close_popup: None,
            shortcut: None,
            test_id: None,
            shortcut_test_id: None,
            submenu: false,
            expanded: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}
