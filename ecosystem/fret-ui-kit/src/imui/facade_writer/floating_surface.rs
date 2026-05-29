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

macro_rules! tooltip_drag_surface_methods {
    () => {
        fn tooltip_text(
            &mut self,
            id: &str,
            trigger: ResponseExt,
            text: impl Into<Arc<str>>,
        ) -> bool {
            self.tooltip_text_with_options(id, trigger, text, TooltipOptions::default())
        }

        fn tooltip_text_with_options(
            &mut self,
            id: &str,
            trigger: ResponseExt,
            text: impl Into<Arc<str>>,
            options: TooltipOptions,
        ) -> bool {
            floating_popup::tooltip_text_with_options(self, id, trigger, text, options)
        }

        fn tooltip(
            &mut self,
            id: &str,
            trigger: ResponseExt,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            self.tooltip_with_options(id, trigger, TooltipOptions::default(), f)
        }

        fn tooltip_with_options(
            &mut self,
            id: &str,
            trigger: ResponseExt,
            options: TooltipOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            floating_popup::tooltip_with_options(self, id, trigger, options, f)
        }

        /// Publish a typed payload for the trigger's existing pressable drag gesture.
        ///
        /// Notes:
        /// - This follows Fret's response-driven authoring style instead of cloning Dear ImGui's
        ///   begin/end drag-drop grammar.
        /// - The payload is stored in a model-backed immediate store keyed by the active drag
        ///   session, because object-safe pointer action hooks do not create typed `DragSession`
        ///   payloads directly.
        fn drag_source<T: std::any::Any>(
            &mut self,
            trigger: ResponseExt,
            payload: T,
        ) -> DragSourceResponse {
            self.drag_source_with_options(trigger, payload, DragSourceOptions::default())
        }

        fn drag_source_with_options<T: std::any::Any>(
            &mut self,
            trigger: ResponseExt,
            payload: T,
            options: DragSourceOptions,
        ) -> DragSourceResponse {
            floating_popup::drag_source_with_options(self, trigger, payload, options)
        }

        /// Resolve a typed drop target against the trigger's existing pressable surface.
        ///
        /// Preview state is reported while a compatible payload hovers the target. Delivery is
        /// reported exactly once on the next render after pointer release over the target.
        fn drop_target<T: std::any::Any>(&mut self, trigger: ResponseExt) -> DropTargetResponse<T> {
            self.drop_target_with_options(trigger, DropTargetOptions::default())
        }

        fn drop_target_with_options<T: std::any::Any>(
            &mut self,
            trigger: ResponseExt,
            options: DropTargetOptions,
        ) -> DropTargetResponse<T> {
            floating_popup::drop_target_with_options(self, trigger, options)
        }
    };
}

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

pub(super) use floating_popup_surface_methods;
pub(super) use tooltip_drag_surface_methods;
pub(super) use window_surface_methods;
