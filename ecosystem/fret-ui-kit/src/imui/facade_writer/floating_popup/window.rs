use super::*;

pub(in crate::imui::facade_writer) fn window<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    initial_position: Point,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window::floating_window_show(ui, id, title, initial_position, f)
}

pub(in crate::imui::facade_writer) fn window_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    title: impl Into<Arc<str>>,
    initial_position: Point,
    options: WindowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> FloatingWindowResponse {
    floating_window::floating_window_show_with_options(ui, id, title, initial_position, options, f)
}
