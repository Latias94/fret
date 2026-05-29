macro_rules! layout_surface_methods {
    () => {
        /// Explicit vertical item-flow convenience for ImGui ports.
        ///
        /// This does not add an implicit layout cursor. It is a scoped vertical group whose default
        /// gap reads `component.imui.item_spacing_y_px` (fallback `4px`).
        fn items(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
            container_methods::items(self, None, f);
        }

        fn items_with_options(
            &mut self,
            options: ItemFlowOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::items_with_options(self, None, options, f);
        }

        /// Explicit horizontal same-line group for ImGui ports.
        ///
        /// This intentionally scopes "same line" to the closure instead of reaching backward to a
        /// previous item. The default gap reads `component.imui.item_spacing_x_px` (fallback `8px`).
        fn same_line(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
            container_methods::same_line(self, None, f);
        }

        fn same_line_with_options(
            &mut self,
            options: SameLineOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::same_line_with_options(self, None, options, f);
        }

        fn dummy(&mut self, size: Size) {
            container_methods::dummy(self, size);
        }

        fn dummy_with_options(&mut self, size: Size, options: DummyOptions) {
            container_methods::dummy_with_options(self, size, options);
        }

        fn spacing(&mut self) {
            container_methods::spacing(self);
        }

        fn spacing_with_options(&mut self, options: SpacingOptions) {
            container_methods::spacing_with_options(self, options);
        }

        fn indent(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
            container_methods::indent(self, None, f);
        }

        fn indent_with_options(
            &mut self,
            options: IndentOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::indent_with_options(self, None, options, f);
        }

        fn horizontal(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
            container_methods::horizontal(self, None, f);
        }

        fn horizontal_with_options(
            &mut self,
            options: HorizontalOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::horizontal_with_options(self, None, options, f);
        }

        fn vertical(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
            container_methods::vertical(self, None, f);
        }

        fn vertical_with_options(
            &mut self,
            options: VerticalOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::vertical_with_options(self, None, options, f);
        }
    };
}

pub(crate) use layout_surface_methods;
