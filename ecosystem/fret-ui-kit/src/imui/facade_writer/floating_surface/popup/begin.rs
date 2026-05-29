macro_rules! popup_begin_surface_methods {
    () => {
        fn begin_popup_menu(
            &mut self,
            id: &str,
            trigger: Option<GlobalElementId>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            self.begin_popup_menu_with_options(id, trigger, PopupMenuOptions::default(), f)
        }

        fn begin_popup_menu_with_options(
            &mut self,
            id: &str,
            trigger: Option<GlobalElementId>,
            options: PopupMenuOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            floating_popup::begin_popup_menu_with_options(self, id, trigger, options, f)
        }

        fn begin_popup_modal(
            &mut self,
            id: &str,
            trigger: Option<GlobalElementId>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            self.begin_popup_modal_with_options(id, trigger, PopupModalOptions::default(), f)
        }

        fn begin_popup_modal_with_options(
            &mut self,
            id: &str,
            trigger: Option<GlobalElementId>,
            options: PopupModalOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> bool {
            floating_popup::begin_popup_modal_with_options(self, id, trigger, options, f)
        }
    };
}

pub(crate) use popup_begin_surface_methods;
