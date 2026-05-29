macro_rules! context_popup_surface_methods {
    () => {
        fn begin_popup_context_menu(
            &mut self,
            id: &str,
            trigger: ResponseExt,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            self.begin_popup_context_menu_with_options(id, trigger, PopupMenuOptions::default(), f)
        }

        fn begin_popup_context_menu_with_options(
            &mut self,
            id: &str,
            trigger: ResponseExt,
            options: PopupMenuOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            floating_popup::begin_popup_context_menu_with_options(self, id, trigger, options, f)
        }
    };
}

pub(crate) use context_popup_surface_methods;
