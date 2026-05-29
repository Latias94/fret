macro_rules! floating_popup_surface_methods {
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

        fn begin_popup_menu(
            &mut self,
            id: &str,
            trigger: Option<GlobalElementId>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            self.begin_popup_menu_with_options(id, trigger, PopupMenuOptions::default(), f)
        }

        fn begin_popup_menu_with_options(
            &mut self,
            id: &str,
            trigger: Option<GlobalElementId>,
            options: PopupMenuOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            floating_popup::begin_popup_menu_with_options(self, id, trigger, options, f)
        }

        fn begin_popup_modal(
            &mut self,
            id: &str,
            trigger: Option<GlobalElementId>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            self.begin_popup_modal_with_options(id, trigger, PopupModalOptions::default(), f)
        }

        fn begin_popup_modal_with_options(
            &mut self,
            id: &str,
            trigger: Option<GlobalElementId>,
            options: PopupModalOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            floating_popup::begin_popup_modal_with_options(self, id, trigger, options, f)
        }
    };
}

pub(crate) use floating_popup_surface_methods;
