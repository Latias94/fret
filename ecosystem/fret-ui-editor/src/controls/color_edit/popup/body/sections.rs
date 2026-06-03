use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::input_group::derived_test_id;

use super::super::super::drag_drop::ColorDragDropStore;
use super::super::super::{
    ColorEditAlphaPreview, ColorEditDragDropOptions, ColorEditPaletteEntry,
    ColorEditPopupNumericInputs, ColorEditPopupOptions, ColorEditPopupPicker,
    ColorEditPopupRuntimeOptions, OnColorEditEyedropper, OnColorEditPaletteSlotDrop,
};
use super::super::eyedropper::color_eyedropper_action;
use super::super::numeric::color_numeric_inputs;
use super::super::options::color_picker_options;
use super::super::picker::{alpha_bar, hsv_hue_wheel_picker, hsv_picker};
use super::super::preview::color_side_preview;
use super::super::swatches::{history_swatches, preset_swatches};
use super::layout::ColorPopupContentArgs;

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

    let picker = match effective_popup_options.picker {
        ColorEditPopupPicker::HsvHueBar => Some(hsv_picker(
            cx,
            current,
            model.clone(),
            draft.clone(),
            error.clone(),
            show_alpha,
            effective_popup_options.shows_alpha_bar(show_alpha),
            enabled,
            derived_test_id(popup_test_id.as_ref(), "hsv"),
        )),
        ColorEditPopupPicker::HsvHueWheel => Some(hsv_hue_wheel_picker(
            cx,
            current,
            model.clone(),
            draft.clone(),
            error.clone(),
            show_alpha,
            effective_popup_options.shows_alpha_bar(show_alpha),
            enabled,
            derived_test_id(popup_test_id.as_ref(), "hsv-wheel"),
        )),
        ColorEditPopupPicker::Hidden => None,
    };
    let picker_options = popup_options.shows_picker_options(show_alpha).then(|| {
        color_picker_options(
            cx,
            current,
            popup_options,
            runtime_options,
            popup_runtime_options.clone(),
            show_alpha,
            enabled,
            derived_test_id(popup_test_id.as_ref(), "options"),
        )
    });
    let side_preview = effective_popup_options
        .side_preview
        .has_visible_content()
        .then(|| {
            color_side_preview(
                cx,
                current,
                reference_color,
                model.clone(),
                draft.clone(),
                error.clone(),
                effective_popup_options.side_preview,
                show_alpha,
                enabled,
                alpha_preview,
                derived_test_id(popup_test_id.as_ref(), "preview"),
            )
        });
    let has_side_preview = side_preview.is_some();
    let eyedropper = on_eyedropper.map(|on_eyedropper| {
        color_eyedropper_action(
            cx,
            current,
            model.clone(),
            draft.clone(),
            error.clone(),
            show_alpha,
            enabled,
            on_eyedropper,
            eyedropper_test_id,
        )
    });
    let numbers = (effective_popup_options.numeric_inputs != ColorEditPopupNumericInputs::Hidden)
        .then(|| {
            color_numeric_inputs(
                cx,
                current,
                model.clone(),
                draft.clone(),
                rgb_draft.clone(),
                hsv_draft.clone(),
                numeric_error.clone(),
                effective_popup_options.numeric_inputs,
                show_alpha,
                enabled,
                derived_test_id(popup_test_id.as_ref(), "numbers"),
            )
        });
    let history_swatches = (!history.is_empty()).then(|| {
        history_swatches(
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
        )
    });
    let swatches = (effective_popup_options.presets && !palette.is_empty()).then(|| {
        preset_swatches(
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
            popup_test_id.clone(),
        )
    });
    let standalone_alpha_bar = if effective_popup_options.picker == ColorEditPopupPicker::Hidden
        && effective_popup_options.shows_alpha_bar(show_alpha)
    {
        Some(alpha_bar(
            cx,
            current,
            model.clone(),
            draft.clone(),
            error.clone(),
            enabled,
            derived_test_id(popup_test_id.as_ref(), "alpha"),
        ))
    } else {
        None
    };

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
