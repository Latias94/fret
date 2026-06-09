use fret_runtime::KeyChord;

pub(in crate::imui::boolean_controls::radio) struct RadioBehaviorOptions {
    pub(in crate::imui::boolean_controls::radio) enabled: bool,
    pub(in crate::imui::boolean_controls::radio) activate_shortcut: Option<KeyChord>,
    pub(in crate::imui::boolean_controls::radio) shortcut_repeat: bool,
}
