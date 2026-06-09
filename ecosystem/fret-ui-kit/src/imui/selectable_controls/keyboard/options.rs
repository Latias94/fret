use fret_runtime::{KeyChord, Model};

pub(in crate::imui::selectable_controls) struct SelectableKeyboardOptions {
    pub(in crate::imui::selectable_controls) close_popup: Option<Model<bool>>,
    pub(in crate::imui::selectable_controls) activate_shortcut: Option<KeyChord>,
    pub(in crate::imui::selectable_controls) shortcut_repeat: bool,
}
