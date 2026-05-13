use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn checkbox_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        self.checkbox_model_with_options(label, model, CheckboxOptions::default())
    }

    pub fn checkbox_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
        options: CheckboxOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::checkbox_model_with_options(
            self, label, model, options,
        );
        self.record_focusable(resp.id, focusable);
        resp
    }

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
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn switch_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
    ) -> ResponseExt {
        self.switch_model_with_options(label, model, SwitchOptions::default())
    }

    pub fn switch_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<bool>,
        options: SwitchOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::switch_model_with_options(
            self, label, model, options,
        );
        self.record_focusable(resp.id, focusable);
        resp
    }
}
