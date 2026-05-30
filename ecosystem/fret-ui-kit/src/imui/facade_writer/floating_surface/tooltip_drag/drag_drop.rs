macro_rules! drag_drop_surface_methods {
    () => {
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

pub(crate) use drag_drop_surface_methods;
