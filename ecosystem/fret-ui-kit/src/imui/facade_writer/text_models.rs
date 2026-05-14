use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn input_text_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
        self.input_text_model_with_options(model, InputTextOptions::default())
    }

    pub fn input_text_model_with_options(
        &mut self,
        model: &fret_runtime::Model<String>,
        options: InputTextOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::input_text_model_with_options(self, model, options);
        self.record_focusable(resp.id(), focusable);
        resp
    }

    pub fn input_text_completion_model(
        &mut self,
        id: &str,
        model: &fret_runtime::Model<String>,
        candidates: &[Arc<str>],
    ) -> InputTextPickerResponse {
        self.input_text_completion_model_with_options(
            id,
            model,
            candidates,
            InputTextPickerOptions::default(),
        )
    }

    pub fn input_text_completion_model_with_options(
        &mut self,
        id: &str,
        model: &fret_runtime::Model<String>,
        candidates: &[Arc<str>],
        options: InputTextPickerOptions,
    ) -> InputTextPickerResponse {
        let focusable = options.input.enabled
            && options.input.focusable
            && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp = <Self as UiWriterImUiFacadeExt<H>>::input_text_completion_model_with_options(
            self, id, model, candidates, options,
        );
        self.record_focusable(resp.id(), focusable);
        resp
    }

    pub fn input_text_history_model(
        &mut self,
        id: &str,
        model: &fret_runtime::Model<String>,
        history: &[Arc<str>],
    ) -> InputTextPickerResponse {
        self.input_text_history_model_with_options(
            id,
            model,
            history,
            InputTextPickerOptions::default(),
        )
    }

    pub fn input_text_history_model_with_options(
        &mut self,
        id: &str,
        model: &fret_runtime::Model<String>,
        history: &[Arc<str>],
        options: InputTextPickerOptions,
    ) -> InputTextPickerResponse {
        let focusable = options.input.enabled
            && options.input.focusable
            && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp = <Self as UiWriterImUiFacadeExt<H>>::input_text_history_model_with_options(
            self, id, model, history, options,
        );
        self.record_focusable(resp.id(), focusable);
        resp
    }

    pub fn textarea_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
        self.textarea_model_with_options(model, TextAreaOptions::default())
    }

    pub fn textarea_model_with_options(
        &mut self,
        model: &fret_runtime::Model<String>,
        options: TextAreaOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let focusable = enabled && options.focusable;
        let resp =
            <Self as UiWriterImUiFacadeExt<H>>::textarea_model_with_options(self, model, options);
        self.record_focusable(resp.id(), focusable);
        resp
    }
}
