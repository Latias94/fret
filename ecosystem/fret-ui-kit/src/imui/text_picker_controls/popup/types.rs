use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::GlobalElementId;

use super::super::super::PopupMenuOptions;
use super::super::keyboard::{InputTextPickerKeyboardPick, InputTextPickerKeyboardState};

pub(in crate::imui::text_picker_controls) struct InputTextPickerPopupInput<'a> {
    pub(in crate::imui::text_picker_controls) id: &'a str,
    pub(in crate::imui::text_picker_controls) trigger: Option<GlobalElementId>,
    pub(in crate::imui::text_picker_controls) popup: PopupMenuOptions,
    pub(in crate::imui::text_picker_controls) model: Model<String>,
    pub(in crate::imui::text_picker_controls) popup_open: Model<bool>,
    pub(in crate::imui::text_picker_controls) keyboard_state:
        Option<Model<InputTextPickerKeyboardState>>,
    pub(in crate::imui::text_picker_controls) visible_candidates: &'a [(usize, Arc<str>)],
    pub(in crate::imui::text_picker_controls) selected_value: String,
    pub(in crate::imui::text_picker_controls) active_source_index: Option<usize>,
    pub(in crate::imui::text_picker_controls) pending_keyboard_pick:
        Option<InputTextPickerKeyboardPick>,
    pub(in crate::imui::text_picker_controls) item_test_id_base: Option<Arc<str>>,
    pub(in crate::imui::text_picker_controls) install_keyboard_handler: bool,
    pub(in crate::imui::text_picker_controls) keyboard_repeat: bool,
}

pub(in crate::imui::text_picker_controls) struct InputTextPickerPopupResult {
    pub(in crate::imui::text_picker_controls) opened: bool,
    pub(in crate::imui::text_picker_controls) picked_index: Option<usize>,
    pub(in crate::imui::text_picker_controls) picked: Option<Arc<str>>,
}
