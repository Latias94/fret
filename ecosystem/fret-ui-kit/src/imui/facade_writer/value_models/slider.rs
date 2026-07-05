use super::super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn slider_f32_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: impl crate::imui::IntoImUiFloatModel,
    ) -> ResponseExt {
        self.slider_f32_model_with_options(label, model, SliderOptions::default())
    }

    pub fn slider_f32_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: impl crate::imui::IntoImUiFloatModel,
        options: SliderOptions,
    ) -> ResponseExt {
        let model = model.into_imui_float_model();
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::slider_f32_model_with_options(
            self, label, model, options,
        );
        self.record_focusable(resp.id(), focusable);
        resp
    }
}
