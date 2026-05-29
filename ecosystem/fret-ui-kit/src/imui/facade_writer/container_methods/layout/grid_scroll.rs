use super::super::*;

pub(in crate::imui::facade_writer) fn grid<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    grid_with_options(ui, build_focus, GridOptions::default(), f);
}

pub(in crate::imui::facade_writer) fn grid_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: GridOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| grid_container_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(in crate::imui::facade_writer) fn scroll<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    scroll_with_options(ui, build_focus, ScrollOptions::default(), f);
}

pub(in crate::imui::facade_writer) fn scroll_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: ScrollOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| scroll_container_element(cx, build_focus, options, f));
    ui.add(element);
}
