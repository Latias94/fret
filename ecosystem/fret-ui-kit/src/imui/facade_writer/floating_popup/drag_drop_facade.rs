use super::*;

pub(in crate::imui::facade_writer) fn drag_source_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    T: std::any::Any,
>(
    ui: &mut W,
    trigger: ResponseExt,
    payload: T,
    options: DragSourceOptions,
) -> DragSourceResponse {
    drag_drop::drag_source_with_options(ui, trigger, payload, options)
}

pub(in crate::imui::facade_writer) fn drop_target_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    T: std::any::Any,
>(
    ui: &mut W,
    trigger: ResponseExt,
    options: DropTargetOptions,
) -> DropTargetResponse<T> {
    drag_drop::drop_target_with_options(ui, trigger, options)
}
