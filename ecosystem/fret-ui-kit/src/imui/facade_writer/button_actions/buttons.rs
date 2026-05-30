use super::super::*;

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
}
