macro_rules! textarea_text_model_surface_methods {
    () => {
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

pub(crate) use textarea_text_model_surface_methods;
