use std::sync::Arc;

use fret_core::{Color, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::EditorTokenKeys;
use crate::primitives::chrome::resolve_editor_text_field_style;
use crate::primitives::style::EditorStyle;

use super::super::super::ColorEdit;
use super::super::super::ColorEditPopupRuntimeOptions;
use super::super::super::drag_drop::{
    ColorDragDropStore, color_drag_drop_store_for, prune_color_drag_drop_store,
    resolve_color_drag_threshold,
};
use super::super::super::model::format_hex;
use super::super::super::state::{
    copy_menu_open_model, draft_model, error_model, popup_open_model, popup_runtime_options_model,
    reference_model, sync_popup_runtime_options, tooltip_open_model,
};
use super::super::super::{
    ColorEditCopyOptions, ColorEditDragDropOptions, ColorEditPopupOptions, ColorEditTooltipOptions,
};
use super::super::affordance::{ColorEditFrameAffordances, color_edit_frame_affordances};
use super::super::test_ids::{ColorEditElementTestIds, color_edit_element_test_ids};

pub(super) struct ColorEditFrameSetup {
    pub(super) open: Model<bool>,
    pub(super) tooltip_open: Model<bool>,
    pub(super) copy_menu_open: Model<bool>,
    pub(super) reference: Model<Option<Color>>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) row_height: Px,
    pub(super) control_height: Px,
    pub(super) popup_padding: Px,
    pub(super) text_input_chrome: fret_ui::TextInputStyle,
    pub(super) text_input_text_style: TextStyle,
    pub(super) error_color: Color,
    pub(super) current: Color,
    pub(super) current_hex: Arc<str>,
    pub(super) drag_drop_store: Model<ColorDragDropStore>,
    pub(super) drag_drop_options: ColorEditDragDropOptions,
    pub(super) drag_threshold: Px,
    pub(super) test_ids: ColorEditElementTestIds,
    pub(super) popup_options: ColorEditPopupOptions,
    pub(super) tooltip_options: ColorEditTooltipOptions,
    pub(super) copy_options: ColorEditCopyOptions,
    pub(super) popup_runtime_options: Model<ColorEditPopupRuntimeOptions>,
    pub(super) affordances: ColorEditFrameAffordances,
}

pub(super) fn color_edit_frame_setup<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    control: &ColorEdit,
) -> ColorEditFrameSetup {
    let open = popup_open_model(cx);
    let tooltip_open = tooltip_open_model(cx);
    let copy_menu_open = copy_menu_open_model(cx);
    let reference = reference_model(cx);
    let draft = draft_model(cx);
    let error = error_model(cx);

    let (
        row_height,
        control_height,
        popup_padding,
        text_input_chrome,
        text_input_text_style,
        error_color,
    ) = {
        let theme = Theme::global(&*cx.app);
        let style = EditorStyle::resolve(theme);
        let (text_input_chrome, text_input_text_style) =
            resolve_editor_text_field_style(theme, Size::default(), &ChromeRefinement::default());
        let popup_padding = theme
            .metric_by_key(EditorTokenKeys::COLOR_POPUP_PADDING)
            .unwrap_or(Px(8.0));
        (
            style.density.row_height,
            style
                .frame_chrome_small()
                .control_outer_height(style.density.row_height),
            popup_padding,
            text_input_chrome,
            text_input_text_style,
            theme.color_token("destructive"),
        )
    };

    let current = cx
        .get_model_copied(&control.model, Invalidation::Paint)
        .unwrap_or(Color::TRANSPARENT);
    let current_hex = format_hex(current, control.options.show_alpha);
    let drag_drop_store = color_drag_drop_store_for(cx);
    prune_color_drag_drop_store(cx, &drag_drop_store);
    let drag_drop_options = control.options.drag_drop;
    let drag_threshold = resolve_color_drag_threshold(cx);
    let test_ids = color_edit_element_test_ids(&control.options);
    let popup_options = control.options.popup;
    let tooltip_options = control.options.tooltip;
    let copy_options = control.options.copy;
    let eyedropper_available = control.options.on_eyedropper.is_some();
    let popup_runtime_options = popup_runtime_options_model(cx, popup_options.runtime_defaults());
    sync_popup_runtime_options(cx, &popup_runtime_options, popup_options.runtime_defaults());
    let popup_options_for_frame = popup_options.with_runtime_options(
        cx.get_model_copied(&popup_runtime_options, Invalidation::Paint)
            .unwrap_or_else(|| popup_options.runtime_defaults()),
    );
    let palette = control.options.palette.clone();
    let history = control.options.history.clone();
    let affordances = color_edit_frame_affordances(
        &control.options,
        popup_options_for_frame,
        !palette.is_empty(),
        !history.is_empty(),
        eyedropper_available,
    );

    ColorEditFrameSetup {
        open,
        tooltip_open,
        copy_menu_open,
        reference,
        draft,
        error,
        row_height,
        control_height,
        popup_padding,
        text_input_chrome,
        text_input_text_style,
        error_color,
        current,
        current_hex,
        drag_drop_store,
        drag_drop_options,
        drag_threshold,
        test_ids,
        popup_options,
        tooltip_options,
        copy_options,
        popup_runtime_options,
        affordances,
    }
}
