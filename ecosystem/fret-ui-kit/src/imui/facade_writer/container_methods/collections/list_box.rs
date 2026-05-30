use super::super::*;

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
