macro_rules! model_surface_methods {
    () => {
        fn checkbox_model(
            &mut self,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<bool>,
        ) -> ResponseExt {
            boolean_controls::checkbox_model(self, label.into(), model)
        }

        fn checkbox_model_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<bool>,
            options: CheckboxOptions,
        ) -> ResponseExt {
            boolean_controls::checkbox_model_with_options(self, label.into(), model, options)
        }

        fn radio(&mut self, label: impl Into<Arc<str>>, selected: bool) -> ResponseExt {
            self.radio_with_options(label, selected, RadioOptions::default())
        }

        fn radio_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            selected: bool,
            options: RadioOptions,
        ) -> ResponseExt {
            boolean_controls::radio_with_options(self, label.into(), selected, options)
        }

        fn switch_model(
            &mut self,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<bool>,
        ) -> ResponseExt {
            self.switch_model_with_options(label, model, SwitchOptions::default())
        }

        fn switch_model_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<bool>,
            options: SwitchOptions,
        ) -> ResponseExt {
            boolean_controls::switch_model_with_options(self, label.into(), model, options)
        }

        fn slider_f32_model(
            &mut self,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<f32>,
        ) -> ResponseExt {
            self.slider_f32_model_with_options(label, model, SliderOptions::default())
        }

        fn slider_f32_model_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<f32>,
            options: SliderOptions,
        ) -> ResponseExt {
            slider_controls::slider_f32_model_with_options(self, label.into(), model, options)
        }

        fn combo_model(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<Option<Arc<str>>>,
            items: &[Arc<str>],
        ) -> ResponseExt {
            self.combo_model_with_options(id, label, model, items, ComboModelOptions::default())
        }

        fn combo_model_with_options(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<Option<Arc<str>>>,
            items: &[Arc<str>],
            options: ComboModelOptions,
        ) -> ResponseExt {
            combo_model_controls::combo_model_with_options(
                self,
                id,
                label.into(),
                model,
                items,
                options,
            )
        }

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

pub(super) use model_surface_methods;
