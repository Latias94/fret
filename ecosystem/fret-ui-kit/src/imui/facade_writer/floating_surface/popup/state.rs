macro_rules! popup_state_surface_methods {
    () => {
        /// Returns the internal open model for a named popup scope.
        ///
        /// This is intended to support ImGui-like `OpenPopup` / `BeginPopup` splits without
        /// forcing callers to allocate a dedicated `Model<bool>` per popup.
        fn popup_open_model(&mut self, id: &str) -> fret_runtime::Model<bool> {
            floating_popup::popup_open_model(self, id)
        }

        /// Drops all internal state for a named popup scope.
        ///
        /// This is primarily intended for ephemeral/dynamic scopes where the id space could grow
        /// without bound (e.g. popups keyed by user-generated strings). Dropping a scope will close
        /// the popup (if open) and release the internal models if no other references exist.
        fn drop_popup_scope(&mut self, id: &str) {
            floating_popup::drop_popup_scope(self, id);
        }

        fn open_popup(&mut self, id: &str) {
            floating_popup::open_popup(self, id);
        }

        fn open_popup_at(&mut self, id: &str, anchor: fret_core::Rect) {
            floating_popup::open_popup_at(self, id, anchor);
        }

        fn close_popup(&mut self, id: &str) {
            floating_popup::close_popup(self, id);
        }
    };
}

pub(crate) use popup_state_surface_methods;
