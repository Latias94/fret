use std::sync::Arc;

use fret_core::{Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::OnCloseAutoFocus;
use fret_ui::element::{LayoutStyle, Length, PointerRegionProps, SizeStyle};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use super::drag_drop::ColorDragDropStore;
use super::state::{draft_model, error_model};
use super::{
    ColorEditAlphaPreview, ColorEditDragDropOptions, ColorEditPaletteEntry, ColorEditPopupOptions,
    ColorEditPopupRuntimeOptions, OnColorEditPaletteSlotDrop,
};

mod body;
pub(in crate::controls::color_edit) mod copy;
mod eyedropper;
mod numeric;
mod options;
pub(super) mod picker;
pub(super) mod preview;
mod swatches;
pub(in crate::controls::color_edit) mod tooltip;

use self::body::{ColorPopupBodyArgs, color_popup_body};
pub(super) use self::copy::request_color_copy_menu_overlay;
pub(super) use self::preview::color_preview_stack;
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
            vec![color_popup_body(
                cx,
                ColorPopupBodyArgs {
                    model: model.clone(),
                    reference: reference.clone(),
                    draft: draft.clone(),
                    error: error.clone(),
                    open: open_for_content.clone(),
                    rgb_draft: rgb_draft.clone(),
                    hsv_draft: hsv_draft.clone(),
                    numeric_error: numeric_error.clone(),
                    show_alpha,
                    enabled,
                    alpha_preview,
                    palette: palette.clone(),
                    history: history.clone(),
                    drag_drop_store: drag_drop_store.clone(),
                    drag_drop_options,
                    drag_threshold,
                    on_palette_slot_drop: on_palette_slot_drop.clone(),
                    on_eyedropper: on_eyedropper.clone(),
                    popup_options,
                    popup_runtime_options: popup_runtime_options.clone(),
                    popup_padding,
                    popup_test_id: popup_test_id.clone(),
                    eyedropper_test_id: eyedropper_test_id.clone(),
                },
            )]
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
