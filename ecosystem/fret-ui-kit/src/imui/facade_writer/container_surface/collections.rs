macro_rules! collection_surface_methods {
    () => {
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
    };
}

pub(crate) use collection_surface_methods;
