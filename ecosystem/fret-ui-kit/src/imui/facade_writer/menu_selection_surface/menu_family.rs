macro_rules! menu_family_surface_methods {
    () => {
        fn begin_menu(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> DisclosureResponse {
            self.begin_menu_with_options(id, label, BeginMenuOptions::default(), f)
        }

        fn begin_menu_with_options(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            options: BeginMenuOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> DisclosureResponse {
            menu_family_controls::begin_menu_with_options(self, id, label.into(), options, f)
        }

        fn begin_submenu(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> DisclosureResponse {
            self.begin_submenu_with_options(id, label, BeginSubmenuOptions::default(), f)
        }

        fn begin_submenu_with_options(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            options: BeginSubmenuOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> DisclosureResponse {
            menu_family_controls::begin_submenu_with_options(self, id, label.into(), options, f)
        }
    };
}

pub(crate) use menu_family_surface_methods;
