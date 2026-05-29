macro_rules! menu_item_surface_methods {
    () => {
        fn menu_separator(&mut self) {
            self.separator();
        }

        fn menu_item(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
            self.menu_item_with_options(label, MenuItemOptions::default())
        }

        fn menu_item_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            options: MenuItemOptions,
        ) -> ResponseExt {
            menu_controls::menu_item_with_options(self, label.into(), options)
        }

        fn menu_item_checkbox_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            checked: bool,
            options: MenuItemOptions,
        ) -> ResponseExt {
            menu_controls::menu_item_checkbox_with_options(self, label.into(), checked, options)
        }

        fn menu_item_radio_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            checked: bool,
            options: MenuItemOptions,
        ) -> ResponseExt {
            menu_controls::menu_item_radio_with_options(self, label.into(), checked, options)
        }

        fn menu_item_action(
            &mut self,
            label: impl Into<Arc<str>>,
            action: impl Into<ActionId>,
        ) -> ResponseExt {
            self.menu_item_action_with_options(label, action, MenuItemOptions::default())
        }

        fn menu_item_action_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            action: impl Into<ActionId>,
            options: MenuItemOptions,
        ) -> ResponseExt {
            menu_controls::menu_item_action_with_options(self, label.into(), action.into(), options)
        }

        fn menu_item_command(&mut self, command: impl Into<CommandId>) -> ResponseExt {
            self.menu_item_command_with_options(command, MenuItemOptions::default())
        }

        fn menu_item_command_with_options(
            &mut self,
            command: impl Into<CommandId>,
            options: MenuItemOptions,
        ) -> ResponseExt {
            menu_items::menu_item_command_with_options(self, command.into(), options)
        }
    };
}

pub(crate) use menu_item_surface_methods;
