macro_rules! collection_grid_surface_methods {
    () => {
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
    };
}

pub(crate) use collection_grid_surface_methods;
