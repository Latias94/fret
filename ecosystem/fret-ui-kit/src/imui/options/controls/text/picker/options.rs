use std::sync::Arc;

use super::super::super::super::menus::PopupMenuOptions;
use super::super::input::InputTextOptions;
use super::filter::InputTextPickerFilter;

#[derive(Debug, Clone)]
pub struct InputTextPickerOptions {
    pub input: InputTextOptions,
    pub popup: PopupMenuOptions,
    pub filter: InputTextPickerFilter,
    pub max_items: usize,
    pub open_on_focus: bool,
    pub open_when_empty: bool,
    pub hide_when_exact_match: bool,
    pub keyboard_navigation: bool,
    pub keyboard_repeat: bool,
    pub test_id: Option<Arc<str>>,
}
