use super::super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn combo_model(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        model: impl crate::imui::IntoImUiOptionalTextModel,
        items: &[Arc<str>],
    ) -> ResponseExt {
        self.combo_model_with_options(id, label, model, items, ComboModelOptions::default())
    }

    pub fn combo_model_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        model: impl crate::imui::IntoImUiOptionalTextModel,
        items: &[Arc<str>],
        options: ComboModelOptions,
    ) -> ResponseExt {
        let model = model.into_imui_optional_text_model();
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::combo_model_with_options(
            self, id, label, model, items, options,
        );
        self.record_focusable(resp.id(), focusable);
        resp
    }
}
