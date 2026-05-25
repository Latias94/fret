use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn horizontal(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.horizontal_with_options(HorizontalOptions::default(), f);
    }

    pub fn horizontal_with_options(
        &mut self,
        options: HorizontalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element =
            self.with_cx_mut(|cx| horizontal_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn menu_bar(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.menu_bar_with_options(MenuBarOptions::default(), f);
    }

    pub fn menu_bar_with_options(
        &mut self,
        options: MenuBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self
            .with_cx_mut(|cx| menu_family_controls::menu_bar_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn tab_bar(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        self.tab_bar_with_options(id, TabBarOptions::default(), f)
    }

    pub fn tab_bar_with_options(
        &mut self,
        id: &str,
        options: TabBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        let build_focus = self.build_focus.clone();
        let (element, response) = self.with_cx_mut(|cx| {
            tab_family_controls::tab_bar_element(cx, id, build_focus, options, f)
        });
        self.add(element);
        response
    }

    pub fn vertical(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.vertical_with_options(VerticalOptions::default(), f);
    }

    pub fn vertical_with_options(
        &mut self,
        options: VerticalOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element =
            self.with_cx_mut(|cx| vertical_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn grid(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.grid_with_options(GridOptions::default(), f);
    }

    pub fn grid_with_options(
        &mut self,
        options: GridOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self.with_cx_mut(|cx| grid_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn list_box(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        self.list_box_with_options(
            id,
            ListBoxOptions {
                label: Some(label.into()),
                ..Default::default()
            },
            f,
        );
    }

    pub fn list_box_with_options(
        &mut self,
        id: &str,
        options: ListBoxOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self
            .with_cx_mut(|cx| list_box_controls::list_box_element(cx, id, build_focus, options, f));
        self.add(element);
    }

    pub fn table(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) -> TableResponse {
        self.table_with_options(id, columns, TableOptions::default(), f)
    }

    pub fn table_with_options(
        &mut self,
        id: &str,
        columns: &[TableColumn],
        options: TableOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTable<'cx2, 'a2, H>),
    ) -> TableResponse {
        let build_focus = self.build_focus.clone();
        let (element, response) = self.with_cx_mut(|cx| {
            table_controls::table_element(cx, id, columns, build_focus, options, f)
        });
        self.add(element);
        response
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
        self.virtual_list_with_options(id, len, VirtualListOptions::default(), key_at, row)
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
        let (element, response) = self.with_cx_mut(|cx| {
            virtual_list_controls::virtual_list_element(
                cx,
                id,
                len,
                build_focus,
                options,
                key_at,
                row,
            )
        });
        self.add(element);
        response
    }

    pub fn scroll(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        self.scroll_with_options(ScrollOptions::default(), f);
    }

    pub fn scroll_with_options(
        &mut self,
        options: ScrollOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        let element = self.with_cx_mut(|cx| scroll_container_element(cx, build_focus, options, f));
        self.add(element);
    }

    pub fn child_region(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ChildRegionResponse {
        self.child_region_with_options(id, ChildRegionOptions::default(), f)
    }

    pub fn child_region_with_options(
        &mut self,
        id: &str,
        options: ChildRegionOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> ChildRegionResponse {
        let build_focus = self.build_focus.clone();
        let (element, response) = self
            .with_cx_mut(|cx| child_region::child_region_element(cx, id, build_focus, options, f));
        self.add(element);
        response
    }
}
