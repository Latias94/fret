use super::*;

impl<'cx, 'a, H: UiHost> ImUiFacade<'cx, 'a, H> {
    pub fn menu_bar(&mut self, f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>)) {
        let build_focus = self.build_focus.clone();
        container_methods::menu_bar(self, build_focus, f);
    }

    pub fn menu_bar_with_options(
        &mut self,
        options: MenuBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
    ) {
        let build_focus = self.build_focus.clone();
        container_methods::menu_bar_with_options(self, build_focus, options, f);
    }

    pub fn tab_bar(
        &mut self,
        id: &str,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        let build_focus = self.build_focus.clone();
        container_methods::tab_bar(self, build_focus, id, f)
    }

    pub fn tab_bar_with_options(
        &mut self,
        id: &str,
        options: TabBarOptions,
        f: impl for<'cx2, 'a2> FnOnce(&mut ImUiTabBar<'cx2, 'a2, H>),
    ) -> TabBarResponse {
        let build_focus = self.build_focus.clone();
        container_methods::tab_bar_with_options(self, build_focus, id, options, f)
    }
}
