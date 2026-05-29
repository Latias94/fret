macro_rules! basic_debug_draw_surface_methods {
    () => {
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
    };
}

pub(crate) use basic_debug_draw_surface_methods;
