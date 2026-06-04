use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use super::super::drag_drop::{ColorEditDeliveredDropArgs, apply_delivered_color_drop};
use super::super::layout::{ColorEditRootLayoutArgs, color_edit_root_layout};
use super::ColorEdit;

mod children;
mod overlays;
mod setup;

use children::{ColorEditFrameChildrenArgs, color_edit_frame_children};
use overlays::{ColorEditFrameOverlayArgs, request_color_edit_frame_overlays};
use setup::color_edit_frame_setup;

pub(super) fn color_edit_into_element_keyed<H: UiHost>(
    control: ColorEdit,
    cx: &mut ElementContext<'_, H>,
) -> AnyElement {
    let setup = color_edit_frame_setup(cx, &control);

    let children = color_edit_frame_children(
        cx,
        ColorEditFrameChildrenArgs {
            control: &control,
            open: setup.open.clone(),
            tooltip_open: setup.tooltip_open.clone(),
            copy_menu_open: setup.copy_menu_open.clone(),
            reference: setup.reference.clone(),
            draft: setup.draft.clone(),
            error: setup.error.clone(),
            drag_drop_store: setup.drag_drop_store.clone(),
            current_hex: setup.current_hex.clone(),
            current: setup.current,
            affordances: &setup.affordances,
            popup_options: setup.popup_options,
            tooltip_options: setup.tooltip_options,
            copy_options: setup.copy_options,
            drag_drop_options: setup.drag_drop_options,
            drag_threshold: setup.drag_threshold,
            test_ids: &setup.test_ids,
            row_height: setup.row_height,
        },
    );
    let input = children.input;
    let swatch = children.swatch;

    apply_delivered_color_drop(
        cx,
        ColorEditDeliveredDropArgs {
            store: setup.drag_drop_store.clone(),
            target_id: swatch.id,
            model: control.model.clone(),
            draft: setup.draft.clone(),
            error: setup.error.clone(),
            current: setup.current,
            show_alpha: control.options.show_alpha,
            enabled: setup.affordances.drag_drop_enabled,
        },
    );

    request_color_edit_frame_overlays(
        cx,
        ColorEditFrameOverlayArgs {
            control: &control,
            swatch_id: swatch.id,
            open: setup.open.clone(),
            tooltip_open: setup.tooltip_open.clone(),
            copy_menu_open: setup.copy_menu_open.clone(),
            reference: setup.reference.clone(),
            draft: setup.draft.clone(),
            error: setup.error.clone(),
            current: setup.current,
            drag_drop_store: setup.drag_drop_store.clone(),
            drag_drop_options: setup.drag_drop_options,
            drag_threshold: setup.drag_threshold,
            popup_options: setup.popup_options,
            tooltip_options: setup.tooltip_options,
            copy_options: setup.copy_options,
            popup_runtime_options: setup.popup_runtime_options.clone(),
            popup_padding: setup.popup_padding,
            test_ids: &setup.test_ids,
        },
    );

    color_edit_root_layout(
        cx,
        ColorEditRootLayoutArgs {
            swatch,
            input,
            error: setup.error,
            layout: control.options.layout,
            test_id: control.options.test_id.clone(),
            row_height: setup.row_height,
        },
    )
}
