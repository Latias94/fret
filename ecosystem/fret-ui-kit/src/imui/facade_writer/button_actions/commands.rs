use super::super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn button_command(&mut self, command: impl Into<CommandId>) -> ResponseExt {
        self.button_command_with_options(command, ButtonOptions::default())
    }

    pub fn button_command_with_options(
        &mut self,
        command: impl Into<CommandId>,
        options: ButtonOptions,
    ) -> ResponseExt {
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::button_command_with_options(self, command, options);
        self.record_focusable(resp.id(), resp.enabled());
        resp
    }
}
