macro_rules! basic_surface_methods {
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

        fn debug_draw<K: Hash>(
            &mut self,
            id: K,
            draw: impl FnOnce(&mut ImUiDebugDrawList),
        ) -> DebugDrawResponse {
            self.debug_draw_with_options(id, DebugDrawOptions::default(), draw)
        }

        fn debug_draw_with_options<K: Hash>(
            &mut self,
            id: K,
            options: DebugDrawOptions,
            draw: impl FnOnce(&mut ImUiDebugDrawList),
        ) -> DebugDrawResponse {
            basic_items::debug_draw_with_options(self, id, options, draw)
        }

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

pub(super) use basic_surface_methods;
