use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::ColorEdit;
use super::super::super::drag_drop::ColorDragDropStore;
use super::super::super::popup::{
    request_color_copy_menu_overlay, request_color_tooltip_overlay, request_popup_overlay,
};
use super::super::super::{
    ColorEditCopyOptions, ColorEditDragDropOptions, ColorEditPopupOptions,
    ColorEditPopupRuntimeOptions, ColorEditTooltipOptions,
};
use super::super::test_ids::ColorEditElementTestIds;

pub(super) struct ColorEditFrameOverlayArgs<'a> {
    pub(super) control: &'a ColorEdit,
    pub(super) swatch_id: GlobalElementId,
    pub(super) open: Model<bool>,
    pub(super) tooltip_open: Model<bool>,
    pub(super) copy_menu_open: Model<bool>,
    pub(super) reference: Model<Option<Color>>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) current: Color,
    pub(super) drag_drop_store: Model<ColorDragDropStore>,
    pub(super) drag_drop_options: ColorEditDragDropOptions,
    pub(super) drag_threshold: Px,
    pub(super) popup_options: ColorEditPopupOptions,
    pub(super) tooltip_options: ColorEditTooltipOptions,
    pub(super) copy_options: ColorEditCopyOptions,
    pub(super) popup_runtime_options: Model<ColorEditPopupRuntimeOptions>,
    pub(super) popup_padding: Px,
    pub(super) row_height: Px,
    pub(super) text_input_chrome: fret_ui::TextInputStyle,
    pub(super) text_input_text_style: fret_core::TextStyle,
    pub(super) error_color: Color,
    pub(super) test_ids: &'a ColorEditElementTestIds,
}

pub(super) fn request_color_edit_frame_overlays<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorEditFrameOverlayArgs<'_>,
) {
    request_popup_overlay(
        cx,
        args.swatch_id,
        args.control.model.clone(),
        args.reference.clone(),
        args.draft.clone(),
        args.error.clone(),
        args.open,
        args.control.options.show_alpha,
        args.control.options.enabled,
        args.control.options.alpha_preview,
        args.control.options.palette.clone(),
        args.control.options.history.clone(),
        args.drag_drop_store,
        args.drag_drop_options,
        args.drag_threshold,
        args.control.options.on_palette_slot_drop.clone(),
        args.control.options.on_eyedropper.clone(),
        args.popup_options,
        args.popup_runtime_options,
        args.popup_padding,
        args.row_height,
        args.text_input_chrome,
        args.text_input_text_style,
        args.error_color,
        args.test_ids.popup.clone(),
        args.test_ids.eyedropper.clone(),
    );
    request_color_tooltip_overlay(
        cx,
        args.swatch_id,
        args.tooltip_open,
        args.current,
        args.control.options.show_alpha,
        args.control.options.alpha_preview,
        args.tooltip_options,
        args.test_ids.tooltip.clone(),
    );
    request_color_copy_menu_overlay(
        cx,
        args.swatch_id,
        args.copy_menu_open,
        args.current,
        args.control.options.show_alpha,
        args.copy_options,
        args.test_ids.copy_menu.clone(),
    );
}
