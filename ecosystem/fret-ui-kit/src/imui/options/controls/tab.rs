use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TabItemOptions {
    pub enabled: bool,
    pub default_selected: bool,
    pub test_id: Option<Arc<str>>,
    pub panel_test_id: Option<Arc<str>>,
    /// Exact key chord that activates the tab trigger while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for TabItemOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            default_selected: false,
            test_id: None,
            panel_test_id: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}
