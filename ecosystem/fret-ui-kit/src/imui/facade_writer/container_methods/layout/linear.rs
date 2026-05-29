use super::super::*;

pub(in crate::imui::facade_writer) fn horizontal<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    horizontal_with_options(ui, build_focus, HorizontalOptions::default(), f);
}

pub(in crate::imui::facade_writer) fn horizontal_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: HorizontalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| horizontal_container_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(in crate::imui::facade_writer) fn vertical<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    vertical_with_options(ui, build_focus, VerticalOptions::default(), f);
}

pub(in crate::imui::facade_writer) fn vertical_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: VerticalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| vertical_container_element(cx, build_focus, options, f));
    ui.add(element);
}
