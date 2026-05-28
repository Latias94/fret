use super::*;

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
}
