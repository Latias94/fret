macro_rules! collection_table_surface_methods {
    () => {
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
    };
}

pub(crate) use collection_table_surface_methods;
