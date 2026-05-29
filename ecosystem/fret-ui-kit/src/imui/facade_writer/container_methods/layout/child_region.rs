use super::super::*;

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
