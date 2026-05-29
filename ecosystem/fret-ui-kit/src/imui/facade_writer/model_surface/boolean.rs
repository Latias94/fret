macro_rules! boolean_model_surface_methods {
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
    };
}

pub(crate) use boolean_model_surface_methods;
