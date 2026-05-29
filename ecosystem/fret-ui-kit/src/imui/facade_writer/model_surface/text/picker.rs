macro_rules! picker_text_model_surface_methods {
    () => {
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
    };
}

pub(crate) use picker_text_model_surface_methods;
