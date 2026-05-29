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

pub(crate) use tooltip_drag_surface_methods;
