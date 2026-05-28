use super::*;

mod command;

pub(super) use command::menu_item_command_with_options;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn menu_item(&mut self, label: impl Into<Arc<str>>) -> ResponseExt {
        self.menu_item_with_options(label, MenuItemOptions::default())
    }

    pub fn menu_item_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_with_options(self, label, options);
        self.record_focusable(resp.id(), enabled);
        resp
    }

    pub fn menu_item_checkbox_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        checked: bool,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_checkbox_with_options(
            self, label, checked, options,
        );
        self.record_focusable(resp.id(), enabled);
        resp
    }

    pub fn menu_item_radio_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        checked: bool,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let enabled = options.enabled && self.with_cx_mut(|cx| !imui_is_disabled(cx));
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_radio_with_options(
            self, label, checked, options,
        );
        self.record_focusable(resp.id(), enabled);
        resp
    }

    pub fn menu_item_action(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
    ) -> ResponseExt {
        self.menu_item_action_with_options(label, action, MenuItemOptions::default())
    }

    pub fn menu_item_action_with_options(
        &mut self,
        label: impl Into<Arc<str>>,
        action: impl Into<ActionId>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_action_with_options(
            self, label, action, options,
        );
        self.record_focusable(resp.id(), resp.enabled());
        resp
    }

    pub fn begin_menu(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.begin_menu_with_options(id, label, BeginMenuOptions::default(), f)
    }

    pub fn begin_menu_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: BeginMenuOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        menu_family_controls::begin_menu_with_options(self, id, label.into(), options, f)
    }

    pub fn begin_submenu(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        self.begin_submenu_with_options(id, label, BeginSubmenuOptions::default(), f)
    }

    pub fn begin_submenu_with_options(
        &mut self,
        id: &str,
        label: impl Into<Arc<str>>,
        options: BeginSubmenuOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) -> DisclosureResponse {
        menu_family_controls::begin_submenu_with_options(self, id, label.into(), options, f)
    }

    pub fn menu_item_command(&mut self, command: impl Into<CommandId>) -> ResponseExt {
        self.menu_item_command_with_options(command, MenuItemOptions::default())
    }

    pub fn menu_item_command_with_options(
        &mut self,
        command: impl Into<CommandId>,
        options: MenuItemOptions,
    ) -> ResponseExt {
        let resp = <Self as UiWriterImUiFacadeExt<H>>::menu_item_command_with_options(
            self, command, options,
        );
        self.record_focusable(resp.id(), resp.enabled());
        resp
    }
}
