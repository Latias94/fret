use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_ui::action::{DismissReason, OnCloseAutoFocus, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow,
    PositionStyle, SpacingLength,
};
use fret_ui::{GlobalElementId, UiHost};

use super::super::{ImUiFacade, PopupMenuOptions, UiWriterImUiFacadeExt};
use crate::primitives::menu::root as menu_root;
use crate::primitives::menu::sub as menu_sub;
use crate::primitives::popper;
use crate::{OverlayController, OverlayPresence};

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
        let open = super::super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
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
    menubar_policy: Option<super::super::menu_family_controls::ImUiMenubarPolicyState>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> Option<PopupMenuBuilt> {
    ui.with_cx_mut(|cx| {
        let (open, anchor_model, panel_id) =
            super::super::with_popup_store_for_id(cx, id, |st, _app| {
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
            super::super::with_popup_store_for_id(cx, id, |st, _app| {
                st.panel_id = None;
                st.keep_alive_generation = None;
            });
            cx.app.request_redraw(cx.window);
            return None;
        };

        let keep_alive_generation = super::super::popup_render_generation_for_window(cx);
        super::super::with_popup_store_for_id(cx, id, move |st, _app| {
            st.keep_alive_generation = Some(keep_alive_generation);
        });

        let desired = panel_id
            .and_then(|id| cx.last_bounds_for_element(id).map(|r| r.size))
            .unwrap_or(options.estimated_size);
        let layout = popper::popper_content_layout_sized(
            cx.environment_viewport_bounds(fret_ui::Invalidation::Layout),
            anchor,
            desired,
            options.placement,
        );

        let (popover, border) = {
            let theme = fret_ui::Theme::global(&*cx.app);
            (theme.color_token("popover"), theme.color_token("border"))
        };

        let nav_items = Rc::new(RefCell::new(Vec::<GlobalElementId>::new()));
        let nav_items_for_state = nav_items.clone();
        let mut menu_id_for_focus: Option<GlobalElementId> = None;
        let mut build = Some(f);
        let popup_policy_for_panel = popup_policy.clone();
        let menubar_policy_for_panel = menubar_policy.clone();
        let panel = cx.with_root_name(root_name, |cx| {
            cx.named("fret-ui-kit.imui.popup.panel", |cx| {
                let mut semantics = fret_ui::element::SemanticsProps::default();
                semantics.role = SemanticsRole::Menu;
                semantics.test_id = Some(Arc::from(format!("imui-popup-{id}")));
                semantics.layout = LayoutStyle {
                    position: PositionStyle::Absolute,
                    inset: InsetStyle {
                        left: Some(layout.rect.origin.x).into(),
                        top: Some(layout.rect.origin.y).into(),
                        ..Default::default()
                    },
                    overflow: Overflow::Visible,
                    ..Default::default()
                };

                let menu = cx.semantics_with_id(semantics, move |cx, menu_id| {
                    cx.state_for(
                        menu_id,
                        || ImUiMenuNavState {
                            items: nav_items_for_state.clone(),
                        },
                        |st| st.items.borrow_mut().clear(),
                    );

                    let mut panel_props = ContainerProps::default();
                    panel_props.background = Some(popover);
                    panel_props.border = Edges::all(Px(1.0));
                    panel_props.border_color = Some(border);
                    panel_props.corner_radii =
                        Corners::all(super::super::control_chrome::PANEL_RADIUS);
                    panel_props.padding = Edges::all(Px(4.0)).into();

                    vec![cx.container(panel_props, move |cx| {
                        let mut col = ColumnProps::default();
                        col.gap = SpacingLength::Px(Px(2.0));
                        col.layout.size.width = Length::Auto;
                        col.layout.size.height = Length::Auto;
                        vec![cx.column(col, move |cx| {
                            let render_children = move |cx: &mut fret_ui::ElementContext<'_, H>| {
                                let mut out: Vec<AnyElement> = Vec::new();
                                let mut ui = ImUiFacade {
                                    cx,
                                    out: &mut out,
                                    build_focus: None,
                                };
                                if let Some(f) = build.take() {
                                    f(&mut ui);
                                }
                                out
                            };
                            if let Some(menubar_policy) = menubar_policy_for_panel.clone() {
                                cx.provide(menubar_policy, move |cx| {
                                    cx.provide(popup_policy_for_panel.clone(), render_children)
                                })
                            } else {
                                cx.provide(popup_policy_for_panel.clone(), render_children)
                            }
                        })]
                    })]
                });
                menu_id_for_focus = Some(menu.id);
                super::super::with_popup_store_for_id(cx, id, |st, _app| {
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

pub(super) fn begin_popup_menu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupMenuOptions,
    preserve_focus_outside_while_submenu_open: bool,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    let overlay_id = ui.with_cx_mut(|cx| {
        let overlay_key = format!("fret-ui-kit.imui.popup.overlay.{id}");
        cx.named(overlay_key.as_str(), |cx| cx.root_id())
    });
    let root_name = OverlayController::popover_root_name(overlay_id);
    let popup_policy = popup_menu_policy_state_for_root(ui, id, root_name.as_str());
    let menubar_policy = ui.with_cx_mut(|cx| {
        cx.provided::<super::super::menu_family_controls::ImUiMenubarPolicyState>()
            .cloned()
    });
    let Some(built) = build_popup_menu(
        ui,
        id,
        root_name.as_str(),
        options,
        popup_policy.clone(),
        menubar_policy.clone(),
        f,
    ) else {
        return false;
    };

    ui.with_cx_mut(|cx| {
        let open = super::super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
        let trigger_id = trigger.unwrap_or(overlay_id);
        let initial_focus = if options.auto_focus {
            menu_root::MenuInitialFocusTargets::new()
                .keyboard_entry_focus(built.first_item)
                .pointer_content_focus(built.content_focus)
        } else {
            menu_root::MenuInitialFocusTargets::new()
        };
        let on_dismiss_request = if preserve_focus_outside_while_submenu_open {
            let submenu_models = popup_policy.submenu_models.clone();
            let open_for_dismiss = open.clone();
            Some(Arc::new(
                move |host: &mut dyn fret_ui::action::UiActionHost,
                      _acx,
                      req: &mut fret_ui::action::DismissRequestCx| {
                    if matches!(req.reason, DismissReason::FocusOutside) {
                        let submenu_open = host
                            .models_mut()
                            .read(&submenu_models.open_value, |value| value.clone())
                            .ok()
                            .flatten();
                        if submenu_open.is_some() {
                            req.prevent_default();
                            return;
                        }
                    }
                    let _ = host
                        .models_mut()
                        .update(&open_for_dismiss, |value| *value = false);
                },
            ) as OnDismissRequest)
        } else {
            None
        };
        let on_close_auto_focus = menubar_policy.as_ref().map(|policy| {
            let suppress_close_auto_focus = policy.suppress_close_auto_focus_once.clone();
            Arc::new(
                move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                      _acx,
                      req: &mut fret_ui::action::AutoFocusRequestCx| {
                    let suppress = host
                        .models_mut()
                        .read(&suppress_close_auto_focus, |value| *value)
                        .ok()
                        .unwrap_or(false);
                    if !suppress {
                        return;
                    }
                    let _ = host
                        .models_mut()
                        .update(&suppress_close_auto_focus, |value| *value = false);
                    req.prevent_default();
                },
            ) as OnCloseAutoFocus
        });
        let req = menu_root::dismissible_menu_request_with_modal_and_dismiss_handler(
            cx,
            overlay_id,
            trigger_id,
            open,
            OverlayPresence::instant(true),
            built.children,
            root_name.clone(),
            initial_focus,
            None,
            on_close_auto_focus,
            on_dismiss_request,
            Some(menu_root::submenu_pointer_move_handler(
                popup_policy.submenu_models.clone(),
                popup_policy.submenu_cfg,
            )),
            options.modal,
        );
        OverlayController::request(cx, req);
    });

    true
}
