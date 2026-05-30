use super::super::*;

pub(in crate::imui::facade_writer) fn tab_bar<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
) -> TabBarResponse
where
    W: UiWriter<H> + ?Sized,
{
    tab_bar_with_options(ui, build_focus, id, TabBarOptions::default(), f)
}

pub(in crate::imui::facade_writer) fn tab_bar_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    options: TabBarOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
) -> TabBarResponse
where
    W: UiWriter<H> + ?Sized,
{
    let (element, response) =
        ui.with_cx_mut(|cx| tab_family_controls::tab_bar_element(cx, id, build_focus, options, f));
    ui.add(element);
    response
}
