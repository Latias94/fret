macro_rules! layout_group_surface_methods {
    () => {
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

pub(crate) use layout_group_surface_methods;
