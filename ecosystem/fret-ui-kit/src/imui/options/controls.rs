use std::sync::Arc;

use fret_core::{Px, SemanticsRole, Size};

use super::menus::PopupMenuOptions;

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

#[derive(Debug, Clone)]
pub struct SelectableOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub selected: bool,
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
            close_popup: None,
            a11y_label: None,
            a11y_role: Some(SemanticsRole::ListBoxOption),
            test_id: None,
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}

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
pub struct ComboOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub popup: PopupMenuOptions,
    /// Exact key chord that activates the combo trigger while it is focused.
    ///
    /// This is an item-local shortcut seam. It does not participate in global shortcut ownership
    /// arbitration.
    pub activate_shortcut: Option<fret_runtime::KeyChord>,
    /// Whether `activate_shortcut` should fire on repeated keydown events.
    pub shortcut_repeat: bool,
}

impl Default for ComboOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            popup: PopupMenuOptions::default(),
            activate_shortcut: None,
            shortcut_repeat: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonArrowDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonVariant {
    Default,
    Small,
    Arrow(ButtonArrowDirection),
    Invisible { size: Size },
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Default
    }
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

#[derive(Debug, Clone)]
pub struct InputTextOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub a11y_role: Option<SemanticsRole>,
    pub placeholder: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub submit_command: Option<fret_runtime::CommandId>,
    pub cancel_command: Option<fret_runtime::CommandId>,
}

impl Default for InputTextOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            a11y_role: Some(SemanticsRole::TextField),
            placeholder: None,
            test_id: None,
            submit_command: None,
            cancel_command: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextAreaOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub min_height: Px,
    /// If true, opt into a stable multiline line-box policy suitable for UI/form text areas.
    ///
    /// This is expected to reduce baseline jitter across mixed-script / emoji lines, at the cost
    /// of potentially clipping glyph ink that exceeds the chosen line box.
    pub stable_line_boxes: bool,
}

impl Default for TextAreaOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            min_height: Px(80.0),
            stable_line_boxes: false,
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

#[derive(Debug, Clone)]
pub struct SliderOptions {
    pub enabled: bool,
    pub focusable: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            focusable: true,
            a11y_label: None,
            test_id: None,
            min: 0.0,
            max: 100.0,
            step: 1.0,
        }
    }
}

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
