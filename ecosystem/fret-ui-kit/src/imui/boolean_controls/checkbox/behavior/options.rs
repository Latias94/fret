use fret_runtime::KeyChord;

pub(in crate::imui::boolean_controls::checkbox) struct CheckboxBehaviorOptions {
    pub(in crate::imui::boolean_controls::checkbox) enabled: bool,
    pub(in crate::imui::boolean_controls::checkbox) activate_shortcut: Option<KeyChord>,
    pub(in crate::imui::boolean_controls::checkbox) shortcut_repeat: bool,
}
