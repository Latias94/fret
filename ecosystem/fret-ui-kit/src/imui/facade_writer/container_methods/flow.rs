use super::*;

pub(in crate::imui::facade_writer) fn items<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    items_with_options(ui, build_focus, ItemFlowOptions::default(), f);
}

pub(in crate::imui::facade_writer) fn items_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: ItemFlowOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| layout_sugar::items_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(in crate::imui::facade_writer) fn same_line<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    same_line_with_options(ui, build_focus, SameLineOptions::default(), f);
}

pub(in crate::imui::facade_writer) fn same_line_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    options: SameLineOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| layout_sugar::same_line_element(cx, build_focus, options, f));
    ui.add(element);
}

pub(in crate::imui::facade_writer) fn dummy<H: UiHost, W>(ui: &mut W, size: Size)
where
    W: UiWriter<H> + ?Sized,
{
    dummy_with_options(ui, size, DummyOptions::default());
}

pub(in crate::imui::facade_writer) fn dummy_with_options<H: UiHost, W>(
    ui: &mut W,
    size: Size,
    options: DummyOptions,
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| layout_sugar::dummy_element(cx, size, options));
    ui.add(element);
}

pub(in crate::imui::facade_writer) fn spacing<H: UiHost, W>(ui: &mut W)
where
    W: UiWriter<H> + ?Sized,
{
    spacing_with_options(ui, SpacingOptions::default());
}

pub(in crate::imui::facade_writer) fn spacing_with_options<H: UiHost, W>(
    ui: &mut W,
    options: SpacingOptions,
) where
    W: UiWriter<H> + ?Sized,
{
    let element = ui.with_cx_mut(|cx| layout_sugar::spacing_element(cx, options));
    ui.add(element);
}

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
