macro_rules! container_surface_methods {
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

        fn menu_bar(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
            container_methods::menu_bar(self, None, f);
        }

        fn menu_bar_with_options(
            &mut self,
            options: MenuBarOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::menu_bar_with_options(self, None, options, f);
        }

        fn tab_bar(
            &mut self,
            id: &str,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
        ) -> TabBarResponse {
            container_methods::tab_bar(self, None, id, f)
        }

        fn tab_bar_with_options(
            &mut self,
            id: &str,
            options: TabBarOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
        ) -> TabBarResponse {
            container_methods::tab_bar_with_options(self, None, id, options, f)
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

        fn list_box(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::list_box(self, None, id, label, f);
        }

        fn list_box_with_options(
            &mut self,
            id: &str,
            options: ListBoxOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::list_box_with_options(self, None, id, options, f);
        }

        fn grid(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
            container_methods::grid(self, None, f);
        }

        fn grid_with_options(
            &mut self,
            options: GridOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::grid_with_options(self, None, options, f);
        }

        fn table(
            &mut self,
            id: &str,
            columns: &[TableColumn],
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
        ) -> TableResponse {
            container_methods::table(self, None, id, columns, f)
        }

        fn table_with_options(
            &mut self,
            id: &str,
            columns: &[TableColumn],
            options: TableOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
        ) -> TableResponse {
            container_methods::table_with_options(self, None, id, columns, options, f)
        }

        fn virtual_list<K, R>(
            &mut self,
            id: &str,
            len: usize,
            key_at: K,
            row: R,
        ) -> VirtualListResponse
        where
            K: FnMut(usize) -> fret_ui::ItemKey,
            R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
        {
            container_methods::virtual_list(self, None, id, len, key_at, row)
        }

        fn virtual_list_with_options<K, R>(
            &mut self,
            id: &str,
            len: usize,
            options: VirtualListOptions,
            key_at: K,
            row: R,
        ) -> VirtualListResponse
        where
            K: FnMut(usize) -> fret_ui::ItemKey,
            R: for<'cx2, 'a2> FnMut(&mut ImUiFacade<'cx2, 'a2, H>, usize),
        {
            container_methods::virtual_list_with_options(self, None, id, len, options, key_at, row)
        }

        fn scroll(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
            container_methods::scroll(self, None, f);
        }

        fn scroll_with_options(
            &mut self,
            options: ScrollOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::scroll_with_options(self, None, options, f);
        }

        fn child_region(
            &mut self,
            id: &str,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> ChildRegionResponse {
            container_methods::child_region(self, None, id, f)
        }

        fn child_region_with_options(
            &mut self,
            id: &str,
            options: ChildRegionOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> ChildRegionResponse {
            container_methods::child_region_with_options(self, None, id, options, f)
        }
    };
}

pub(super) use container_surface_methods;
