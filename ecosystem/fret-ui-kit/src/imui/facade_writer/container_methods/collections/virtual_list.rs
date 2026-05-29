use super::super::*;

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
