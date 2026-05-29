macro_rules! scope_surface_methods {
    () => {
        fn push_id<K: Hash, R>(
            &mut self,
            key: K,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>) -> R,
        ) -> R {
            scope_methods::push_id(self, key, f)
        }

        /// Disable all `imui`-facade interactions within the closure and dim visuals (ImGui-style
        /// `BeginDisabled/EndDisabled`).
        ///
        /// Notes:
        /// - This helper is scoped to the closure (Rust-friendly) rather than a manual begin/end
        ///   pair.
        /// - Nested disabled scopes do not multiply opacity; only the outermost disabled scope
        ///   applies the visual dimming.
        /// - The disabled alpha multiplier is controlled by theme number
        ///   `component.imui.disabled_alpha` (default `0.60`).
        fn disabled_scope(
            &mut self,
            disabled: bool,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            scope_methods::disabled_scope(self, disabled, f);
        }
    };
}

pub(super) use scope_surface_methods;
