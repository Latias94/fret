use fret_core::{Px, Rect, Size};
use fret_ui::{GlobalElementId, UiHost};

use super::{ImUiFacade, PopupMenuOptions, PopupModalOptions, ResponseExt, UiWriterImUiFacadeExt};

mod menu;
mod modal;

pub(in crate::imui) use menu::{ImUiMenuNavState, ImUiPopupMenuPolicyState};

pub(super) fn popup_open_model<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) -> fret_runtime::Model<bool> {
    ui.with_cx_mut(|cx| super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone()))
}

pub(super) fn drop_popup_scope<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) {
    ui.with_cx_mut(|cx| super::drop_popup_scope_for_id(cx, id));
}

pub(super) fn open_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(ui: &mut W, id: &str) {
    ui.with_cx_mut(|cx| {
        let keep_alive_generation = super::popup_render_generation_for_window(cx);
        let open = super::with_popup_store_for_id(cx, id, move |st, _app| {
            st.keep_alive_generation = Some(keep_alive_generation);
            st.open.clone()
        });
        let _ = cx.app.models_mut().update(&open, |v| *v = true);
        cx.app.request_redraw(cx.window);
    });
}

pub(super) fn open_popup_at<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    anchor: Rect,
) {
    ui.with_cx_mut(|cx| {
        let keep_alive_generation = super::popup_render_generation_for_window(cx);
        let (open, anchor_model) = super::with_popup_store_for_id(cx, id, move |st, _app| {
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

pub(super) fn close_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(ui: &mut W, id: &str) {
    ui.with_cx_mut(|cx| {
        let open = super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
        let _ = cx.app.models_mut().update(&open, |v| *v = false);
        cx.app.request_redraw(cx.window);
    });
}

pub(super) fn begin_popup_menu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupMenuOptions,
    preserve_focus_outside_while_submenu_open: bool,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    menu::begin_popup_menu_with_options(
        ui,
        id,
        trigger,
        options,
        preserve_focus_outside_while_submenu_open,
        f,
    )
}

pub(super) fn begin_popup_modal_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupModalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    modal::begin_popup_modal_with_options(ui, id, trigger, options, f)
}

pub(super) fn begin_popup_context_menu_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    options: PopupMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    if trigger.context_menu_requested() {
        let anchor = trigger
            .context_menu_anchor()
            .map(|p| Rect::new(p, Size::new(Px(1.0), Px(1.0))))
            .or(trigger.rect());
        if let Some(anchor) = anchor {
            open_popup_at(ui, id, anchor);
        }
    }

    begin_popup_menu_with_options(ui, id, trigger.id(), options, false, f)
}
