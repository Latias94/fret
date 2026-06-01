use std::sync::Arc;

use fret_core::Rect;
use fret_runtime::Model;
use fret_ui::elements::GlobalElementId;
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::primitives::active_descendant as active_desc;

pub(in crate::controls::enum_select::overlay) fn enum_select_viewport_test_id(
    list_test_id: &str,
) -> Arc<str> {
    Arc::from(format!("{list_test_id}.viewport"))
}

pub(in crate::controls::enum_select::overlay) fn reveal_selected_row_if_needed<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    scroll_handle: &ScrollHandle,
    viewport_id: GlobalElementId,
    selected_row_element: Option<GlobalElementId>,
    pending_selected_reveal: &Model<bool>,
) {
    let Some(selected_row_element) = selected_row_element else {
        clear_pending_selected_reveal(cx, pending_selected_reveal);
        return;
    };

    let did_reveal = active_desc::scroll_active_element_into_view_y(
        cx,
        scroll_handle,
        viewport_id,
        selected_row_element,
    );
    let already_visible =
        element_visible_within_viewport_y(cx, viewport_id, selected_row_element).unwrap_or(false);
    if did_reveal || already_visible {
        clear_pending_selected_reveal(cx, pending_selected_reveal);
    }
}

fn clear_pending_selected_reveal<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    pending_selected_reveal: &Model<bool>,
) {
    let _ = cx
        .app
        .models_mut()
        .update(pending_selected_reveal, |pending| *pending = false);
}

fn element_visible_within_viewport_y<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    viewport_element: GlobalElementId,
    child_element: GlobalElementId,
) -> Option<bool> {
    let viewport = cx.last_bounds_for_element(viewport_element)?;
    let child = cx.last_bounds_for_element(child_element)?;
    Some(rect_visible_within_viewport_y(viewport, child))
}

pub(in crate::controls::enum_select::overlay) fn rect_visible_within_viewport_y(
    viewport: Rect,
    child: Rect,
) -> bool {
    let viewport_h = viewport.size.height.0.max(0.0);
    if viewport_h <= 0.0 {
        return false;
    }

    let view_top = viewport.origin.y.0;
    let view_bottom = view_top + viewport_h;
    let child_top = child.origin.y.0;
    let child_h = child.size.height.0.max(0.0);
    let child_bottom = child_top + child_h;

    if child_h >= viewport_h - 0.01 {
        child_top >= view_top - 0.01
    } else {
        child_top >= view_top - 0.01 && child_bottom <= view_bottom + 0.01
    }
}
