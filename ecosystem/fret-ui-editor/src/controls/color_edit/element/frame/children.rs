use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::super::super::ColorEdit;
use super::super::super::drag_drop::ColorDragDropStore;
use super::super::super::input::{ColorEditInputArgs, color_hex_input};
use super::super::super::swatch::{ColorEditSwatchArgs, color_swatch};
use super::super::super::{
    ColorEditCopyOptions, ColorEditDragDropOptions, ColorEditPopupOptions, ColorEditTooltipOptions,
};
use super::super::affordance::ColorEditFrameAffordances;
use super::super::test_ids::ColorEditElementTestIds;

pub(super) struct ColorEditFrameChildren {
    pub(super) input: AnyElement,
    pub(super) swatch: AnyElement,
}

pub(super) struct ColorEditFrameChildrenArgs<'a> {
    pub(super) control: &'a ColorEdit,
    pub(super) open: Model<bool>,
    pub(super) tooltip_open: Model<bool>,
    pub(super) copy_menu_open: Model<bool>,
    pub(super) reference: Model<Option<Color>>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) drag_drop_store: Model<ColorDragDropStore>,
    pub(super) current: Color,
    pub(super) current_hex: Arc<str>,
    pub(super) affordances: &'a ColorEditFrameAffordances,
    pub(super) popup_options: ColorEditPopupOptions,
    pub(super) tooltip_options: ColorEditTooltipOptions,
    pub(super) copy_options: ColorEditCopyOptions,
    pub(super) drag_drop_options: ColorEditDragDropOptions,
    pub(super) drag_threshold: Px,
    pub(super) test_ids: &'a ColorEditElementTestIds,
    pub(super) control_height: Px,
    pub(super) text_input_chrome: fret_ui::TextInputStyle,
    pub(super) text_input_text_style: fret_core::TextStyle,
}

pub(super) fn color_edit_frame_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorEditFrameChildrenArgs<'_>,
) -> ColorEditFrameChildren {
    let input = color_hex_input(
        cx,
        ColorEditInputArgs {
            model: args.control.model.clone(),
            draft: args.draft.clone(),
            error: args.error.clone(),
            current_hex: args.current_hex.clone(),
            show_alpha: args.control.options.show_alpha,
            enabled: args.control.options.enabled,
            focusable: args.control.options.focusable,
            test_id: args.test_ids.input.clone(),
            control_height: args.control_height,
            text_input_chrome: args.text_input_chrome,
            text_input_text_style: args.text_input_text_style,
        },
    );

    let swatch = color_swatch(
        cx,
        ColorEditSwatchArgs {
            model: args.control.model.clone(),
            open: args.open,
            tooltip_open: args.tooltip_open,
            copy_menu_open: args.copy_menu_open,
            reference: args.reference,
            drag_drop_store: args.drag_drop_store,
            current: args.current,
            current_hex: args.current_hex,
            show_alpha: args.control.options.show_alpha,
            alpha_preview: args.control.options.alpha_preview,
            enabled: args.control.options.enabled,
            swatch_enabled: args.affordances.swatch_enabled,
            swatch_focusable: args.affordances.swatch_focusable,
            popup_has_visible_content: args.affordances.popup_has_visible_content,
            popup_options: args.popup_options,
            tooltip_options: args.tooltip_options,
            copy_options: args.copy_options,
            copy_enabled: args.affordances.copy_enabled,
            drag_drop_enabled: args.affordances.drag_drop_enabled,
            drag_drop_options: args.drag_drop_options,
            drag_threshold: args.drag_threshold,
            test_id: args.test_ids.swatch.clone(),
        },
    );

    ColorEditFrameChildren { input, swatch }
}
