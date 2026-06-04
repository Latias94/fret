use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::super::super::drag_drop::ColorDragDropStore;
use super::super::super::{
    ColorEditAlphaPreview, ColorEditDragDropOptions, ColorEditPaletteEntry, ColorEditPopupOptions,
    ColorEditPopupRuntimeOptions, OnColorEditEyedropper, OnColorEditPaletteSlotDrop,
};
use super::layout::ColorPopupContentArgs;

mod actions;
mod picker;
mod preview;
mod swatches;

use actions::{
    color_popup_eyedropper_section, color_popup_numeric_section, color_popup_picker_options_section,
};
use picker::{color_popup_picker_section, color_popup_standalone_alpha_bar_section};
use preview::color_popup_side_preview_section;
use swatches::{color_popup_history_swatches_section, color_popup_preset_swatches_section};

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
    pub(super) popup_test_id: Option<Arc<str>>,
    pub(super) eyedropper_test_id: Option<Arc<str>>,
}

pub(super) struct ColorPopupBodySections {
    pub(super) content: ColorPopupContentArgs,
    pub(super) has_side_preview: bool,
}

pub(super) fn color_popup_body_sections<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorPopupBodySectionsArgs,
) -> ColorPopupBodySections {
    let ColorPopupBodySectionsArgs {
        current,
        reference_color,
        model,
        draft,
        error,
        open,
        rgb_draft,
        hsv_draft,
        numeric_error,
        show_alpha,
        enabled,
        alpha_preview,
        palette,
        history,
        drag_drop_store,
        drag_drop_options,
        drag_threshold,
        on_palette_slot_drop,
        on_eyedropper,
        popup_options,
        popup_runtime_options,
        runtime_options,
        effective_popup_options,
        popup_test_id,
        eyedropper_test_id,
    } = args;

    let picker = color_popup_picker_section(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        show_alpha,
        effective_popup_options,
        enabled,
        popup_test_id.as_ref(),
    );
    let picker_options = color_popup_picker_options_section(
        cx,
        current,
        popup_options,
        runtime_options,
        popup_runtime_options.clone(),
        show_alpha,
        enabled,
        popup_test_id.as_ref(),
    );
    let side_preview = color_popup_side_preview_section(
        cx,
        current,
        reference_color,
        model.clone(),
        draft.clone(),
        error.clone(),
        effective_popup_options,
        show_alpha,
        enabled,
        alpha_preview,
        popup_test_id.as_ref(),
    );
    let has_side_preview = side_preview.is_some();
    let eyedropper = color_popup_eyedropper_section(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        show_alpha,
        enabled,
        on_eyedropper,
        eyedropper_test_id,
    );
    let numbers = color_popup_numeric_section(
        cx,
        current,
        model.clone(),
        draft.clone(),
        rgb_draft.clone(),
        hsv_draft.clone(),
        numeric_error.clone(),
        effective_popup_options,
        show_alpha,
        enabled,
        popup_test_id.as_ref(),
    );
    let history_swatches = color_popup_history_swatches_section(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        open.clone(),
        show_alpha,
        enabled,
        alpha_preview,
        history.clone(),
        drag_drop_store.clone(),
        drag_drop_options,
        drag_threshold,
        popup_test_id.clone(),
    );
    let swatches = color_popup_preset_swatches_section(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        open.clone(),
        show_alpha,
        enabled,
        alpha_preview,
        palette.clone(),
        drag_drop_store.clone(),
        drag_drop_options,
        drag_threshold,
        on_palette_slot_drop.clone(),
        effective_popup_options,
        popup_test_id.clone(),
    );
    let standalone_alpha_bar = color_popup_standalone_alpha_bar_section(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        effective_popup_options,
        show_alpha,
        enabled,
        popup_test_id.as_ref(),
    );

    ColorPopupBodySections {
        content: ColorPopupContentArgs {
            picker,
            side_preview,
            picker_options,
            eyedropper,
            numbers,
            history_swatches,
            swatches,
            standalone_alpha_bar,
        },
        has_side_preview,
    }
}
