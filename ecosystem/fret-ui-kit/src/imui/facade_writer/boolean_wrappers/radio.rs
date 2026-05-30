use super::super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn radio(&mut self, label: impl Into<Arc<str>>, selected: bool) -> ResponseExt {
        self.radio_with_options(label, selected, RadioOptions::default())
    }

    pub fn radio_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        selected: bool,
        options: RadioOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::radio_with_options(self, label, selected, options);
        self.record_focusable(resp.id(), focusable);
        resp
    }
}
