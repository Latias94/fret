use super::*;

pub(in crate::imui::facade_writer) fn menu_item_command_with_options<H, W>(
    ui: &mut W,
    command: CommandId,
    options: MenuItemOptions,
) -> ResponseExt
where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
{
    let presentation =
        ui.with_cx_mut(|cx| crate::command::command_presentation_for_window(cx, &command));

    let mut options = options;
    options.enabled = options.enabled && presentation.enabled;
    if options.shortcut.is_none() {
        options.shortcut = presentation.shortcut;
    }

    menu_controls::menu_item_action_with_options(ui, presentation.label, command, options)
}
