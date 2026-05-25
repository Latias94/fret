use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn items(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::items(self, build_focus, f);
    }

    pub fn items_with_options(
        &mut self,
        options: ItemFlowOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::items_with_options(self, build_focus, options, f);
    }

    pub fn same_line(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::same_line(self, build_focus, f);
    }

    pub fn same_line_with_options(
        &mut self,
        options: SameLineOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::same_line_with_options(self, build_focus, options, f);
    }

    pub fn dummy(&mut self, size: Size) {
        container_methods::dummy(self, size);
    }

    pub fn dummy_with_options(&mut self, size: Size, options: DummyOptions) {
        container_methods::dummy_with_options(self, size, options);
    }

    pub fn spacing(&mut self) {
        container_methods::spacing(self);
    }

    pub fn spacing_with_options(&mut self, options: SpacingOptions) {
        container_methods::spacing_with_options(self, options);
    }

    pub fn indent(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::indent(self, build_focus, f);
    }

    pub fn indent_with_options(
        &mut self,
        options: IndentOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::indent_with_options(self, build_focus, options, f);
    }

    pub fn horizontal(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::horizontal(self, build_focus, f);
    }

    pub fn horizontal_with_options(
        &mut self,
        options: HorizontalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::horizontal_with_options(self, build_focus, options, f);
    }

    pub fn menu_bar(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::menu_bar(self, build_focus, f);
    }

    pub fn menu_bar_with_options(
        &mut self,
        options: MenuBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::menu_bar_with_options(self, build_focus, options, f);
    }

    pub fn tab_bar(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        let build_focus = self.build_focus.clone();
        container_methods::tab_bar(self, build_focus, id, f)
    }

    pub fn tab_bar_with_options(
        &mut self,
        id: &str,
        options: TabBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        let build_focus = self.build_focus.clone();
        container_methods::tab_bar_with_options(self, build_focus, id, options, f)
    }

    pub fn vertical(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::vertical(self, build_focus, f);
    }

    pub fn vertical_with_options(
        &mut self,
        options: VerticalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::vertical_with_options(self, build_focus, options, f);
    }

    pub fn grid(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::grid(self, build_focus, f);
    }

    pub fn grid_with_options(
        &mut self,
        options: GridOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::grid_with_options(self, build_focus, options, f);
    }

    pub fn table(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) -> TableResponse {
        let build_focus = self.build_focus.clone();
        container_methods::table(self, build_focus, id, columns, f)
    }

    pub fn table_with_options(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        options: TableOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) -> TableResponse {
        let build_focus = self.build_focus.clone();
        container_methods::table_with_options(self, build_focus, id, columns, options, f)
    }

    pub fn virtual_list<K, R>(
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
        let build_focus = self.build_focus.clone();
        container_methods::virtual_list(self, build_focus, id, len, key_at, row)
    }

    pub fn virtual_list_with_options<K, R>(
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
        let build_focus = self.build_focus.clone();
        container_methods::virtual_list_with_options(
            self,
            build_focus,
            id,
            len,
            options,
            key_at,
            row,
        )
    }

    pub fn scroll(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::scroll(self, build_focus, f);
    }

    pub fn scroll_with_options(
        &mut self,
        options: ScrollOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::scroll_with_options(self, build_focus, options, f);
    }

    pub fn child_region(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ChildRegionResponse {
        let build_focus = self.build_focus.clone();
        container_methods::child_region(self, build_focus, id, f)
    }

    pub fn child_region_with_options(
        &mut self,
        id: &str,
        options: ChildRegionOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ChildRegionResponse {
        let build_focus = self.build_focus.clone();
        container_methods::child_region_with_options(self, build_focus, id, options, f)
    }
}
