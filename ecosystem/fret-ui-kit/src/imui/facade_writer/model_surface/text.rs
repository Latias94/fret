macro_rules! text_model_surface_methods {
    () => {
        fn input_text_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
            self.input_text_model_with_options(model, InputTextOptions::default())
        }

        fn input_text_model_with_options(
            &mut self,
            model: &fret_runtime::Model<String>,
            options: InputTextOptions,
        ) -> ResponseExt {
            text_controls::input_text_model_with_options(self, model, options)
        }

        fn input_text_completion_model(
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

        fn input_text_completion_model_with_options(
            &mut self,
            id: &str,
            model: &fret_runtime::Model<String>,
            candidates: &[Arc<str>],
            options: InputTextPickerOptions,
        ) -> InputTextPickerResponse {
            text_picker_controls::input_text_completion_model_with_options(
                self, id, model, candidates, options,
            )
        }

        fn input_text_history_model(
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

        fn input_text_history_model_with_options(
            &mut self,
            id: &str,
            model: &fret_runtime::Model<String>,
            history: &[Arc<str>],
            options: InputTextPickerOptions,
        ) -> InputTextPickerResponse {
            text_picker_controls::input_text_history_model_with_options(
                self, id, model, history, options,
            )
        }

        fn textarea_model(&mut self, model: &fret_runtime::Model<String>) -> ResponseExt {
            self.textarea_model_with_options(model, TextAreaOptions::default())
        }

        fn textarea_model_with_options(
            &mut self,
            model: &fret_runtime::Model<String>,
            options: TextAreaOptions,
        ) -> ResponseExt {
            text_controls::textarea_model_with_options(self, model, options)
        }
    };
}

pub(crate) use text_model_surface_methods;
