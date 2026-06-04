mod policies;
mod popup;

use std::sync::Arc;

use fret_ui::element::{LayoutStyle, Length, SizeStyle};

use super::{
    ColorEditPaletteEntry, OnColorEditEyedropper, OnColorEditPaletteSlotDrop,
    default_color_edit_palette,
};

pub use policies::{
    ColorEditAlphaPreview, ColorEditCopyOptions, ColorEditDragDropOptions, ColorEditTooltipOptions,
};
pub(in crate::controls::color_edit) use popup::ColorEditPopupRuntimeOptions;
pub use popup::{
    ColorEditPopupNumericInputs, ColorEditPopupOptions, ColorEditPopupPicker,
    ColorEditPopupSidePreview,
};

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
