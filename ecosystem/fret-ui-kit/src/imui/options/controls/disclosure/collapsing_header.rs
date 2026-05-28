use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CollapsingHeaderOptions {
    pub enabled: bool,
    pub open: Option<fret_runtime::Model<bool>>,
    pub default_open: bool,
    pub test_id: Option<Arc<str>>,
    pub header_test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
    /// Exact key chord that activates the disclosure trigger while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for CollapsingHeaderOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            open: None,
            default_open: false,
            test_id: None,
            header_test_id: None,
            content_test_id: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}
