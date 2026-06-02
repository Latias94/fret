use std::sync::Arc;

use crate::primitives::input_group::derived_test_id;

use super::super::options::ColorEditOptions;

pub(super) struct ColorEditElementTestIds {
    pub(super) input: Option<Arc<str>>,
    pub(super) swatch: Option<Arc<str>>,
    pub(super) popup: Option<Arc<str>>,
    pub(super) tooltip: Option<Arc<str>>,
    pub(super) copy_menu: Option<Arc<str>>,
    pub(super) eyedropper: Option<Arc<str>>,
}

pub(super) fn color_edit_element_test_ids(options: &ColorEditOptions) -> ColorEditElementTestIds {
    ColorEditElementTestIds {
        input: options
            .input_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "input")),
        swatch: options
            .swatch_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "swatch")),
        popup: options
            .popup_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "popup")),
        tooltip: options
            .tooltip_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "tooltip")),
        copy_menu: options
            .copy_menu_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "copy-menu")),
        eyedropper: options
            .eyedropper_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "eyedropper")),
    }
}
