use std::cell::RefCell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{GlobalElementId, UiHost};

use super::policy::{ImUiMenuNavState, ImUiPopupMenuPolicyState};
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::imui::{ImUiFacade, PopupMenuOptions, UiWriterImUiFacadeExt};

mod content;
mod layout;
mod state;

pub(super) struct PopupMenuBuilt {
    pub(super) children: Vec<AnyElement>,
    pub(super) first_item: Option<GlobalElementId>,
    pub(super) content_focus: Option<GlobalElementId>,
}

pub(super) fn build_popup_menu<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    root_name: &str,
    options: PopupMenuOptions,
    popup_policy: ImUiPopupMenuPolicyState,
    menubar_policy: Option<ImUiMenubarPolicyState>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> Option<PopupMenuBuilt> {
    ui.with_cx_mut(|cx| {
        let Some(panel_state) =
            state::prepare_popup_menu_panel_state(cx, id, options.estimated_size)
        else {
            return None;
        };
        let layout = layout::popup_menu_panel_layout(
            cx,
            panel_state.anchor,
            panel_state.desired,
            options.placement,
        );
        let palette = layout::popup_menu_panel_palette(fret_ui::Theme::global(&*cx.app));

        let nav_items = Rc::new(RefCell::new(Vec::<GlobalElementId>::new()));
        let nav_items_for_state = nav_items.clone();
        let mut menu_id_for_focus: Option<GlobalElementId> = None;
        let mut build = Some(f);
        let popup_policy_for_panel = popup_policy.clone();
        let menubar_policy_for_panel = menubar_policy.clone();
        let panel = cx.with_root_name(root_name, |cx| {
            cx.named("fret-ui-kit.imui.popup.panel", |cx| {
                let semantics = layout::popup_menu_panel_semantics(id, layout.rect.origin);
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
                        popup_policy_for_panel.clone(),
                        menubar_policy_for_panel.clone(),
                        &mut build,
                    )
                });
                menu_id_for_focus = Some(menu.id);
                state::store_popup_menu_panel_id(cx, id, menu.id);
                menu
            })
        });

        let first_item = nav_items.borrow().first().copied();
        Some(PopupMenuBuilt {
            children: vec![panel],
            first_item,
            content_focus: menu_id_for_focus,
        })
    })
}
