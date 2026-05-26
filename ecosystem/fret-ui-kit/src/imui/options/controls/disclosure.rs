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

#[derive(Debug, Clone)]
pub struct TreeNodeOptions {
    pub enabled: bool,
    pub open: Option<fret_runtime::Model<bool>>,
    pub default_open: bool,
    pub selected: bool,
    pub leaf: bool,
    /// Optional hierarchy level for accessibility semantics (1-based).
    ///
    /// This also drives the default visual indentation for the first-cut immediate tree helper.
    pub level: u32,
    pub pos_in_set: Option<u32>,
    pub set_size: Option<u32>,
    pub test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
    /// Exact key chord that activates the disclosure trigger while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for TreeNodeOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            open: None,
            default_open: false,
            selected: false,
            leaf: false,
            level: 1,
            pos_in_set: None,
            set_size: None,
            test_id: None,
            content_test_id: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}
