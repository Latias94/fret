use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn slider_f32_model(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<f32>,
    ) -> ResponseExt {
        self.slider_f32_model_with_options(label, model, SliderOptions::default())
    }

    pub fn slider_f32_model_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<f32>,
        options: SliderOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::slider_f32_model_with_options(
            self, label, model, options,
        );
        self.record_focusable(resp.id, focusable);
        resp
    }

    pub fn combo_model(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<Option<Arc<str>>>,
        items: &[Arc<str>],
    ) -> ResponseExt {
        self.combo_model_with_options(id, label, model, items, ComboModelOptions::default())
    }

    pub fn combo_model_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        model: &fret_runtime::Model<Option<Arc<str>>>,
        items: &[Arc<str>],
        options: ComboModelOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp = <Self as UiWriterImUiFacadeExt<H>>::combo_model_with_options(
            self, id, label, model, items, options,
        );
        self.record_focusable(resp.id, focusable);
        resp
    }
}
