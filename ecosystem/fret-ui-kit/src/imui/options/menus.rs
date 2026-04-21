use std::sync::Arc;

use fret_core::{Px, Size};

use crate::primitives::popper;

#[derive(Debug, Clone, Copy)]
pub struct PopupMenuOptions {
    pub placement: popper::PopperContentPlacement,
    pub estimated_size: Size,
    pub modal: bool,
    pub auto_focus: bool,
}

impl Default for PopupMenuOptions {
    fn default() -> Self {
        Self {
            placement: popper::PopperContentPlacement::new(
                popper::LayoutDirection::Ltr,
                popper::Side::Bottom,
                popper::Align::Start,
                Px(4.0),
            ),
            estimated_size: Size::new(Px(160.0), Px(120.0)),
            modal: true,
            auto_focus: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MenuBarOptions {
    pub gap: crate::MetricRef,
    pub test_id: Option<Arc<str>>,
}

impl Default for MenuBarOptions {
    fn default() -> Self {
        Self {
            gap: crate::MetricRef::space(crate::Space::N1),
            test_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TabBarOptions {
    pub selected: Option<fret_runtime::Model<Option<Arc<str>>>>,
    pub gap: crate::MetricRef,
    pub test_id: Option<Arc<str>>,
}

impl Default for TabBarOptions {
    fn default() -> Self {
        Self {
            selected: None,
            gap: crate::MetricRef::space(crate::Space::N1),
            test_id: None,
        }
    }
}

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

#[derive(Debug, Clone, Copy)]
pub struct PopupModalOptions {
    pub size: Size,
    pub close_on_outside_press: bool,
}

impl Default for PopupModalOptions {
    fn default() -> Self {
        Self {
            size: Size::new(Px(320.0), Px(200.0)),
            close_on_outside_press: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TooltipOptions {
    pub placement: popper::PopperContentPlacement,
    pub estimated_size: Size,
    pub window_margin: Px,
    pub open_delay_frames_override: Option<u32>,
    pub close_delay_frames_override: Option<u32>,
    pub disable_hoverable_content: Option<bool>,
    pub test_id: Option<Arc<str>>,
}

impl Default for TooltipOptions {
    fn default() -> Self {
        Self {
            placement: popper::PopperContentPlacement::new(
                popper::LayoutDirection::Ltr,
                popper::Side::Top,
                popper::Align::Center,
                Px(6.0),
            )
            .with_shift_cross_axis(true),
            estimated_size: Size::new(Px(180.0), Px(32.0)),
            window_margin: Px(8.0),
            open_delay_frames_override: None,
            close_delay_frames_override: None,
            disable_hoverable_content: None,
            test_id: None,
        }
    }
}

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
