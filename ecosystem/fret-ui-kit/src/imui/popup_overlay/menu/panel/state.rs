use fret_core::{Rect, Size};
use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{popup_render_generation_for_window, with_popup_store_for_id};

pub(super) struct PopupMenuPanelState {
    pub(super) anchor: Rect,
    pub(super) desired: Size,
}

pub(super) fn prepare_popup_menu_panel_state<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    estimated_size: Size,
) -> Option<PopupMenuPanelState> {
    let (open, anchor_model, panel_id) = with_popup_store_for_id(cx, id, |st, _app| {
        (st.open.clone(), st.anchor.clone(), st.panel_id)
    });
    let is_open = cx
        .read_model(&open, fret_ui::Invalidation::Paint, |_app, v| *v)
        .unwrap_or(false);
    if !is_open {
        return None;
    }

    let anchor = cx
        .read_model(&anchor_model, fret_ui::Invalidation::Paint, |_app, v| *v)
        .unwrap_or(None);
    let Some(anchor) = anchor else {
        close_popup_menu_missing_anchor(cx, id, &open, &anchor_model);
        return None;
    };

    refresh_popup_menu_keep_alive(cx, id);

    let desired = panel_id
        .and_then(|id| cx.last_bounds_for_element(id).map(|r| r.size))
        .unwrap_or(estimated_size);
    Some(PopupMenuPanelState { anchor, desired })
}

pub(super) fn store_popup_menu_panel_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    panel_id: GlobalElementId,
) {
    with_popup_store_for_id(cx, id, |st, _app| st.panel_id = Some(panel_id));
}

fn close_popup_menu_missing_anchor<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    open: &Model<bool>,
    anchor_model: &Model<Option<Rect>>,
) {
    let _ = cx.app.models_mut().update(open, |v| *v = false);
    let _ = cx.app.models_mut().update(anchor_model, |v| *v = None);
    with_popup_store_for_id(cx, id, |st, _app| {
        st.panel_id = None;
        st.keep_alive_generation = None;
    });
    cx.app.request_redraw(cx.window);
}

fn refresh_popup_menu_keep_alive<H: UiHost>(cx: &mut ElementContext<'_, H>, id: &str) {
    let keep_alive_generation = popup_render_generation_for_window(cx);
    with_popup_store_for_id(cx, id, move |st, _app| {
        st.keep_alive_generation = Some(keep_alive_generation);
    });
}
