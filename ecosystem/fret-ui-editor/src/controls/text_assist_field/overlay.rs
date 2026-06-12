//! Anchored text-assist overlay request and placement owner.

use std::sync::Arc;

use fret_core::{Edges, Px, Size};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{AnyElement, Overflow};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};
use fret_ui_kit::primitives::{popper, popper_content};
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use crate::primitives::popup_list::{editor_popup_side_offset, editor_popup_window_margin};

pub(super) fn request_text_assist_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input_id: GlobalElementId,
    field_id: Option<GlobalElementId>,
    open: Model<bool>,
    query_model: Model<String>,
    dismissed_query_model: Model<String>,
    panel: AnyElement,
    surface_height: Px,
) {
    let Some(anchor) = fret_ui_kit::overlay::anchor_bounds_for_element(cx, input_id) else {
        // The panel belongs to the overlay layer. If anchor bounds are not available on this
        // frame, skip it and retry next frame instead of inserting it into layout flow.
        cx.app.request_redraw(cx.window);
        return;
    };
    let outer = fret_ui_kit::overlay::outer_bounds_with_window_margin_for_environment(
        cx,
        Invalidation::Layout,
        editor_popup_window_margin(),
    );
    let placement = popper::PopperContentPlacement::new(
        popper::LayoutDirection::Ltr,
        Side::Bottom,
        Align::Start,
        editor_popup_side_offset(),
    )
    .with_collision_padding(Edges::all(editor_popup_window_margin()));
    let desired = Size::new(anchor.size.width, surface_height);
    let layout = popper::popper_content_layout_sized(outer, anchor, desired, placement);
    cx.diagnostics_record_overlay_placement_placed_rect(
        Some("editor.text_assist"),
        Some(input_id),
        Some(panel.id),
        outer,
        anchor,
        layout.rect,
        Some(layout.side),
    );
    let overlay_panel = popper_content::popper_wrapper_panel_at(
        cx,
        layout.rect,
        Edges::all(Px(0.0)),
        Overflow::Visible,
        move |_cx| vec![panel],
    );

    let overlay_id = cx
        .named("text_assist_field.overlay", |cx| {
            cx.spacer(Default::default())
        })
        .id;
    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let presence = OverlayPresence::instant(is_open);
    let query_model_for_dismiss = query_model.clone();
    let dismissed_query_model_for_dismiss = dismissed_query_model.clone();
    let open_for_dismiss = open.clone();

    let mut request = OverlayRequest::dismissible_popover(
        overlay_id,
        input_id,
        open,
        presence,
        vec![overlay_panel],
    );
    request.root_name = Some(format!("editor.text_assist.{}", input_id.0));
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    if let Some(field_id) = field_id {
        request = request.add_dismissable_branch(field_id);
    }
    request.dismissible_on_dismiss_request =
        Some(Arc::new(move |host, action_cx: ActionCx, _req| {
            let query = host
                .models_mut()
                .read(&query_model_for_dismiss, Clone::clone)
                .ok()
                .unwrap_or_default();
            let _ = host
                .models_mut()
                .update(&dismissed_query_model_for_dismiss, |value| {
                    value.clear();
                    value.push_str(&query);
                });
            let _ = host.models_mut().update(&open_for_dismiss, |value| {
                *value = false;
            });
            host.request_redraw(action_cx.window);
        }));

    OverlayController::request(cx, request);
}

#[track_caller]
pub(super) fn overlay_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    cx.local_model(|| false)
}
