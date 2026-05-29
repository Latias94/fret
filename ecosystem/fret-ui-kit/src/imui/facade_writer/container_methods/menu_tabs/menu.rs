use super::super::*;

pub(in crate::imui::facade_writer) fn menu_bar<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    menu_bar_with_options(ui, build_focus, MenuBarOptions::default(), f);
}

pub(in crate::imui::facade_writer) fn menu_bar_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: MenuBarOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element =
        ui.with_cx_mut(|cx| menu_family_controls::menu_bar_element(cx, build_focus, options, f));
    ui.add(element);
}
