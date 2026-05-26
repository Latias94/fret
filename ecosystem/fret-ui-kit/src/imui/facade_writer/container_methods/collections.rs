use super::*;

pub(in crate::imui::facade_writer) fn list_box<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    label: impl Into<Arc<str>>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    list_box_with_options(
        ui,
        build_focus,
        id,
        ListBoxOptions {
            label: Some(label.into()),
            ..Default::default()
        },
        f,
    );
}

pub(in crate::imui::facade_writer) fn list_box_with_options<H: UiHost, W>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    options: ListBoxOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) where
    W: UiWriter<H> + ?Sized,
{
    let element =
        ui.with_cx_mut(|cx| list_box_controls::list_box_element(cx, id, build_focus, options, f));
    ui.add(element);
}

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

pub(in crate::imui::facade_writer) fn virtual_list<H: UiHost, W, K, R>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    len: usize,
    key_at: K,
    row: R,
) -> VirtualListResponse
where
    W: UiWriter<H> + ?Sized,
    K: FnMut(usize) -> fret_ui::ItemKey,
    R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
{
    virtual_list_with_options(
        ui,
        build_focus,
        id,
        len,
        VirtualListOptions::default(),
        key_at,
        row,
    )
}

pub(in crate::imui::facade_writer) fn virtual_list_with_options<H: UiHost, W, K, R>(
    ui: &mut W,
    build_focus: BuildFocus,
    id: &str,
    len: usize,
    options: VirtualListOptions,
    key_at: K,
    row: R,
) -> VirtualListResponse
where
    W: UiWriter<H> + ?Sized,
    K: FnMut(usize) -> fret_ui::ItemKey,
    R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
{
    let (element, response) = ui.with_cx_mut(|cx| {
        virtual_list_controls::virtual_list_element(cx, id, len, build_focus, options, key_at, row)
    });
    ui.add(element);
    response
}
