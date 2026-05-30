use super::super::*;

pub(in crate::imui::facade_writer) fn table<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    columns: &[TableColumn],
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
) -> TableResponse
where
    W: UiWriter<H> + ?Sized,
{
    table_with_options(ui, build_focus, id, columns, TableOptions::default(), f)
}

pub(in crate::imui::facade_writer) fn table_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    columns: &[TableColumn],
    options: TableOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
) -> TableResponse
where
    W: UiWriter<H> + ?Sized,
{
    let (element, response) = ui
        .with_cx_mut(|cx| table_controls::table_element(cx, id, columns, build_focus, options, f));
    ui.add(element);
    response
}
