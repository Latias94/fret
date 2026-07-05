use super::super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn input_text_model(&mut self, model: impl crate::imui::IntoImUiTextModel) -> ResponseExt {
        self.input_text_model_with_options(model, InputTextOptions::default())
    }

    pub fn input_text_model_with_options(
        &mut self,
        model: impl crate::imui::IntoImUiTextModel,
        options: InputTextOptions,
    ) -> ResponseExt {
        let model = model.into_imui_text_model();
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::input_text_model_with_options(self, model, options);
        self.record_focusable(resp.id(), focusable);
        resp
    }
}
