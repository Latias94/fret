/// Picker surface shown inside the `ColorEdit` popup.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ColorEditPopupPicker {
    /// Dear ImGui's default `PickerHueBar` shape: saturation/value area plus a hue bar.
    #[default]
    HsvHueBar,
    /// Dear ImGui's `PickerHueWheel` shape.
    ///
    /// Use `ColorEditPopupPicker::HsvHueWheel` for a hue wheel plus a rotated saturation/value
    /// triangle.
    HsvHueWheel,
    /// Hide the picker surface while keeping other popup affordances available.
    Hidden,
}

/// Numeric edit rows shown inside the `ColorEdit` popup.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ColorEditPopupNumericInputs {
    /// Show both RGB and HSV numeric rows.
    #[default]
    RgbAndHsv,
    /// Show only the RGB numeric row.
    Rgb,
    /// Show only the HSV numeric row.
    Hsv,
    /// Hide numeric edit rows.
    Hidden,
}

/// Side preview surface shown inside the `ColorEdit` popup.
///
/// Dear ImGui's picker shows a current preview by default and, when a reference color is provided,
/// an original preview that restores the reference when activated. Fret keeps the same behavior as
/// explicit per-control popup policy instead of global picker flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ColorEditPopupSidePreview {
    /// Hide the popup side preview row.
    Hidden,
    /// Show only the current color preview.
    Current,
    /// Show the current color and the reference captured when the popup opened.
    #[default]
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
