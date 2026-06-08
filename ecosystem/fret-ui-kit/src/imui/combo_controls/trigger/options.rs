use std::sync::Arc;

use fret_runtime::KeyChord;

pub(in crate::imui::combo_controls) struct ComboTriggerOptions {
    pub(in crate::imui::combo_controls) enabled: bool,
    pub(in crate::imui::combo_controls) focusable: bool,
    pub(in crate::imui::combo_controls) a11y_label: Option<Arc<str>>,
    pub(in crate::imui::combo_controls) test_id: Option<Arc<str>>,
    pub(in crate::imui::combo_controls) activate_shortcut: Option<KeyChord>,
    pub(in crate::imui::combo_controls) shortcut_repeat: bool,
    pub(in crate::imui::combo_controls) open: bool,
}
