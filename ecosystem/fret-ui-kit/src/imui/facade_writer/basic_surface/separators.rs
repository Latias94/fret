macro_rules! basic_separator_surface_methods {
    () => {
        fn separator(&mut self) {
            basic_items::separator(self);
        }

        fn separator_text(&mut self, label: impl Into<Arc<str>>) {
            self.separator_text_with_options(label, SeparatorTextOptions::default());
        }

        fn separator_text_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            options: SeparatorTextOptions,
        ) {
            basic_items::separator_text_with_options(self, label.into(), options);
        }
    };
}

pub(crate) use basic_separator_surface_methods;
