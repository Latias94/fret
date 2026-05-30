macro_rules! menu_tab_surface_methods {
    () => {
        fn menu_bar(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
            container_methods::menu_bar(self, None, f);
        }

        fn menu_bar_with_options(
            &mut self,
            options: MenuBarOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) {
            container_methods::menu_bar_with_options(self, None, options, f);
        }

        fn tab_bar(
            &mut self,
            id: &str,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
        ) -> TabBarResponse {
            container_methods::tab_bar(self, None, id, f)
        }

        fn tab_bar_with_options(
            &mut self,
            id: &str,
            options: TabBarOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
        ) -> TabBarResponse {
            container_methods::tab_bar_with_options(self, None, id, options, f)
        }
    };
}

pub(crate) use menu_tab_surface_methods;
