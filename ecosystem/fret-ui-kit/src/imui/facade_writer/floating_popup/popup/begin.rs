use super::super::*;

pub(in crate::imui::facade_writer) fn begin_popup_menu_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    popup_overlay::begin_popup_menu_with_options(ui, id, trigger, options, false, f)
}

pub(in crate::imui::facade_writer) fn begin_popup_modal_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupModalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    popup_overlay::begin_popup_modal_with_options(ui, id, trigger, options, f)
}

pub(in crate::imui::facade_writer) fn begin_popup_context_menu_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    options: PopupMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    popup_overlay::begin_popup_context_menu_with_options(ui, id, trigger, options, f)
}
