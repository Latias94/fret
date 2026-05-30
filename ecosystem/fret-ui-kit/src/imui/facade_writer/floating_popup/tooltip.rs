use super::*;

pub(in crate::imui::facade_writer) fn tooltip_text_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    text: impl Into<Arc<str>>,
    options: TooltipOptions,
) -> bool {
    tooltip_overlay::tooltip_text_with_options(ui, id, trigger, text.into(), options)
}

pub(in crate::imui::facade_writer) fn tooltip_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    options: TooltipOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    tooltip_overlay::tooltip_with_options(ui, id, trigger, options, f)
}
