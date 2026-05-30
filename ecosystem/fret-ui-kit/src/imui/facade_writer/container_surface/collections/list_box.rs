macro_rules! collection_list_box_surface_methods {
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
    };
}

pub(crate) use collection_list_box_surface_methods;
