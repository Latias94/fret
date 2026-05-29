macro_rules! region_surface_methods {
    () => {
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

pub(crate) use region_surface_methods;
