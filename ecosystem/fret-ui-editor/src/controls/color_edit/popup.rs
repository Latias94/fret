use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::OnCloseAutoFocus;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, PointerRegionProps,
    SizeStyle, SpacingLength,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use crate::primitives::input_group::derived_test_id;
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;

use super::drag_drop::ColorDragDropStore;
use super::{
    ColorEditAlphaPreview, ColorEditDragDropOptions, ColorEditPaletteEntry,
    ColorEditPopupNumericInputs, ColorEditPopupOptions, ColorEditPopupPicker,
    ColorEditPopupRuntimeOptions, OnColorEditPaletteSlotDrop, draft_model, error_model,
};

pub(in crate::controls::color_edit) mod copy;
mod eyedropper;
mod numeric;
mod options;
pub(super) mod picker;
pub(super) mod preview;
mod swatches;
pub(in crate::controls::color_edit) mod tooltip;

pub(super) use self::copy::request_color_copy_menu_overlay;
use self::eyedropper::color_eyedropper_action;
use self::numeric::color_numeric_inputs;
use self::options::color_picker_options;
use self::picker::{alpha_bar, hsv_hue_wheel_picker, hsv_picker};
pub(super) use self::preview::color_preview_stack;
use self::preview::color_side_preview;
use self::swatches::{history_swatches, preset_swatches};
pub(super) use self::tooltip::request_color_tooltip_overlay;

pub(super) fn request_popup_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    swatch_id: fret_ui::elements::GlobalElementId,
    model: Model<Color>,
    reference: Model<Option<Color>>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    palette: Arc<[ColorEditPaletteEntry]>,
    history: Arc<[ColorEditPaletteEntry]>,
    drag_drop_store: Model<ColorDragDropStore>,
    drag_drop_options: ColorEditDragDropOptions,
    drag_threshold: Px,
    on_palette_slot_drop: Option<OnColorEditPaletteSlotDrop>,
    on_eyedropper: Option<super::OnColorEditEyedropper>,
    popup_options: ColorEditPopupOptions,
    popup_runtime_options: Model<ColorEditPopupRuntimeOptions>,
    popup_padding: Px,
    popup_test_id: Option<Arc<str>>,
    eyedropper_test_id: Option<Arc<str>>,
) {
    if !popup_options.has_visible_content_with_swatches(
        show_alpha,
        !palette.is_empty(),
        !history.is_empty(),
    ) && on_eyedropper.is_none()
    {
        return;
    }

    let rgb_draft = draft_model(cx);
    let hsv_draft = draft_model(cx);
    let numeric_error = error_model(cx);
    let overlay_id = cx
        .named("color_edit.popup", |cx| cx.spacer(Default::default()))
        .id;
    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let presence = OverlayPresence::instant(is_open);

    let close_focus: OnCloseAutoFocus = Arc::new(move |host, _cx, req| {
        req.prevent_default();
        host.request_focus(swatch_id);
    });

    let placement = popper::PopperContentPlacement::new(
        popper::LayoutDirection::Ltr,
        Side::Bottom,
        Align::Start,
        Px(4.0),
    )
    .with_collision_padding(Edges::all(Px(8.0)));

    let open_for_content = open.clone();
    let popup = cx.anchored_props(
        fret_ui::element::AnchoredProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            outer_margin: Edges::all(Px(0.0)),
            anchor_element: Some(swatch_id.0),
            side: placement.side,
            align: placement.align,
            side_offset: placement.side_offset,
            options: placement.options(),
            ..Default::default()
        },
        move |cx| {
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
            let eyedropper = on_eyedropper.clone().map(|on_eyedropper| {
                color_eyedropper_action(
                    cx,
                    current,
                    model.clone(),
                    draft.clone(),
                    error.clone(),
                    show_alpha,
                    enabled,
                    on_eyedropper,
                    eyedropper_test_id.clone(),
                )
            });
            let numbers = (effective_popup_options.numeric_inputs
                != ColorEditPopupNumericInputs::Hidden)
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
                    open_for_content.clone(),
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
                    open_for_content.clone(),
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
            let standalone_alpha_bar = if effective_popup_options.picker
                == ColorEditPopupPicker::Hidden
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
                move |_cx| {
                    let mut out = Vec::new();
                    if let Some(picker) = picker {
                        out.push(picker);
                    }
                    if let Some(picker_options) = picker_options {
                        out.push(picker_options);
                    }
                    if let Some(side_preview) = side_preview {
                        out.push(side_preview);
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
                            width: Length::Px(Px(216.0)),
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

            let popup = if let Some(test_id) = popup_test_id.as_ref() {
                popup.test_id(test_id.clone())
            } else {
                popup
            };

            vec![popup]
        },
    );

    let mut request = OverlayRequest::dismissible_menu(
        overlay_id,
        swatch_id,
        open,
        presence,
        vec![cx.pointer_region(
            PointerRegionProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                enabled: true,
                capture_phase_pointer_moves: false,
            },
            move |_cx| vec![popup],
        )],
    );
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    request.on_close_auto_focus = Some(close_focus);

    OverlayController::request(cx, request);
}
