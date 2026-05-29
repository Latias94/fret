use super::super::*;

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
