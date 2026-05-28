use super::*;

mod command;
mod item_methods;

pub(super) use command::menu_item_command_with_options;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
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
