use super::types::ColorEditPopupPicker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::controls::color_edit) struct ColorEditPopupRuntimeOptions {
    pub(in crate::controls::color_edit) default_picker: ColorEditPopupPicker,
    pub(in crate::controls::color_edit) picker: ColorEditPopupPicker,
    pub(in crate::controls::color_edit) default_alpha_bar: bool,
    pub(in crate::controls::color_edit) alpha_bar: bool,
}

impl ColorEditPopupRuntimeOptions {
    pub(in crate::controls::color_edit) fn needs_default_sync(self, defaults: Self) -> bool {
        self.default_picker != defaults.default_picker
            || self.default_alpha_bar != defaults.default_alpha_bar
    }

    pub(in crate::controls::color_edit) fn sync_defaults(&mut self, defaults: Self) {
        if self.default_picker != defaults.default_picker {
            self.default_picker = defaults.default_picker;
            self.picker = defaults.picker;
        }
        if self.default_alpha_bar != defaults.default_alpha_bar {
            self.default_alpha_bar = defaults.default_alpha_bar;
            self.alpha_bar = defaults.alpha_bar;
        }
    }
}
