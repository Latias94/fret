use std::cell::RefCell;
use std::rc::Rc;

use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::PopupMenuBuilt;
use super::content;
use super::layout::{self, PopupMenuPanelPalette};
use super::state::store_popup_menu_panel_id;
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::imui::popup_overlay::menu::policy::{ImUiMenuNavState, ImUiPopupMenuPolicyState};

pub(super) fn assemble_popup_menu_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: &str,
    root_name: &str,
    origin: fret_core::Point,
    palette: PopupMenuPanelPalette,
    popup_policy: ImUiPopupMenuPolicyState,
    menubar_policy: Option<ImUiMenubarPolicyState>,
    build: &mut Option<impl for<'cx2, 'a2> FnOnce(&mut crate::imui::ImUiFacade<'cx2, 'a2, H>)>,
) -> PopupMenuBuilt {
    let nav_items = Rc::new(RefCell::new(Vec::<GlobalElementId>::new()));
    let nav_items_for_state = nav_items.clone();
    let mut menu_id_for_focus: Option<GlobalElementId> = None;

    let panel = cx.with_root_name(root_name, |cx| {
        cx.named("fret-ui-kit.imui.popup.panel", |cx| {
            let semantics = layout::popup_menu_panel_semantics(id, origin);
            let menu = cx.semantics_with_id(semantics, move |cx, menu_id| {
                cx.state_for(
                    menu_id,
                    || ImUiMenuNavState {
                        items: nav_items_for_state.clone(),
                    },
                    |st| st.items.borrow_mut().clear(),
                );

                content::popup_menu_panel_children(
                    cx,
                    palette,
                    popup_policy.clone(),
                    menubar_policy.clone(),
                    build,
                )
            });
            menu_id_for_focus = Some(menu.id);
            store_popup_menu_panel_id(cx, id, menu.id);
            menu
        })
    });

    let first_item = nav_items.borrow().first().copied();
    PopupMenuBuilt {
        children: vec![panel],
        first_item,
        content_focus: menu_id_for_focus,
    }
}
