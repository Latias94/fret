use std::sync::Arc;

use fret_core::{Px, Size};

use super::super::super::menus::PopupMenuOptions;
use super::input::InputTextOptions;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputTextPickerFilter {
    #[default]
    ContainsCaseInsensitive,
    PrefixCaseInsensitive,
    None,
}

impl InputTextPickerFilter {
    pub fn matches(self, query: &str, candidate: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        match self {
            Self::None => true,
            Self::PrefixCaseInsensitive => candidate
                .to_lowercase()
                .starts_with(query.to_lowercase().as_str()),
            Self::ContainsCaseInsensitive => candidate
                .to_lowercase()
                .contains(query.to_lowercase().as_str()),
        }
    }
}

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

impl Default for InputTextPickerOptions {
    fn default() -> Self {
        Self {
            input: InputTextOptions::default(),
            popup: PopupMenuOptions {
                modal: false,
                auto_focus: false,
                estimated_size: Size::new(Px(220.0), Px(160.0)),
                ..PopupMenuOptions::default()
            },
            filter: InputTextPickerFilter::ContainsCaseInsensitive,
            max_items: 8,
            open_on_focus: true,
            open_when_empty: false,
            hide_when_exact_match: true,
            keyboard_navigation: true,
            keyboard_repeat: false,
            test_id: None,
        }
    }
}
