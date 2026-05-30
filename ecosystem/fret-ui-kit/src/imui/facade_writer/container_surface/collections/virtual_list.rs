macro_rules! collection_virtual_list_surface_methods {
    () => {
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

pub(crate) use collection_virtual_list_surface_methods;
