macro_rules! window_surface_methods {
    () => {
        /// Render an in-window floating window.
        ///
        /// Scope:
        /// - in-window (not an OS window / viewport),
        /// - draggable via the title bar,
        /// - position is stored as element-local state under the window id scope,
        /// - `floating_layer(...)` owns bring-to-front ordering and hit-test order,
        /// - `WindowOptions` / `FloatingWindowOptions` own close, resize, collapse,
        ///   focus-on-click, activate-on-click, and no-inputs / pointer-pass-through policy.
        ///
        /// Notes:
        /// - `id` must be stable across frames (mirrors Dear ImGui's "window name is the id" rule).
        /// - OS-window tear-out and multi-viewport behavior are docking/runner concerns, not this
        ///   in-window helper.
        fn window(
            &mut self,
            id: &str,
            title: impl Into<Arc<str>>,
            initial_position: Point,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> FloatingWindowResponse {
            floating_popup::window(self, id, title, initial_position, f)
        }

        /// Render a floating window with explicit state and behavior options.
        fn window_with_options(
            &mut self,
            id: &str,
            title: impl Into<Arc<str>>,
            initial_position: Point,
            options: WindowOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> FloatingWindowResponse {
            floating_popup::window_with_options(self, id, title, initial_position, options, f)
        }
    };
}

pub(crate) use window_surface_methods;
