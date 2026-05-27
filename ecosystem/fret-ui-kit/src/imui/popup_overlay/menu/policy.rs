use std::cell::RefCell;
use std::rc::Rc;

use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;
use crate::primitives::menu::root as menu_root;
use crate::primitives::menu::sub as menu_sub;

#[derive(Debug, Clone)]
pub(in crate::imui) struct ImUiMenuNavState {
    pub(in crate::imui) items: Rc<RefCell<Vec<GlobalElementId>>>,
}

#[derive(Debug, Clone)]
pub(in crate::imui) struct ImUiPopupMenuPolicyState {
    pub(in crate::imui) submenu_models: menu_sub::MenuSubmenuModels,
    pub(in crate::imui) submenu_cfg: menu_sub::MenuSubmenuConfig,
}

pub(super) fn popup_menu_policy_state_for_root<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    root_name: &str,
) -> ImUiPopupMenuPolicyState {
    ui.with_cx_mut(|cx| {
        let open = super::super::super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
        let is_open = cx
            .read_model(&open, fret_ui::Invalidation::Paint, |_app, value| *value)
            .unwrap_or(false);
        let submenu_cfg = menu_sub::MenuSubmenuConfig::default();
        let submenu_models = cx.with_root_name(root_name, |cx| {
            let timer_handler = cx.named("fret-ui-kit.imui.popup.menu-policy", |cx| cx.root_id());
            menu_root::sync_root_open_and_ensure_submenu(cx, is_open, timer_handler, submenu_cfg)
        });
        ImUiPopupMenuPolicyState {
            submenu_models,
            submenu_cfg,
        }
    })
}
