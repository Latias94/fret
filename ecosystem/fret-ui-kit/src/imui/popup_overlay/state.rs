use fret_core::Rect;
use fret_ui::UiHost;

use super::super::UiWriterImUiFacadeExt;

pub(in crate::imui) fn popup_open_model<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) -> fret_runtime::Model<bool> {
    ui.with_cx_mut(|cx| super::super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone()))
}

pub(in crate::imui) fn drop_popup_scope<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) {
    ui.with_cx_mut(|cx| super::super::drop_popup_scope_for_id(cx, id));
}

pub(in crate::imui) fn open_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) {
    ui.with_cx_mut(|cx| {
        let keep_alive_generation = super::super::popup_render_generation_for_window(cx);
        let open = super::super::with_popup_store_for_id(cx, id, move |st, _app| {
            st.keep_alive_generation = Some(keep_alive_generation);
            st.open.clone()
        });
        let _ = cx.app.models_mut().update(&open, |v| *v = true);
        cx.app.request_redraw(cx.window);
    });
}

pub(in crate::imui) fn open_popup_at<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    anchor: Rect,
) {
    ui.with_cx_mut(|cx| {
        let keep_alive_generation = super::super::popup_render_generation_for_window(cx);
        let (open, anchor_model) =
            super::super::with_popup_store_for_id(cx, id, move |st, _app| {
                st.keep_alive_generation = Some(keep_alive_generation);
                (st.open.clone(), st.anchor.clone())
            });
        let _ = cx
            .app
            .models_mut()
            .update(&anchor_model, |v| *v = Some(anchor));
        let _ = cx.app.models_mut().update(&open, |v| *v = true);
        cx.app.request_redraw(cx.window);
    });
}

pub(in crate::imui) fn close_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) {
    ui.with_cx_mut(|cx| {
        let open = super::super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
        let _ = cx.app.models_mut().update(&open, |v| *v = false);
        cx.app.request_redraw(cx.window);
    });
}
