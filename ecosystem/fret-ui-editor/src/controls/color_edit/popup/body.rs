use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::input_group::derived_test_id;
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;

use super::super::drag_drop::ColorDragDropStore;
use super::super::{
    ColorEditAlphaPreview, ColorEditDragDropOptions, ColorEditPaletteEntry,
    ColorEditPopupNumericInputs, ColorEditPopupOptions, ColorEditPopupPicker,
    ColorEditPopupRuntimeOptions, OnColorEditEyedropper, OnColorEditPaletteSlotDrop,
};
use super::eyedropper::color_eyedropper_action;
use super::numeric::color_numeric_inputs;
use super::options::color_picker_options;
use super::picker::{alpha_bar, hsv_hue_wheel_picker, hsv_picker};
use super::preview::color_side_preview;
use super::swatches::{history_swatches, preset_swatches};

const COLOR_POPUP_WIDTH: Px = Px(216.0);
const COLOR_POPUP_WITH_SIDE_PREVIEW_WIDTH: Px = Px(272.0);

pub(super) struct ColorPopupBodyArgs {
    pub(super) model: Model<Color>,
    pub(super) reference: Model<Option<Color>>,
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
    pub(super) popup_padding: Px,
    pub(super) popup_test_id: Option<Arc<str>>,
    pub(super) eyedropper_test_id: Option<Arc<str>>,
}

pub(super) fn color_popup_body<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorPopupBodyArgs,
) -> AnyElement {
    let ColorPopupBodyArgs {
        model,
        reference,
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
        popup_padding,
        popup_test_id,
        eyedropper_test_id,
    } = args;

    let popup_chrome = {
        let theme = Theme::global(&*cx.app);
        resolve_editor_popup_surface_chrome(theme, true)
    };
    let current = cx
        .get_model_copied(&model, Invalidation::Paint)
        .unwrap_or(Color::TRANSPARENT);
    let reference_color = cx
        .get_model_copied(&reference, Invalidation::Paint)
        .unwrap_or(None);
    let runtime_options = cx
        .get_model_copied(&popup_runtime_options, Invalidation::Paint)
        .unwrap_or_else(|| popup_options.runtime_defaults());
    let effective_popup_options = popup_options.with_runtime_options(runtime_options);
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
    let popup_width = if effective_popup_options.picker != ColorEditPopupPicker::Hidden
        && effective_popup_options.side_preview.has_visible_content()
    {
        COLOR_POPUP_WITH_SIDE_PREVIEW_WIDTH
    } else {
        COLOR_POPUP_WIDTH
    };
    let content = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(8.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            let mut out = Vec::new();
            match (picker, side_preview) {
                (Some(picker), Some(side_preview)) => {
                    out.push(picker_side_preview_row(cx, picker, side_preview));
                }
                (Some(picker), None) => out.push(picker),
                (None, Some(side_preview)) => out.push(side_preview),
                (None, None) => {}
            }
            if let Some(picker_options) = picker_options {
                out.push(picker_options);
            }
            if let Some(eyedropper) = eyedropper {
                out.push(eyedropper);
            }
            if let Some(numbers) = numbers {
                out.push(numbers);
            }
            if let Some(history_swatches) = history_swatches {
                out.push(history_swatches);
            }
            if let Some(swatches) = swatches {
                out.push(swatches);
            }
            if let Some(standalone_alpha_bar) = standalone_alpha_bar {
                out.push(standalone_alpha_bar);
            }
            out
        },
    );
    let popup = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(popup_width),
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(popup_padding).into(),
            background: Some(popup_chrome.bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(popup_chrome.border),
            corner_radii: Corners::all(popup_chrome.radius),
            shadow: popup_chrome.shadow,
            ..Default::default()
        },
        move |_cx| vec![content],
    );

    if let Some(test_id) = popup_test_id.as_ref() {
        popup.test_id(test_id.clone())
    } else {
        popup
    }
}

fn picker_side_preview_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    picker: AnyElement,
    side_preview: AnyElement,
) -> AnyElement {
    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(8.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Start,
            wrap: false,
        },
        move |_cx| vec![picker, side_preview],
    )
}
