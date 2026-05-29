macro_rules! value_combo_model_surface_methods {
    () => {
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
    };
}

pub(crate) use value_combo_model_surface_methods;
