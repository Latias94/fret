use super::*;

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

pub(in crate::imui::facade_writer) fn child_region<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> ChildRegionResponse
where
    W: UiWriter<H> + ?Sized,
{
    child_region_with_options(ui, build_focus, id, ChildRegionOptions::default(), f)
}

pub(in crate::imui::facade_writer) fn child_region_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    options: ChildRegionOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> ChildRegionResponse
where
    W: UiWriter<H> + ?Sized,
{
    let (element, response) =
        ui.with_cx_mut(|cx| child_region::child_region_element(cx, id, build_focus, options, f));
    ui.add(element);
    response
}
