use std::sync::Arc;

use fret_ui::element::{LayoutStyle, Length, SizeStyle};

use super::{
    ColorEditPaletteEntry, OnColorEditEyedropper, OnColorEditPaletteSlotDrop,
    default_color_edit_palette,
};

/// Alpha preview policy for `ColorEdit` swatches.
///
/// Dear ImGui exposes this as `AlphaOpaque`, `AlphaNoBg`, and `AlphaPreviewHalf` flags on
/// `ColorButton` / `ColorEdit`. Fret keeps it as explicit per-control editor policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEditAlphaPreview {
    /// Show transparent colors over a checkerboard background.
    Checkerboard,
    /// Show the current RGB channels as fully opaque in preview only.
    Opaque,
    /// Show the color with its real alpha without a checkerboard background.
    NoBackground,
    /// Split the preview between opaque RGB and transparent checkerboard-backed RGB.
    Half,
}

impl Default for ColorEditAlphaPreview {
    fn default() -> Self {
        Self::Checkerboard
    }
}

/// Per-control color drag/drop policy for editor `ColorEdit`.
///
/// Dear ImGui enables color drag/drop by default and uses `NoDragDrop` as the opt-out flag. Fret
/// keeps the same default for local editor payloads while making cross-window routing explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEditDragDropOptions {
    pub enabled: bool,
    pub cross_window: bool,
}

impl Default for ColorEditDragDropOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            cross_window: false,
        }
    }
}

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

/// Hover tooltip policy for editor `ColorEdit` preview swatches.
///
/// Dear ImGui exposes this as `ImGuiColorEditFlags_NoTooltip`. Fret keeps it as explicit
/// per-control editor policy and avoids global color-edit option state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEditTooltipOptions {
    pub enabled: bool,
}

impl Default for ColorEditTooltipOptions {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Context-menu copy policy for editor `ColorEdit` preview swatches.
///
/// Dear ImGui exposes `Copy as..` inside `ColorEditOptionsPopup()`. Fret keeps the behavior local
/// to the editor control and writes through the existing clipboard effect boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEditCopyOptions {
    pub enabled: bool,
}

impl Default for ColorEditCopyOptions {
    fn default() -> Self {
        Self { enabled: true }
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

#[derive(Clone)]
pub struct ColorEditOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub focusable: bool,
    pub show_alpha: bool,
    pub alpha_preview: ColorEditAlphaPreview,
    pub drag_drop: ColorEditDragDropOptions,
    pub popup: ColorEditPopupOptions,
    pub tooltip: ColorEditTooltipOptions,
    pub copy: ColorEditCopyOptions,
    /// Optional app-owned eyedropper activation hook shown inside the popup.
    ///
    /// Screen sampling is platform/security-sensitive and is not part of the current Fret runtime
    /// contract. Apps that own a native/web eyedropper can opt into this callback and either return
    /// a synchronous sampled color or run an asynchronous flow themselves.
    pub on_eyedropper: Option<OnColorEditEyedropper>,
    /// App-owned palette entries shown by the popup preset row when `popup.presets` is enabled.
    ///
    /// Dear ImGui's custom palette demo stores palette slots in app state. Fret mirrors that
    /// ownership by making the palette data explicit on the editor control options.
    pub palette: Arc<[ColorEditPaletteEntry]>,
    /// App-owned recent color entries shown inside the popup before the palette row.
    ///
    /// Fret does not record a global color history. Apps that want recent colors should keep that
    /// list in their own model and pass it here each frame.
    pub history: Arc<[ColorEditPaletteEntry]>,
    /// Called when a compatible editor color payload is dropped onto a popup palette slot.
    ///
    /// The callback owns the final app-state mutation. When it is absent, palette swatches still
    /// publish RGB drag payloads but do not accept drops as editable slots.
    pub on_palette_slot_drop: Option<OnColorEditPaletteSlotDrop>,
    /// Explicit identity source for internal state (draft/error/open models, overlay root ids).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple color edits from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub swatch_test_id: Option<Arc<str>>,
    pub input_test_id: Option<Arc<str>>,
    pub popup_test_id: Option<Arc<str>>,
    pub tooltip_test_id: Option<Arc<str>>,
    pub copy_menu_test_id: Option<Arc<str>>,
    pub eyedropper_test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for ColorEditOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColorEditOptions")
            .field("layout", &self.layout)
            .field("enabled", &self.enabled)
            .field("focusable", &self.focusable)
            .field("show_alpha", &self.show_alpha)
            .field("alpha_preview", &self.alpha_preview)
            .field("drag_drop", &self.drag_drop)
            .field("popup", &self.popup)
            .field("tooltip", &self.tooltip)
            .field("copy", &self.copy)
            .field(
                "on_eyedropper",
                &self.on_eyedropper.as_ref().map(|_| "<callback>"),
            )
            .field("palette", &self.palette)
            .field("history", &self.history)
            .field(
                "on_palette_slot_drop",
                &self.on_palette_slot_drop.as_ref().map(|_| "<callback>"),
            )
            .field("id_source", &self.id_source)
            .field("test_id", &self.test_id)
            .field("swatch_test_id", &self.swatch_test_id)
            .field("input_test_id", &self.input_test_id)
            .field("popup_test_id", &self.popup_test_id)
            .field("tooltip_test_id", &self.tooltip_test_id)
            .field("copy_menu_test_id", &self.copy_menu_test_id)
            .field("eyedropper_test_id", &self.eyedropper_test_id)
            .finish()
    }
}

impl Default for ColorEditOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            show_alpha: false,
            alpha_preview: ColorEditAlphaPreview::default(),
            drag_drop: ColorEditDragDropOptions::default(),
            popup: ColorEditPopupOptions::default(),
            tooltip: ColorEditTooltipOptions::default(),
            copy: ColorEditCopyOptions::default(),
            on_eyedropper: None,
            palette: default_color_edit_palette(),
            history: Vec::new().into(),
            on_palette_slot_drop: None,
            id_source: None,
            test_id: None,
            swatch_test_id: None,
            input_test_id: None,
            popup_test_id: None,
            tooltip_test_id: None,
            copy_menu_test_id: None,
            eyedropper_test_id: None,
        }
    }
}
