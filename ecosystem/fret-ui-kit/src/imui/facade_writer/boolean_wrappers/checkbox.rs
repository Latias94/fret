use super::super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn checkbox_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: impl crate::imui::IntoImUiBoolModel,
    ) -> ResponseExt {
        self.checkbox_model_with_options(label, model, CheckboxOptions::default())
    }

    pub fn checkbox_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: impl crate::imui::IntoImUiBoolModel,
        options: CheckboxOptions,
    ) -> ResponseExt {
        let model = model.into_imui_bool_model();
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::checkbox_model_with_options(
            self, label, model, options,
        );
        self.record_focusable(resp.id(), focusable);
        resp
    }
}
