/// Picker surface shown inside the `ColorEdit` popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEditPopupPicker {
    /// Dear ImGui's default `PickerHueBar` shape: saturation/value area plus a hue bar.
    HsvHueBar,
    /// Dear ImGui's `PickerHueWheel` shape.
    ///
    /// Use `ColorEditPopupPicker::HsvHueWheel` for a hue wheel plus a rotated saturation/value
    /// triangle.
    HsvHueWheel,
    /// Hide the picker surface while keeping other popup affordances available.
    Hidden,
}

impl Default for ColorEditPopupPicker {
    fn default() -> Self {
        Self::HsvHueBar
    }
}

/// Numeric edit rows shown inside the `ColorEdit` popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEditPopupNumericInputs {
    /// Show both RGB and HSV numeric rows.
    RgbAndHsv,
    /// Show only the RGB numeric row.
    Rgb,
    /// Show only the HSV numeric row.
    Hsv,
    /// Hide numeric edit rows.
    Hidden,
}

impl Default for ColorEditPopupNumericInputs {
    fn default() -> Self {
        Self::RgbAndHsv
    }
}

/// Side preview surface shown inside the `ColorEdit` popup.
///
/// Dear ImGui's picker shows a current preview by default and, when a reference color is provided,
/// an original preview that restores the reference when activated. Fret keeps the same behavior as
/// explicit per-control popup policy instead of global picker flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEditPopupSidePreview {
    /// Hide the popup side preview row.
    Hidden,
    /// Show only the current color preview.
    Current,
    /// Show the current color and the reference captured when the popup opened.
    CurrentAndOriginal,
}

impl ColorEditPopupSidePreview {
    pub(in crate::controls::color_edit) fn has_visible_content(self) -> bool {
        self != Self::Hidden
    }

    pub(in crate::controls::color_edit) fn shows_original(self) -> bool {
        self == Self::CurrentAndOriginal
    }
}

impl Default for ColorEditPopupSidePreview {
    fn default() -> Self {
        Self::CurrentAndOriginal
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::controls::color_edit) struct ColorEditPopupRuntimeOptions {
    pub(in crate::controls::color_edit) default_picker: ColorEditPopupPicker,
    pub(in crate::controls::color_edit) picker: ColorEditPopupPicker,
    pub(in crate::controls::color_edit) default_alpha_bar: bool,
    pub(in crate::controls::color_edit) alpha_bar: bool,
}

impl ColorEditPopupRuntimeOptions {
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
