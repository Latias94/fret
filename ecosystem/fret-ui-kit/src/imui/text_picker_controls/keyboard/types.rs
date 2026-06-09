use std::sync::Arc;

#[derive(Debug, Clone)]
pub(in crate::imui::text_picker_controls) struct InputTextPickerKeyboardPick {
    pub(in crate::imui::text_picker_controls) source_index: usize,
    pub(in crate::imui::text_picker_controls) value: Arc<str>,
}

#[derive(Debug, Clone, Default)]
pub(in crate::imui::text_picker_controls) struct InputTextPickerKeyboardState {
    pub(in crate::imui::text_picker_controls) active_source_index: Option<usize>,
    pub(in crate::imui::text_picker_controls) active_element: Option<fret_ui::GlobalElementId>,
    pub(in crate::imui::text_picker_controls) picked: Option<InputTextPickerKeyboardPick>,
}

#[derive(Debug, Clone)]
pub(in crate::imui::text_picker_controls) struct InputTextPickerKeyboardSnapshot {
    pub(in crate::imui::text_picker_controls) active_source_index: Option<usize>,
    pub(in crate::imui::text_picker_controls) pending_pick: Option<InputTextPickerKeyboardPick>,
    pub(in crate::imui::text_picker_controls) active_element: Option<fret_ui::GlobalElementId>,
}
