use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn action_button(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
    ) -> ResponseExt {
        self.action_button_with_options(label, action, ButtonOptions::default())
    }

    pub fn action_button_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
        options: ButtonOptions,
    ) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::action_button_with_options(
            self, label, action, options,
        );
        self.record_focusable(resp.id(), resp.enabled());
        resp
    }

    pub fn action_payload_button<T>(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
        payload: T,
    ) -> ResponseExt
    where
        T: std::any::Any + Clone + Send + Sync + 'static,
    {
        self.action_payload_button_with_options(label, action, payload, ButtonOptions::default())
    }

    pub fn action_payload_button_with_options<T>(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
        payload: T,
        options: ButtonOptions,
    ) -> ResponseExt
    where
        T: std::any::Any + Clone + Send + Sync + 'static,
    {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::action_payload_button_with_options(
            self, label, action, payload, options,
        );
        self.record_focusable(resp.id(), resp.enabled());
        resp
    }
}
