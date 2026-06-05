mod runtime;
mod types;

pub(in crate::controls::color_edit) use self::runtime::ColorEditPopupRuntimeOptions;
pub use self::types::{
    ColorEditPopupNumericInputs, ColorEditPopupPicker, ColorEditPopupSidePreview,
};

/// Per-control popup defaults for editor `ColorEdit`.
///
/// Dear ImGui stores color edit defaults in the global context via `SetColorEditOptions()`. Fret
/// keeps that policy explicit and app-owned: each editor control receives the popup defaults it
/// should use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEditPopupOptions {
    pub picker: ColorEditPopupPicker,
    pub numeric_inputs: ColorEditPopupNumericInputs,
    pub side_preview: ColorEditPopupSidePreview,
    pub presets: bool,
    pub alpha_bar: bool,
    /// Show popup-local controls for picker shape and AlphaBar visibility.
    ///
    /// This intentionally replaces Dear ImGui's global `ColorPickerOptionsPopup()` state with a
    /// per-control runtime override owned by editor `ColorEdit`.
    pub picker_options: bool,
}

impl ColorEditPopupOptions {
    pub(in crate::controls::color_edit) fn has_visible_content_with_swatches(
        self,
        show_alpha: bool,
        has_palette: bool,
        has_history: bool,
    ) -> bool {
        self.picker != ColorEditPopupPicker::Hidden
            || self.numeric_inputs != ColorEditPopupNumericInputs::Hidden
            || self.side_preview.has_visible_content()
            || (self.presets && has_palette)
            || has_history
            || self.shows_alpha_bar(show_alpha)
            || self.shows_picker_options(show_alpha)
    }

    pub(in crate::controls::color_edit) fn shows_alpha_bar(self, show_alpha: bool) -> bool {
        show_alpha && self.alpha_bar
    }

    pub(in crate::controls::color_edit) fn shows_picker_options(self, show_alpha: bool) -> bool {
        self.picker_options && (self.picker != ColorEditPopupPicker::Hidden || show_alpha)
    }

    pub(in crate::controls::color_edit) fn runtime_defaults(self) -> ColorEditPopupRuntimeOptions {
        ColorEditPopupRuntimeOptions {
            default_picker: self.picker,
            picker: self.picker,
            default_alpha_bar: self.alpha_bar,
            alpha_bar: self.alpha_bar,
        }
    }

    pub(in crate::controls::color_edit) fn with_runtime_options(
        self,
        runtime: ColorEditPopupRuntimeOptions,
    ) -> Self {
        if !self.picker_options {
            return self;
        }

        let mut options = self;
        if self.picker != ColorEditPopupPicker::Hidden
            && runtime.picker != ColorEditPopupPicker::Hidden
        {
            options.picker = runtime.picker;
        }
        options.alpha_bar = runtime.alpha_bar;
        options
    }
}

impl Default for ColorEditPopupOptions {
    fn default() -> Self {
        Self {
            picker: ColorEditPopupPicker::default(),
            numeric_inputs: ColorEditPopupNumericInputs::default(),
            side_preview: ColorEditPopupSidePreview::default(),
            presets: true,
            alpha_bar: true,
            picker_options: true,
        }
    }
}
