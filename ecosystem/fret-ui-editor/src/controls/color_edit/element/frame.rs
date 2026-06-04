use fret_core::{Color, Px};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::{EditorDensity, EditorTokenKeys};

use super::super::drag_drop::{
    ColorEditDeliveredDropArgs, apply_delivered_color_drop, color_drag_drop_store_for,
    prune_color_drag_drop_store, resolve_color_drag_threshold,
};
use super::super::layout::{ColorEditRootLayoutArgs, color_edit_root_layout};
use super::super::model::format_hex;
use super::super::state::{
    copy_menu_open_model, draft_model, error_model, popup_open_model, popup_runtime_options_model,
    reference_model, sync_popup_runtime_options, tooltip_open_model,
};
use super::ColorEdit;
use super::affordance::color_edit_frame_affordances;
use super::test_ids::color_edit_element_test_ids;

mod children;
mod overlays;

use children::{ColorEditFrameChildrenArgs, color_edit_frame_children};
use overlays::{ColorEditFrameOverlayArgs, request_color_edit_frame_overlays};

pub(super) fn color_edit_into_element_keyed<H: UiHost>(
    control: ColorEdit,
    cx: &mut ElementContext<'_, H>,
) -> AnyElement {
    let open = popup_open_model(cx);
    let tooltip_open = tooltip_open_model(cx);
    let copy_menu_open = copy_menu_open_model(cx);
    let reference = reference_model(cx);
    let draft = draft_model(cx);
    let error = error_model(cx);

    let (density, popup_padding) = {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);
        let popup_padding = theme
            .metric_by_key(EditorTokenKeys::COLOR_POPUP_PADDING)
            .unwrap_or(Px(8.0));
        (density, popup_padding)
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

    let children = color_edit_frame_children(
        cx,
        ColorEditFrameChildrenArgs {
            control: &control,
            open: open.clone(),
            tooltip_open: tooltip_open.clone(),
            copy_menu_open: copy_menu_open.clone(),
            reference: reference.clone(),
            draft: draft.clone(),
            error: error.clone(),
            drag_drop_store: drag_drop_store.clone(),
            current_hex: current_hex.clone(),
            current,
            affordances: &affordances,
            popup_options,
            tooltip_options,
            copy_options,
            drag_drop_options,
            drag_threshold,
            test_ids: &test_ids,
            row_height: density.row_height,
        },
    );
    let input = children.input;
    let swatch = children.swatch;

    apply_delivered_color_drop(
        cx,
        ColorEditDeliveredDropArgs {
            store: drag_drop_store.clone(),
            target_id: swatch.id,
            model: control.model.clone(),
            draft: draft.clone(),
            error: error.clone(),
            current,
            show_alpha: control.options.show_alpha,
            enabled: affordances.drag_drop_enabled,
        },
    );

    request_color_edit_frame_overlays(
        cx,
        ColorEditFrameOverlayArgs {
            control: &control,
            swatch_id: swatch.id,
            open,
            tooltip_open,
            copy_menu_open,
            reference,
            draft,
            error: error.clone(),
            current,
            drag_drop_store,
            drag_drop_options,
            drag_threshold,
            popup_options,
            tooltip_options,
            copy_options,
            popup_runtime_options,
            popup_padding,
            test_ids: &test_ids,
        },
    );

    color_edit_root_layout(
        cx,
        ColorEditRootLayoutArgs {
            swatch,
            input,
            error,
            layout: control.options.layout,
            test_id: control.options.test_id.clone(),
            row_height: density.row_height,
        },
    )
}
