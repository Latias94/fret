macro_rules! plain_button_surface_methods {
    () => {
        fn button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
            self.button_with_options(label, ButtonOptions::default())
        }

        fn small_button(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
            self.small_button_with_options(label, ButtonOptions::default())
        }

        fn small_button_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_controls::small_button_with_options(self, label.into(), options)
        }

        fn arrow_button(&mut self, id: &str, direction: ButtonArrowDirection) -> ResponseExt {
            self.arrow_button_with_options(id, direction, ButtonOptions::default())
        }

        fn arrow_button_with_options(
            &mut self,
            id: &str,
            direction: ButtonArrowDirection,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_controls::arrow_button_with_options(self, id, direction, options)
        }

        fn invisible_button(&mut self, id: &str, size: Size) -> ResponseExt {
            self.invisible_button_with_options(id, size, ButtonOptions::default())
        }

        fn invisible_button_with_options(
            &mut self,
            id: &str,
            size: Size,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_controls::invisible_button_with_options(self, id, size, options)
        }

        fn button_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_controls::button_with_options(self, label.into(), options)
        }
    };
}

pub(crate) use plain_button_surface_methods;
