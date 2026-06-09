use fret_runtime::KeyChord;

pub(in crate::imui::boolean_controls::switch) struct SwitchBehaviorOptions {
    pub(in crate::imui::boolean_controls::switch) enabled: bool,
    pub(in crate::imui::boolean_controls::switch) focusable: bool,
    pub(in crate::imui::boolean_controls::switch) activate_shortcut: Option<KeyChord>,
    pub(in crate::imui::boolean_controls::switch) shortcut_repeat: bool,
}
