macro_rules! action_button_surface_methods {
    () => {
        fn action_button(
            &mut self,
            label: impl Into<Arc<str>>,
            action: impl Into<ActionId>,
        ) -> ResponseExt {
            self.action_button_with_options(label, action, ButtonOptions::default())
        }

        fn action_button_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            action: impl Into<ActionId>,
            options: ButtonOptions,
        ) -> ResponseExt {
            button_controls::action_button_with_options(self, label.into(), action.into(), options)
        }
    };
}

pub(crate) use action_button_surface_methods;
