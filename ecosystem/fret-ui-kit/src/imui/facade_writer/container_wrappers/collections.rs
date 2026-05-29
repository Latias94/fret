use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
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
        container_methods::list_box_with_options(self, build_focus, id, options, f);
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
}
