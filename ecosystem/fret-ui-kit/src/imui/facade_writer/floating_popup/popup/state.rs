use super::super::*;

pub(in crate::imui::facade_writer) fn popup_open_model<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
) -> fret_runtime::Model<bool> {
    popup_overlay::popup_open_model(ui, id)
}

pub(in crate::imui::facade_writer) fn drop_popup_scope<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
) {
    popup_overlay::drop_popup_scope(ui, id);
}

pub(in crate::imui::facade_writer) fn open_popup<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
) {
    popup_overlay::open_popup(ui, id);
}

pub(in crate::imui::facade_writer) fn open_popup_at<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    anchor: fret_core::Rect,
) {
    popup_overlay::open_popup_at(ui, id, anchor);
}

pub(in crate::imui::facade_writer) fn close_popup<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
) {
    popup_overlay::close_popup(ui, id);
}
