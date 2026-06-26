use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;

use super::super::super::drag_drop::ColorDragDropStore;
use super::super::super::{
    ColorEditAlphaPreview, ColorEditDragDropOptions, ColorEditPaletteEntry, ColorEditPopupOptions,
    ColorEditPopupRuntimeOptions, OnColorEditEyedropper, OnColorEditPaletteSlotDrop,
};
use super::layout::ColorPopupContentArgs;

mod actions;
mod assembly;
mod picker;
mod preview;
mod swatches;

pub(super) use assembly::color_popup_body_sections;

pub(super) struct ColorPopupBodySectionsArgs {
    pub(super) current: Color,
    pub(super) reference_color: Option<Color>,
    pub(super) model: Model<Color>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) open: Model<bool>,
    pub(super) rgb_draft: Model<String>,
    pub(super) hsv_draft: Model<String>,
    pub(super) numeric_error: Model<Option<Arc<str>>>,
    pub(super) show_alpha: bool,
    pub(super) enabled: bool,
    pub(super) alpha_preview: ColorEditAlphaPreview,
    pub(super) palette: Arc<[ColorEditPaletteEntry]>,
    pub(super) history: Arc<[ColorEditPaletteEntry]>,
    pub(super) drag_drop_store: Model<ColorDragDropStore>,
    pub(super) drag_drop_options: ColorEditDragDropOptions,
    pub(super) drag_threshold: Px,
    pub(super) on_palette_slot_drop: Option<OnColorEditPaletteSlotDrop>,
    pub(super) on_eyedropper: Option<OnColorEditEyedropper>,
    pub(super) popup_options: ColorEditPopupOptions,
    pub(super) popup_runtime_options: Model<ColorEditPopupRuntimeOptions>,
    pub(super) runtime_options: ColorEditPopupRuntimeOptions,
    pub(super) effective_popup_options: ColorEditPopupOptions,
    pub(super) row_height: Px,
    pub(super) text_input_chrome: fret_ui::TextInputStyle,
    pub(super) text_input_text_style: fret_core::TextStyle,
    pub(super) error_color: fret_core::Color,
    pub(super) popup_test_id: Option<Arc<str>>,
    pub(super) eyedropper_test_id: Option<Arc<str>>,
}

pub(super) struct ColorPopupBodySections {
    pub(super) content: ColorPopupContentArgs,
    pub(super) has_side_preview: bool,
}
