use super::super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn switch_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: impl crate::imui::IntoImUiBoolModel,
    ) -> ResponseExt {
        self.switch_model_with_options(label, model, SwitchOptions::default())
    }

    pub fn switch_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: impl crate::imui::IntoImUiBoolModel,
        options: SwitchOptions,
    ) -> ResponseExt {
        let model = model.into_imui_bool_model();
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::switch_model_with_options(
            self, label, model, options,
        );
        self.record_focusable(resp.id(), focusable);
        resp
    }
}
