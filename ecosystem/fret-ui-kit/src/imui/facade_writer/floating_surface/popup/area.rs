macro_rules! floating_area_surface_methods {
    () => {
        /// Render a window-scoped floating window layer that manages z-order (bring-to-front).
        ///
        /// Notes:
        /// - This is an opt-in container; a plain `floating_area(...)` / `window(...)` call
        ///   sequence keeps call-order z.
        /// - Call this late in the parent tree to ensure the layer paints above base content.
        fn floating_layer(
            &mut self,
            id: &str,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            floating_popup::floating_layer(self, id, f);
        }

        /// Render a minimal in-window floating area primitive.
        ///
        /// This is the lowest-level building block for ImGui-like floating surfaces in-window:
        ///
        /// - always in-window (not an OS window / viewport),
        /// - position is stored as element-local state under the area id scope,
        /// - movement is driven by a caller-provided drag surface (via
        ///   `floating_area_drag_surface(...)`),
        /// - optional z-order activation when nested inside `floating_layer(...)`.
        ///
        /// Notes:
        /// - `id` must be stable across frames (mirrors Dear ImGui's "name is the id" rule).
        fn floating_area(
            &mut self,
            id: &str,
            initial_position: Point,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
        ) -> FloatingAreaResponse {
            self.floating_area_with_options(id, initial_position, FloatingAreaOptions::default(), f)
        }

        fn floating_area_with_options(
            &mut self,
            id: &str,
            initial_position: Point,
            options: FloatingAreaOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, FloatingAreaContext),
        ) -> FloatingAreaResponse {
            floating_popup::floating_area_with_options(self, id, initial_position, options, f)
        }

        /// Build a drag surface that moves a floating area (ImGui-style).
        ///
        /// The returned element should be placed as part of the area content (e.g. a title bar).
        fn floating_area_drag_surface(
            &mut self,
            area: FloatingAreaContext,
            props: PointerRegionProps,
            setup: impl FnOnce(&mut ElementContext<'_, H>, GlobalElementId),
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> AnyElement {
            floating_popup::floating_area_drag_surface(self, area, props, setup, f)
        }
    };
}

pub(crate) use floating_area_surface_methods;
