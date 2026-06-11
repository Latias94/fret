use fret_core::{Px, Size};

use super::super::super::super::menus::PopupMenuOptions;
use super::super::input::InputTextOptions;
use super::filter::InputTextPickerFilter;
use super::options::InputTextPickerOptions;

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
