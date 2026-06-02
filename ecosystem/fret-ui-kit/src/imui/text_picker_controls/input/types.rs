use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui::element::AnyElement;

use super::super::super::{InputTextOptions, ResponseExt};
use super::super::keyboard::InputTextPickerKeyboardState;

pub(in crate::imui::text_picker_controls) struct InputTextPickerInputRootRequest<'a> {
    pub(in crate::imui::text_picker_controls) model: Model<String>,
    pub(in crate::imui::text_picker_controls) input_options: InputTextOptions,
    pub(in crate::imui::text_picker_controls) popup_open: Model<bool>,
    pub(in crate::imui::text_picker_controls) keyboard_state:
        Option<Model<InputTextPickerKeyboardState>>,
    pub(in crate::imui::text_picker_controls) visible_candidates: &'a [(usize, Arc<str>)],
    pub(in crate::imui::text_picker_controls) keyboard_navigation: bool,
    pub(in crate::imui::text_picker_controls) keyboard_repeat: bool,
    pub(in crate::imui::text_picker_controls) picker_candidate_visible: bool,
    pub(in crate::imui::text_picker_controls) hide_for_exact_match: bool,
    pub(in crate::imui::text_picker_controls) picker_expanded: bool,
    pub(in crate::imui::text_picker_controls) active_element: Option<GlobalElementId>,
    pub(in crate::imui::text_picker_controls) popup_panel_id: Option<GlobalElementId>,
}

pub(in crate::imui::text_picker_controls) struct BuiltInputTextPickerInputRoot {
    pub(in crate::imui::text_picker_controls) root: AnyElement,
    pub(in crate::imui::text_picker_controls) response: ResponseExt,
}
