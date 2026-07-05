macro_rules! input_text_model_surface_methods {
    () => {
        fn input_text_model(&mut self, model: impl crate::imui::IntoImUiTextModel) -> ResponseExt {
            self.input_text_model_with_options(model, InputTextOptions::default())
        }

        fn input_text_model_with_options(
            &mut self,
            model: impl crate::imui::IntoImUiTextModel,
            options: InputTextOptions,
        ) -> ResponseExt {
            let model = model.into_imui_text_model();
            text_controls::input_text_model_with_options(self, &model, options)
        }
    };
}

pub(crate) use input_text_model_surface_methods;
