use super::super::*;

pub(in crate::imui::facade_writer) fn button_command_with_options<H, W>(
    ui: &mut W,
    command: CommandId,
    options: ButtonOptions,
) -> ResponseExt
where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
{
    let presentation =
        ui.with_cx_mut(|cx| crate::command::command_presentation_for_window(cx, &command));

    let mut options = options;
    options.enabled = options.enabled && presentation.enabled;

    button_controls::action_button_with_options(ui, presentation.label, command, options)
}
