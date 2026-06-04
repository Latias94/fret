use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::drag_drop::ColorDragDropStore;
use super::super::super::super::{
    ColorEditAlphaPreview, ColorEditDragDropOptions, ColorEditPaletteEntry, ColorEditPopupOptions,
    OnColorEditPaletteSlotDrop,
};
use super::super::super::swatches::{history_swatches, preset_swatches};

pub(super) fn color_popup_history_swatches_section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    history: Arc<[ColorEditPaletteEntry]>,
    drag_drop_store: Model<ColorDragDropStore>,
    drag_drop_options: ColorEditDragDropOptions,
    drag_threshold: Px,
    popup_test_id: Option<Arc<str>>,
) -> Option<AnyElement> {
    (!history.is_empty()).then(|| {
        history_swatches(
            cx,
            current,
            model,
            draft,
            error,
            open,
            show_alpha,
            enabled,
            alpha_preview,
            history,
            drag_drop_store,
            drag_drop_options,
            drag_threshold,
            popup_test_id,
        )
    })
}

pub(super) fn color_popup_preset_swatches_section<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    palette: Arc<[ColorEditPaletteEntry]>,
    drag_drop_store: Model<ColorDragDropStore>,
    drag_drop_options: ColorEditDragDropOptions,
    drag_threshold: Px,
    on_palette_slot_drop: Option<OnColorEditPaletteSlotDrop>,
    effective_popup_options: ColorEditPopupOptions,
    popup_test_id: Option<Arc<str>>,
) -> Option<AnyElement> {
    (effective_popup_options.presets && !palette.is_empty()).then(|| {
        preset_swatches(
            cx,
            current,
            model,
            draft,
            error,
            open,
            show_alpha,
            enabled,
            alpha_preview,
            palette,
            drag_drop_store,
            drag_drop_options,
            drag_threshold,
            on_palette_slot_drop,
            popup_test_id,
        )
    })
}
