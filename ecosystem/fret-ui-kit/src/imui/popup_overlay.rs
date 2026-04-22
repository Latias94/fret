use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, Rect, SemanticsRole, Size};
use fret_ui::action::{DismissReason, DismissRequestCx, OnDismissRequest};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow,
    PositionStyle, SpacingLength,
};
use fret_ui::{GlobalElementId, UiHost};

use super::{ImUiFacade, PopupMenuOptions, PopupModalOptions, ResponseExt, UiWriterImUiFacadeExt};
use crate::primitives::dialog;
use crate::primitives::menu::root as menu_root;
use crate::primitives::menu::sub as menu_sub;
use crate::primitives::popper;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

#[derive(Debug, Clone)]
pub(in crate::imui) struct ImUiMenuNavState {
    pub(super) items: Rc<RefCell<Vec<GlobalElementId>>>,
}

#[derive(Debug, Clone)]
pub(in crate::imui) struct ImUiPopupMenuPolicyState {
    pub(super) submenu_models: menu_sub::MenuSubmenuModels,
    pub(super) submenu_cfg: menu_sub::MenuSubmenuConfig,
}

pub(super) fn popup_open_model<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) -> fret_runtime::Model<bool> {
    ui.with_cx_mut(|cx| super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone()))
}

pub(super) fn popup_menu_policy_state_for_root<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    root_name: &str,
) -> ImUiPopupMenuPolicyState {
    ui.with_cx_mut(|cx| {
        let open = super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
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

pub(super) fn drop_popup_scope<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) {
    ui.with_cx_mut(|cx| super::drop_popup_scope_for_id(cx, id));
}

pub(super) fn open_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(ui: &mut W, id: &str) {
    ui.with_cx_mut(|cx| {
        let keep_alive_generation = super::popup_render_generation_for_window(cx);
        let open = super::with_popup_store_for_id(cx, id, move |st, _app| {
            st.keep_alive_generation = Some(keep_alive_generation);
            st.open.clone()
        });
        let _ = cx.app.models_mut().update(&open, |v| *v = true);
        cx.app.request_redraw(cx.window);
    });
}

pub(super) fn open_popup_at<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    anchor: Rect,
) {
    ui.with_cx_mut(|cx| {
        let keep_alive_generation = super::popup_render_generation_for_window(cx);
        let (open, anchor_model) = super::with_popup_store_for_id(cx, id, move |st, _app| {
            st.keep_alive_generation = Some(keep_alive_generation);
            (st.open.clone(), st.anchor.clone())
        });
        let _ = cx
            .app
            .models_mut()
            .update(&anchor_model, |v| *v = Some(anchor));
        let _ = cx.app.models_mut().update(&open, |v| *v = true);
        cx.app.request_redraw(cx.window);
    });
}

pub(super) fn close_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(ui: &mut W, id: &str) {
    ui.with_cx_mut(|cx| {
        let open = super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
        let _ = cx.app.models_mut().update(&open, |v| *v = false);
        cx.app.request_redraw(cx.window);
    });
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
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> Option<PopupMenuBuilt> {
    ui.with_cx_mut(|cx| {
        let (open, anchor_model, panel_id) = super::with_popup_store_for_id(cx, id, |st, _app| {
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
            super::with_popup_store_for_id(cx, id, |st, _app| {
                st.panel_id = None;
                st.keep_alive_generation = None;
            });
            cx.app.request_redraw(cx.window);
            return None;
        };

        let keep_alive_generation = super::popup_render_generation_for_window(cx);
        super::with_popup_store_for_id(cx, id, move |st, _app| {
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
                    panel_props.corner_radii = Corners::all(super::control_chrome::PANEL_RADIUS);
                    panel_props.padding = Edges::all(Px(4.0)).into();

                    vec![cx.container(panel_props, move |cx| {
                        let mut col = ColumnProps::default();
                        col.gap = SpacingLength::Px(Px(2.0));
                        col.layout.size.width = Length::Auto;
                        col.layout.size.height = Length::Auto;
                        vec![cx.column(col, move |cx| {
                            cx.provide(popup_policy_for_panel.clone(), move |cx| {
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
                            })
                        })]
                    })]
                });
                menu_id_for_focus = Some(menu.id);
                super::with_popup_store_for_id(cx, id, |st, _app| st.panel_id = Some(menu.id));
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
    let Some(built) =
        build_popup_menu(ui, id, root_name.as_str(), options, popup_policy.clone(), f)
    else {
        return false;
    };

    ui.with_cx_mut(|cx| {
        let open = super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
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
                    let _ = host.models_mut().update(&open_for_dismiss, |value| *value = false);
                },
            ) as OnDismissRequest)
        } else {
            None
        };
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
            None,
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

pub(super) fn begin_popup_modal_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupModalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    ui.with_cx_mut(|cx| {
        let open = super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
        let is_open = cx
            .read_model(&open, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);
        if !is_open {
            return false;
        }

        let keep_alive_generation = super::popup_render_generation_for_window(cx);
        super::with_popup_store_for_id(cx, id, move |st, _app| {
            st.keep_alive_generation = Some(keep_alive_generation);
        });

        let overlay_key = format!("fret-ui-kit.imui.popup_modal.overlay.{id}");
        let overlay_id = cx.named(overlay_key.as_str(), |cx| cx.root_id());

        let root_name = OverlayController::modal_root_name(overlay_id);

        let (popover, border) = {
            let theme = fret_ui::Theme::global(&*cx.app);
            (theme.color_token("popover"), theme.color_token("border"))
        };

        let dim = Color {
            a: 0.4,
            ..Color::from_srgb_hex_rgb(0x00_00_00)
        };

        let size = options.size;
        let left =
            Px(cx.bounds.origin.x.0 + (cx.bounds.size.width.0 - size.width.0).max(0.0) * 0.5);
        let top =
            Px(cx.bounds.origin.y.0 + (cx.bounds.size.height.0 - size.height.0).max(0.0) * 0.5);

        let close_on_outside_press = options.close_on_outside_press;
        let open_for_dismiss = open.clone();
        let on_dismiss_request: OnDismissRequest = Arc::new(
            move |host, acx, req: &mut DismissRequestCx| match req.reason {
                DismissReason::Escape => {
                    let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
                    host.notify(acx);
                }
                DismissReason::OutsidePress { .. } if close_on_outside_press => {
                    let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
                    host.notify(acx);
                }
                _ => {
                    req.prevent_default();
                }
            },
        );

        let focus_state = Rc::new(Cell::new(None::<GlobalElementId>));
        let focus_state_for_build = focus_state.clone();
        let mut panel_id_for_focus: Option<GlobalElementId> = None;
        let mut build = Some(f);

        let layer = cx.with_root_name(root_name.as_str(), |cx| {
            cx.named("fret-ui-kit.imui.popup_modal.layer", |cx| {
                let mut stack = fret_ui::element::StackProps::default();
                stack.layout.position = PositionStyle::Absolute;
                stack.layout.inset = InsetStyle {
                    left: Some(Px(0.0)).into(),
                    right: Some(Px(0.0)).into(),
                    top: Some(Px(0.0)).into(),
                    bottom: Some(Px(0.0)).into(),
                };
                stack.layout.size.width = Length::Fill;
                stack.layout.size.height = Length::Fill;
                stack.layout.overflow = Overflow::Visible;

                cx.stack_props(stack, |cx| {
                    let backdrop_visual = cx.container(
                        {
                            let mut props = ContainerProps::default();
                            props.layout.position = PositionStyle::Absolute;
                            props.layout.inset = InsetStyle {
                                left: Some(Px(0.0)).into(),
                                right: Some(Px(0.0)).into(),
                                top: Some(Px(0.0)).into(),
                                bottom: Some(Px(0.0)).into(),
                            };
                            props.layout.size.width = Length::Fill;
                            props.layout.size.height = Length::Fill;
                            props.background = Some(dim);
                            props
                        },
                        |_cx| Vec::<AnyElement>::new(),
                    );
                    let backdrop = dialog::modal_barrier_with_dismiss_handler(
                        cx,
                        open.clone(),
                        close_on_outside_press,
                        Some(on_dismiss_request.clone()),
                        [backdrop_visual],
                    );

                    let panel = cx.named("fret-ui-kit.imui.popup_modal.panel", |cx| {
                        let mut semantics = fret_ui::element::SemanticsProps::default();
                        semantics.role = SemanticsRole::Dialog;
                        semantics.test_id = Some(Arc::from(format!("imui-popup-modal-{id}")));
                        semantics.layout = LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                left: Some(left).into(),
                                top: Some(top).into(),
                                ..Default::default()
                            },
                            size: fret_ui::element::SizeStyle {
                                width: Length::Px(size.width),
                                height: Length::Px(size.height),
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        let modal = cx.semantics_with_id(semantics, move |cx, _id| {
                            let mut panel_props = ContainerProps::default();
                            panel_props.background = Some(popover);
                            panel_props.border = Edges::all(Px(1.0));
                            panel_props.border_color = Some(border);
                            panel_props.corner_radii =
                                Corners::all(super::control_chrome::PANEL_RADIUS);
                            panel_props.padding = Edges::all(Px(8.0)).into();
                            panel_props.layout.size.width = Length::Fill;
                            panel_props.layout.size.height = Length::Fill;

                            vec![cx.container(panel_props, move |cx| {
                                let mut out: Vec<AnyElement> = Vec::new();
                                {
                                    let mut ui = ImUiFacade {
                                        cx,
                                        out: &mut out,
                                        build_focus: Some(focus_state_for_build.clone()),
                                    };
                                    if let Some(f) = build.take() {
                                        f(&mut ui);
                                    }
                                }
                                out
                            })]
                        });
                        panel_id_for_focus = Some(modal.id);
                        modal
                    });

                    vec![backdrop, panel]
                })
            })
        });

        let mut req = OverlayRequest::modal(
            overlay_id,
            trigger,
            open.clone(),
            OverlayPresence::instant(true),
            vec![layer],
        );
        req.root_name = Some(root_name);
        req.dismissible_on_dismiss_request = Some(on_dismiss_request);
        req.initial_focus = focus_state.get().or(panel_id_for_focus);
        OverlayController::request(cx, req);

        true
    })
}

pub(super) fn begin_popup_context_menu_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    options: PopupMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    if trigger.context_menu_requested() {
        let anchor = trigger
            .context_menu_anchor()
            .map(|p| Rect::new(p, Size::new(Px(1.0), Px(1.0))))
            .or(trigger.core.rect);
        if let Some(anchor) = anchor {
            open_popup_at(ui, id, anchor);
        }
    }

    begin_popup_menu_with_options(ui, id, trigger.id, options, false, f)
}
