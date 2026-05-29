macro_rules! menu_selection_surface_methods {
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

        fn selectable(&mut self, label: impl Into<Arc<str>>, selected: bool) -> ResponseExt {
            self.selectable_with_options(
                label,
                SelectableOptions {
                    selected,
                    ..Default::default()
                },
            )
        }

        fn selectable_with_options(
            &mut self,
            label: impl Into<Arc<str>>,
            options: SelectableOptions,
        ) -> ResponseExt {
            selectable_controls::selectable_with_options(self, label.into(), options)
        }

        fn multi_selectable<K: Clone + PartialEq + 'static>(
            &mut self,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<ImUiMultiSelectState<K>>,
            all_keys: &[K],
            key: K,
        ) -> ResponseExt {
            self.multi_selectable_with_options(
                label,
                model,
                all_keys,
                key,
                SelectableOptions::default(),
            )
        }

        fn multi_selectable_with_options<K: Clone + PartialEq + 'static>(
            &mut self,
            label: impl Into<Arc<str>>,
            model: &fret_runtime::Model<ImUiMultiSelectState<K>>,
            all_keys: &[K],
            key: K,
            options: SelectableOptions,
        ) -> ResponseExt {
            multi_select::multi_selectable_with_options(
                self,
                label.into(),
                model,
                all_keys,
                key,
                options,
            )
        }

        fn combo(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            preview: impl Into<Arc<str>>,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> ComboResponse {
            self.combo_with_options(id, label, preview, ComboOptions::default(), f)
        }

        fn combo_with_options(
            &mut self,
            id: &str,
            label: impl Into<Arc<str>>,
            preview: impl Into<Arc<str>>,
            options: ComboOptions,
            f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
        ) -> ComboResponse {
            combo_controls::combo_with_options(self, id, label.into(), preview.into(), options, f)
        }

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

pub(super) use menu_selection_surface_methods;
