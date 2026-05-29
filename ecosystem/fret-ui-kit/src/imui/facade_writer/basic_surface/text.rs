macro_rules! basic_text_surface_methods {
    () => {
        fn text(&mut self, text: impl Into<Arc<str>>) {
            basic_items::text(self, text.into());
        }

        fn text_wrapped(&mut self, text: impl Into<Arc<str>>) {
            basic_items::text_wrapped(self, text.into());
        }

        fn bullet_text(&mut self, text: impl Into<Arc<str>>) {
            self.bullet_text_with_options(text, BulletTextOptions::default());
        }

        fn bullet_text_with_options(
            &mut self,
            text: impl Into<Arc<str>>,
            options: BulletTextOptions,
        ) {
            basic_items::bullet_text_with_options(self, text.into(), options);
        }
    };
}

pub(crate) use basic_text_surface_methods;
