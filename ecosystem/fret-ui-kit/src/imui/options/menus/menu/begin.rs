use std::sync::Arc;

use fret_core::{Px, Size};

use super::super::PopupMenuOptions;
use crate::primitives::popper;

#[derive(Debug, Clone)]
pub struct BeginMenuOptions {
    pub enabled: bool,
    pub test_id: Option<Arc<str>>,
    pub popup: PopupMenuOptions,
    /// Exact key chord that activates the menu trigger while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for BeginMenuOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            test_id: None,
            popup: PopupMenuOptions::default(),
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BeginSubmenuOptions {
    pub enabled: bool,
    pub test_id: Option<Arc<str>>,
    pub popup: PopupMenuOptions,
    /// Exact key chord that activates the submenu trigger while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for BeginSubmenuOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            test_id: None,
            popup: PopupMenuOptions {
                placement: popper::PopperContentPlacement::new(
                    popper::LayoutDirection::Ltr,
                    popper::Side::Right,
                    popper::Align::Start,
                    Px(4.0),
                ),
                estimated_size: Size::new(Px(160.0), Px(120.0)),
                modal: false,
                auto_focus: false,
            },
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}
