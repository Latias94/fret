use fret_core::{Color, Px};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::{EditorDensity, EditorTokenKeys};

use super::super::drag_drop::{
    ColorEditDeliveredDropArgs, apply_delivered_color_drop, color_drag_drop_store_for,
    prune_color_drag_drop_store, resolve_color_drag_threshold,
};
use super::super::input::{ColorEditInputArgs, color_hex_input};
use super::super::layout::{ColorEditRootLayoutArgs, color_edit_root_layout};
use super::super::model::format_hex;
use super::super::popup::{
    request_color_copy_menu_overlay, request_color_tooltip_overlay, request_popup_overlay,
};
use super::super::state::{
    copy_menu_open_model, draft_model, error_model, popup_open_model, popup_runtime_options_model,
    reference_model, sync_popup_runtime_options, tooltip_open_model,
};
use super::super::swatch::{ColorEditSwatchArgs, color_swatch};
use super::ColorEdit;
use super::affordance::color_edit_frame_affordances;
use super::test_ids::color_edit_element_test_ids;

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
    let on_eyedropper = control.options.on_eyedropper.clone();
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
        on_eyedropper.is_some(),
    );

    let input = color_hex_input(
        cx,
        ColorEditInputArgs {
            model: control.model.clone(),
            draft: draft.clone(),
            error: error.clone(),
            current_hex: current_hex.clone(),
            show_alpha: control.options.show_alpha,
            enabled: control.options.enabled,
            focusable: control.options.focusable,
            test_id: test_ids.input.clone(),
            row_height: density.row_height,
        },
    );

    let swatch = color_swatch(
        cx,
        ColorEditSwatchArgs {
            model: control.model.clone(),
            open: open.clone(),
            tooltip_open: tooltip_open.clone(),
            copy_menu_open: copy_menu_open.clone(),
            reference: reference.clone(),
            drag_drop_store: drag_drop_store.clone(),
            current,
            current_hex: current_hex.clone(),
            show_alpha: control.options.show_alpha,
            alpha_preview: control.options.alpha_preview,
            enabled: control.options.enabled,
            swatch_enabled: affordances.swatch_enabled,
            swatch_focusable: affordances.swatch_focusable,
            popup_has_visible_content: affordances.popup_has_visible_content,
            popup_options,
            tooltip_options,
            copy_options,
            copy_enabled: affordances.copy_enabled,
            drag_drop_enabled: affordances.drag_drop_enabled,
            drag_drop_options,
            drag_threshold,
            test_id: test_ids.swatch.clone(),
        },
    );

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

    request_popup_overlay(
        cx,
        swatch.id,
        control.model.clone(),
        reference.clone(),
        draft.clone(),
        error.clone(),
        open.clone(),
        control.options.show_alpha,
        control.options.enabled,
        control.options.alpha_preview,
        palette,
        history,
        drag_drop_store.clone(),
        drag_drop_options,
        drag_threshold,
        control.options.on_palette_slot_drop.clone(),
        on_eyedropper,
        popup_options,
        popup_runtime_options,
        popup_padding,
        test_ids.popup,
        test_ids.eyedropper,
    );
    request_color_tooltip_overlay(
        cx,
        swatch.id,
        tooltip_open,
        current,
        control.options.show_alpha,
        control.options.alpha_preview,
        tooltip_options,
        test_ids.tooltip,
    );
    request_color_copy_menu_overlay(
        cx,
        swatch.id,
        copy_menu_open,
        current,
        control.options.show_alpha,
        copy_options,
        test_ids.copy_menu,
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
