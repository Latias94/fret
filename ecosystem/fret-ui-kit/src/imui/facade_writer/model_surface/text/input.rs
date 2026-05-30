macro_rules! input_text_model_surface_methods {
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
    };
}

pub(crate) use input_text_model_surface_methods;
