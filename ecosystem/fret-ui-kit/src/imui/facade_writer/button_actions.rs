use super::*;

mod action_methods;
mod button_command;
pub(super) use button_command::button_command_with_options;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::button(self, label);
        let enabled = self.with_cx_mut(|cx| !imui_is_disabled(cx));
        self.record_focusable(resp.id(), enabled);
        resp
    }

    pub fn small_button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::small_button(self, label);
        let enabled = self.with_cx_mut(|cx| !imui_is_disabled(cx));
        self.record_focusable(resp.id(), enabled);
        resp
    }

    pub fn small_button_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: ButtonOptions,
    ) -> ResponseExt {
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::small_button_with_options(self, label, options);
        self.record_focusable(resp.id(), resp.enabled());
        resp
    }

    pub fn arrow_button(&mut self, id: &str, direction: ButtonArrowDirection) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::arrow_button(self, id, direction);
        self.record_focusable(resp.id(), resp.enabled());
        resp
    }

    pub fn arrow_button_with_options(
        &mut self,
        id: &str,
        direction: ButtonArrowDirection,
        options: ButtonOptions,
    ) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::arrow_button_with_options(
            self, id, direction, options,
        );
        self.record_focusable(resp.id(), resp.enabled());
        resp
    }

    pub fn invisible_button(&mut self, id: &str, size: Size) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::invisible_button(self, id, size);
        self.record_focusable(resp.id(), resp.enabled());
        resp
    }

    pub fn invisible_button_with_options(
        &mut self,
        id: &str,
        size: Size,
        options: ButtonOptions,
    ) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::invisible_button_with_options(
            self, id, size, options,
        );
        self.record_focusable(resp.id(), resp.enabled());
        resp
    }

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
