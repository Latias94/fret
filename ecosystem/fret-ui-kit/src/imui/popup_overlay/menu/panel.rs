use std::cell::RefCell;
use std::rc::Rc;

use fret_ui::element::AnyElement;
use fret_ui::{GlobalElementId, UiHost};

use super::policy::{ImUiMenuNavState, ImUiPopupMenuPolicyState};
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::imui::{ImUiFacade, PopupMenuOptions, UiWriterImUiFacadeExt};

mod content;
mod layout;

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
        let (open, anchor_model, panel_id) =
            super::super::super::with_popup_store_for_id(cx, id, |st, _app| {
                (st.open.clone(), st.anchor.clone(), st.panel_id)
            });
        let is_open = cx
            .read_model(&open, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);
        if !is_open {
            return None;
        }

        let anchor = cx
            .read_model(&anchor_model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(None);
        let Some(anchor) = anchor else {
            let _ = cx.app.models_mut().update(&open, |v| *v = false);
            let _ = cx.app.models_mut().update(&anchor_model, |v| *v = None);
            super::super::super::with_popup_store_for_id(cx, id, |st, _app| {
                st.panel_id = None;
                st.keep_alive_generation = None;
            });
            cx.app.request_redraw(cx.window);
            return None;
        };

        let keep_alive_generation = super::super::super::popup_render_generation_for_window(cx);
        super::super::super::with_popup_store_for_id(cx, id, move |st, _app| {
            st.keep_alive_generation = Some(keep_alive_generation);
        });

        let desired = panel_id
            .and_then(|id| cx.last_bounds_for_element(id).map(|r| r.size))
            .unwrap_or(options.estimated_size);
        let layout = layout::popup_menu_panel_layout(cx, anchor, desired, options.placement);
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
                super::super::super::with_popup_store_for_id(cx, id, |st, _app| {
                    st.panel_id = Some(menu.id)
                });
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
