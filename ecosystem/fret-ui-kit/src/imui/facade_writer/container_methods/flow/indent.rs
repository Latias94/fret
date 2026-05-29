use super::super::*;

pub(in crate::imui::facade_writer) fn indent<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    indent_with_options(ui, build_focus, IndentOptions::default(), f);
}

pub(in crate::imui::facade_writer) fn indent_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: IndentOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| layout_sugar::indent_element(cx, build_focus, options, f));
    ui.add(element);
}
