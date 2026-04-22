//! Immediate-mode menu-bar helpers.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{Corners, Edges, KeyCode, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_runtime::Model;
use fret_ui::action::{ActivateReason, PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{
    AnyElement, ContainerProps, Length, PressableA11y, PressableProps, TextProps,
};
use fret_ui::{ElementContext, GlobalElementId, Theme, UiHost};

use super::{
    BeginMenuOptions, BeginSubmenuOptions, DisclosureResponse, ImUiFacade, MenuBarOptions,
    MenuItemOptions, ResponseExt, UiWriterImUiFacadeExt,
};
use crate::primitives::menu::sub_trigger;
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

#[derive(Debug, Clone)]
pub(in crate::imui) struct ImUiMenubarPolicyState {
    pub(super) open_menu: Model<Option<Arc<str>>>,
    pub(super) group_active: Model<Option<menubar_trigger_row::MenubarActiveTrigger>>,
    pub(super) registry: Model<Vec<menubar_trigger_row::MenubarTriggerRowEntry>>,
    pub(super) suppress_close_auto_focus_once: Model<bool>,
}

pub(super) fn menu_bar_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build_focus: Option<Rc<Cell<Option<GlobalElementId>>>>,
    options: MenuBarOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> AnyElement {
    let gap = options.gap;
    let test_id = options.test_id;
    cx.named("fret-ui-kit.imui.menu-bar", move |cx| {
        let group = cx.root_id();
        let open_menu = cx.local_model_keyed("open_menu", || None::<Arc<str>>);
        let group_active = menubar_trigger_row::ensure_group_active_model(cx, group);
        let registry = menubar_trigger_row::ensure_group_registry_model(cx, group);
        let suppress_close_auto_focus_once =
            cx.local_model_keyed("suppress_close_auto_focus_once", || false);
        let policy = ImUiMenubarPolicyState {
            open_menu,
            group_active,
            registry,
            suppress_close_auto_focus_once,
        };

        let mut builder = crate::ui::h_flex_build(move |cx: &mut ElementContext<'_, H>, out| {
            let _ = cx.app.models_mut().update(
                &policy.registry,
                |entries: &mut Vec<menubar_trigger_row::MenubarTriggerRowEntry>| entries.clear(),
            );
            cx.provide(policy.clone(), move |cx| {
                super::containers::build_imui_children_with_focus(cx, out, build_focus, f);
            });
        });
        builder = builder
            .gap_metric(gap)
            .justify(crate::Justify::Start)
            .items(crate::Items::Center)
            .no_wrap()
            .role(SemanticsRole::MenuBar);
        if let Some(test_id) = test_id {
            builder = builder.test_id(test_id);
        }
        builder.into_element(cx)
    })
}

pub(super) fn begin_menu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: BeginMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::imui_is_disabled(cx));
    let menubar_policy = ui.with_cx_mut(|cx| cx.provided::<ImUiMenubarPolicyState>().cloned());
    let popup_open = ui.popup_open_model(id);
    let row_open = if menubar_policy.is_some() {
        ui.with_cx_mut(|cx| cx.local_model_keyed(format!("menubar_row_open.{id}"), || false))
    } else {
        popup_open.clone()
    };
    let was_open_model =
        ui.with_cx_mut(|cx| cx.local_model_keyed(format!("was_open.{id}"), || false));
    let was_popup_open_model =
        ui.with_cx_mut(|cx| cx.local_model_keyed(format!("was_popup_open.{id}"), || false));
    let open_before = ui.with_cx_mut(|cx| {
        cx.read_model(&row_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    let popup_open_before = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    let was_open_before_render = ui.with_cx_mut(|cx| {
        cx.read_model(
            &was_open_model,
            fret_ui::Invalidation::Paint,
            |_app, value| *value,
        )
        .unwrap_or(false)
    });
    let was_popup_open_before_render = ui.with_cx_mut(|cx| {
        cx.read_model(
            &was_popup_open_model,
            fret_ui::Invalidation::Paint,
            |_app, value| *value,
        )
        .unwrap_or(false)
    });

    let trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        menu_trigger_with_options(
            ui,
            Arc::from(id),
            label.clone(),
            open_before,
            row_open.clone(),
            menubar_policy.clone(),
            enabled,
            options.test_id.clone(),
            options.activate_shortcut,
            options.shortcut_repeat,
        )
    });

    let open_after_trigger = ui.with_cx_mut(|cx| {
        cx.read_model(&row_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    if let Some(policy) = menubar_policy.as_ref()
        && open_after_trigger
        && !trigger.clicked()
        && let Some(trigger_id) = trigger.id
    {
        let is_active_trigger = ui.with_cx_mut(|cx| {
            cx.read_model(
                &policy.group_active,
                fret_ui::Invalidation::Paint,
                |_app, value| {
                    value
                        .as_ref()
                        .is_some_and(|active| active.trigger == trigger_id)
                },
            )
            .unwrap_or(false)
        });
        if is_active_trigger {
            ui.with_cx_mut(|cx| {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&policy.open_menu, |value| *value = Some(Arc::from(id)));
            });
        }
    }

    let open_menu_before = menubar_policy.as_ref().and_then(|policy| {
        ui.with_cx_mut(|cx| {
            cx.read_model(
                &policy.open_menu,
                fret_ui::Invalidation::Paint,
                |_app, value| value.clone(),
            )
            .unwrap_or(None)
        })
    });
    if let Some(policy) = menubar_policy.as_ref()
        && open_after_trigger
        && !popup_open_before
        && was_popup_open_before_render
    {
        ui.with_cx_mut(|cx| {
            let _ = cx
                .app
                .models_mut()
                .update(&row_open, |value| *value = false);
            let _ = cx.app.models_mut().update(&policy.open_menu, |value| {
                if value.as_ref().is_some_and(|current| current.as_ref() == id) {
                    *value = None;
                }
            });
            if let Some(trigger_id) = trigger.id {
                let _ = cx.app.models_mut().update(&policy.group_active, |value| {
                    if value
                        .as_ref()
                        .is_some_and(|active| active.trigger == trigger_id)
                    {
                        *value = None;
                    }
                });
            }
        });
    }
    if let Some(policy) = menubar_policy.as_ref()
        && !open_before
        && was_open_before_render
    {
        if popup_open_before {
            ui.close_popup(id);
        }
        ui.with_cx_mut(|cx| {
            let _ = cx.app.models_mut().update(&policy.open_menu, |value| {
                if value.as_ref().is_some_and(|current| current.as_ref() == id) {
                    *value = None;
                }
            });
        });
    }

    if enabled && trigger.clicked() {
        if let Some(policy) = menubar_policy.as_ref() {
            ui.with_cx_mut(|cx| {
                let _ = cx.app.models_mut().update(&policy.open_menu, |value| {
                    if open_before && value.as_ref().is_some_and(|current| current.as_ref() == id) {
                        *value = None;
                    } else {
                        *value = Some(Arc::from(id));
                    }
                });
            });
        } else if open_before {
            ui.close_popup(id);
        }
    }

    let open_requested = if let Some(policy) = menubar_policy.as_ref() {
        let open_menu_now = ui.with_cx_mut(|cx| {
            cx.read_model(
                &policy.open_menu,
                fret_ui::Invalidation::Paint,
                |_app, value| value.clone(),
            )
            .unwrap_or(None)
        });
        let should_close = open_before
            && (open_menu_now
                .as_ref()
                .is_some_and(|current| current.as_ref() != id)
                || (open_menu_before
                    .as_ref()
                    .is_some_and(|current| current.as_ref() == id)
                    && open_menu_now.is_none()));
        if should_close {
            ui.with_cx_mut(|cx| {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&row_open, |value| *value = false);
            });
            ui.close_popup(id);
        }
        let requested_by_policy = open_menu_now
            .as_ref()
            .is_some_and(|current| current.as_ref() == id);
        requested_by_policy || (open_before && !should_close)
    } else {
        ui.with_cx_mut(|cx| {
            cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
                *value
            })
            .unwrap_or(false)
        })
    };

    if open_requested
        && let Some(menubar_policy) = menubar_policy.as_ref()
        && let Some(trigger_id) = trigger.id
    {
        ui.with_cx_mut(|cx| {
            let open_for_state = row_open.clone();
            let _ = cx
                .app
                .models_mut()
                .update(&menubar_policy.group_active, |value| {
                    *value = Some(menubar_trigger_row::MenubarActiveTrigger {
                        trigger: trigger_id,
                        open: open_for_state,
                    });
                });
            let _ = cx.app.models_mut().update(&row_open, |value| *value = true);
        });
    }
    if open_requested && let Some(anchor) = trigger.core.rect {
        ui.open_popup_at(id, anchor);
    }

    let popup_opened = super::popup_overlay::begin_popup_menu_with_options(
        ui,
        id,
        trigger.id,
        options.popup,
        menubar_policy.is_some(),
        f,
    );
    if !enabled && popup_opened {
        ui.with_cx_mut(|cx| {
            let _ = cx
                .app
                .models_mut()
                .update(&row_open, |value| *value = false);
        });
        ui.close_popup(id);
    }

    let open_after = ui.with_cx_mut(|cx| {
        cx.read_model(&row_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    let popup_open_after = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&was_open_model, |value| *value = open_after);
        let _ = cx
            .app
            .models_mut()
            .update(&was_popup_open_model, |value| *value = popup_open_after);
    });

    DisclosureResponse {
        trigger,
        open: popup_open_after,
        toggled: popup_open_before != popup_open_after,
    }
}

pub(super) fn begin_submenu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: BeginSubmenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::imui_is_disabled(cx));
    let popup_open = ui.popup_open_model(id);
    let popup_policy = ui.with_cx_mut(|cx| {
        cx.provided::<super::popup_overlay::ImUiPopupMenuPolicyState>()
            .cloned()
    });
    let was_open_model =
        ui.with_cx_mut(|cx| cx.local_model_keyed(format!("was_open.{id}"), || false));
    let open_before = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    let was_open_before_render = ui.with_cx_mut(|cx| {
        cx.read_model(
            &was_open_model,
            fret_ui::Invalidation::Paint,
            |_app, value| *value,
        )
        .unwrap_or(false)
    });
    let submenu_value = Arc::<str>::from(id);
    let popup_estimated_size = options.popup.estimated_size;

    let trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        super::menu_controls::menu_item_with_options_and_pressable_hook(
            ui,
            label.clone(),
            MenuItemOptions {
                enabled,
                test_id: options.test_id.clone(),
                submenu: true,
                expanded: Some(open_before),
                activate_shortcut: options.activate_shortcut,
                shortcut_repeat: options.shortcut_repeat,
                ..Default::default()
            },
            {
                let popup_policy = popup_policy.clone();
                let submenu_value = submenu_value.clone();
                move |cx, state, item_id, item_enabled| {
                    let Some(popup_policy) = popup_policy.as_ref() else {
                        return;
                    };
                    let geometry_hint = sub_trigger::MenuSubTriggerGeometryHint {
                        outer: cx.environment_viewport_bounds(fret_ui::Invalidation::Layout),
                        desired: popup_estimated_size,
                    };
                    let _ = sub_trigger::wire(
                        cx,
                        state,
                        item_id,
                        !item_enabled,
                        true,
                        submenu_value.clone(),
                        &popup_policy.submenu_models,
                        popup_policy.submenu_cfg,
                        Some(geometry_hint),
                    );
                }
            },
        )
    });

    if let Some(policy) = popup_policy.as_ref() {
        let open_submenu_before = ui.with_cx_mut(|cx| {
            cx.read_model(
                &policy.submenu_models.open_value,
                fret_ui::Invalidation::Paint,
                |_app, value| value.clone(),
            )
            .unwrap_or(None)
        });
        let is_selected_before = open_submenu_before
            .as_ref()
            .is_some_and(|value| value.as_ref() == id);
        if enabled && trigger.clicked() {
            if open_before && is_selected_before {
                clear_imui_submenu(
                    ui,
                    policy,
                    submenu_value.as_ref(),
                    trigger.id,
                    trigger.core.rect.is_none(),
                );
            } else if !is_selected_before {
                select_imui_submenu(ui, policy, submenu_value.clone(), trigger.id);
            }
        }

        if !open_before && was_open_before_render {
            clear_imui_submenu(ui, policy, submenu_value.as_ref(), trigger.id, true);
        }

        let open_submenu_now = ui.with_cx_mut(|cx| {
            cx.read_model(
                &policy.submenu_models.open_value,
                fret_ui::Invalidation::Paint,
                |_app, value| value.clone(),
            )
            .unwrap_or(None)
        });
        let should_open = open_submenu_now
            .as_ref()
            .is_some_and(|value| value.as_ref() == id);
        let should_close = open_before && !should_open;

        if should_close {
            ui.close_popup(id);
        } else if should_open && let Some(anchor) = trigger.core.rect {
            ui.open_popup_at(id, anchor);
        }
    }

    let popup_opened = super::popup_overlay::begin_popup_menu_with_options(
        ui,
        id,
        trigger.id,
        options.popup,
        false,
        f,
    );
    if !enabled && popup_opened {
        ui.close_popup(id);
    }

    let open_after = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&was_open_model, |value| *value = open_after);
    });

    DisclosureResponse {
        trigger,
        open: open_after,
        toggled: open_before != open_after,
    }
}

fn clear_imui_submenu<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    popup_policy: &super::popup_overlay::ImUiPopupMenuPolicyState,
    submenu_value: &str,
    trigger_id: Option<GlobalElementId>,
    clear_geometry: bool,
) {
    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.open_value, |value| {
                if value
                    .as_ref()
                    .is_some_and(|current| current.as_ref() == submenu_value)
                {
                    *value = None;
                }
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.trigger, |value| {
                if *value == trigger_id {
                    *value = None;
                }
            });
        if clear_geometry {
            let _ = cx
                .app
                .models_mut()
                .update(&popup_policy.submenu_models.geometry, |value| *value = None);
        }
        let _ =
            cx.app
                .models_mut()
                .update(&popup_policy.submenu_models.pending_open_value, |value| {
                    if value
                        .as_ref()
                        .is_some_and(|current| current.as_ref() == submenu_value)
                    {
                        *value = None;
                    }
                });
        let _ = cx.app.models_mut().update(
            &popup_policy.submenu_models.pending_open_trigger,
            |value| {
                if *value == trigger_id {
                    *value = None;
                }
            },
        );
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.pointer_grace_intent, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.pointer_grace_timer, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.close_timer, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.focus_target, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.focus_timer, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.focus_retry_attempts, |value| {
                *value = 0
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.open_timer, |value| {
                *value = None
            });
    });
}

fn select_imui_submenu<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    popup_policy: &super::popup_overlay::ImUiPopupMenuPolicyState,
    submenu_value: Arc<str>,
    trigger_id: Option<GlobalElementId>,
) {
    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.open_value, |value| {
                *value = Some(submenu_value.clone());
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.trigger, |value| {
                *value = trigger_id
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.pending_open_value, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.pending_open_trigger, |value| {
                *value = None
            });
        let _ = cx
            .app
            .models_mut()
            .update(&popup_policy.submenu_models.open_timer, |value| {
                *value = None
            });
    });
}

fn menu_trigger_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    logical_key: Arc<str>,
    label: Arc<str>,
    open: bool,
    open_model: Model<bool>,
    menubar_policy: Option<ImUiMenubarPolicyState>,
    enabled: bool,
    test_id: Option<Arc<str>>,
    activate_shortcut: Option<fret_runtime::KeyChord>,
    shortcut_repeat: bool,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::MenuItem),
            label: Some(label.clone()),
            test_id,
            expanded: Some(open),
            ..Default::default()
        };

        cx.pressable_with_id(props, move |cx, state, id| {
            let open_model = open_model.clone();
            let menubar_policy = menubar_policy.clone();
            let logical_key = logical_key.clone();
            cx.pressable_clear_on_pointer_down();
            cx.pressable_clear_on_pointer_up();
            cx.key_clear_on_key_down_for(id);

            let active_item_model = super::active_item_model_for_window(cx);
            let active_item_model_for_down = active_item_model.clone();
            let active_item_model_for_up = active_item_model.clone();
            let lifecycle_model = super::lifecycle_session_model_for(cx, id);
            let lifecycle_model_for_activate = lifecycle_model.clone();
            let lifecycle_model_for_down = lifecycle_model.clone();
            let lifecycle_model_for_up = lifecycle_model.clone();

            cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
                if reason == ActivateReason::Keyboard {
                    super::mark_lifecycle_instant_if_inactive(
                        host,
                        acx,
                        &lifecycle_model_for_activate,
                        false,
                    );
                }
                host.record_transient_event(acx, super::KEY_CLICKED);
                host.notify(acx);
            }));

            if enabled {
                cx.key_on_key_down_for(
                    id,
                    Arc::new(move |host, acx, down| {
                        if let Some(shortcut) = activate_shortcut {
                            let matches_shortcut =
                                down.key == shortcut.key && down.modifiers == shortcut.mods;
                            if matches_shortcut
                                && (!down.repeat || shortcut_repeat)
                                && !down.ime_composing
                            {
                                super::mark_lifecycle_instant_if_inactive(
                                    host,
                                    acx,
                                    &lifecycle_model,
                                    false,
                                );
                                host.record_transient_event(acx, super::KEY_CLICKED);
                                host.notify(acx);
                                return true;
                            }
                        }

                        false
                    }),
                );
            }

            if let Some(menubar_policy) = menubar_policy.as_ref() {
                let (patient_click_sticky, patient_click_timer) =
                    menubar_trigger_row::ensure_trigger_patient_click_models(cx, id);
                menubar_trigger_row::register_trigger_in_registry(
                    cx,
                    menubar_policy.registry.clone(),
                    logical_key.clone(),
                    id,
                    open_model.clone(),
                    enabled,
                    None,
                );
                menubar_trigger_row::sync_trigger_row_state(
                    cx,
                    menubar_policy.group_active.clone(),
                    id,
                    open_model.clone(),
                    patient_click_sticky.clone(),
                    patient_click_timer.clone(),
                    enabled,
                    state.hovered || state.hovered_raw || state.hovered_raw_below_barrier,
                    state.pressed,
                    state.focused,
                );
                cx.pressable_add_on_activate(menubar_trigger_row::toggle_on_activate(
                    menubar_policy.group_active.clone(),
                    id,
                    open_model.clone(),
                    patient_click_sticky,
                    patient_click_timer,
                ));
                let open_model_for_arrows = open_model.clone();
                cx.key_add_on_key_down_for(
                    id,
                    Arc::new(move |host, _acx, down| {
                        if down.repeat {
                            return false;
                        }
                        match down.key {
                            KeyCode::ArrowDown | KeyCode::ArrowUp => {
                                let _ = host
                                    .models_mut()
                                    .update(&open_model_for_arrows, |value| *value = true);
                                true
                            }
                            _ => false,
                        }
                    }),
                );
            }

            cx.pressable_on_pointer_down(Arc::new(move |host, acx, down| {
                super::mark_lifecycle_activated_on_left_pointer_down(
                    host,
                    acx,
                    down.button,
                    &lifecycle_model_for_down,
                );
                super::mark_active_item_on_left_pointer_down(
                    host,
                    acx,
                    down.button,
                    &active_item_model_for_down,
                    true,
                );
                PressablePointerDownResult::Continue
            }));

            cx.pressable_on_pointer_up(Arc::new(move |host, acx, up| {
                super::mark_lifecycle_deactivated_on_left_pointer_up(
                    host,
                    acx,
                    up.button,
                    &lifecycle_model_for_up,
                );
                super::clear_active_item_on_left_pointer_up(
                    host,
                    acx,
                    up.button,
                    &active_item_model_for_up,
                );
                PressablePointerUpResult::Continue
            }));

            response.core.hovered = state.hovered;
            response.core.pressed = state.pressed;
            response.core.focused = state.focused;
            response.nav_highlighted =
                state.focused && fret_ui::focus_visible::is_focus_visible(cx.app, Some(cx.window));
            response.id = Some(id);
            response.core.clicked = cx.take_transient_for(id, super::KEY_CLICKED);
            response.core.rect = cx.last_bounds_for_element(id);
            let hover_delay =
                super::install_hover_query_hooks_for_pressable(cx, id, state.hovered_raw, None);
            response.pointer_hovered_raw = state.hovered_raw;
            response.pointer_hovered_raw_below_barrier = state.hovered_raw_below_barrier;
            response.hover_stationary_met = hover_delay.stationary_met;
            response.hover_delay_short_met = hover_delay.delay_short_met;
            response.hover_delay_normal_met = hover_delay.delay_normal_met;
            response.hover_delay_short_shared_met = hover_delay.shared_delay_short_met;
            response.hover_delay_normal_shared_met = hover_delay.shared_delay_normal_met;
            response.hover_blocked_by_active_item =
                super::hover_blocked_by_active_item_for(cx, id, &active_item_model);
            super::populate_response_lifecycle_transients(cx, id, response);
            super::populate_response_lifecycle_from_active_state(
                cx,
                id,
                state.pressed,
                false,
                response,
            );
            super::sanitize_response_for_enabled(enabled, response);

            vec![menu_trigger_visual(cx, label.clone(), open, enabled, state)]
        })
    });

    ui.add(element);
    response
}

fn menu_trigger_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    open: bool,
    enabled: bool,
    state: fret_ui::element::PressableState,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let active = open || state.hovered || state.focused || state.pressed;
    let background = if active {
        Some(
            theme
                .color_by_key("accent")
                .unwrap_or_else(|| theme.color_token("accent")),
        )
    } else {
        None
    };
    let foreground = if !enabled {
        theme
            .color_by_key("muted-foreground")
            .unwrap_or_else(|| theme.color_token("muted-foreground"))
    } else if active {
        theme
            .color_by_key("accent-foreground")
            .unwrap_or_else(|| theme.color_token("accent-foreground"))
    } else {
        theme
            .color_by_key("foreground")
            .unwrap_or_else(|| theme.color_token("foreground"))
    };

    let mut chrome = ContainerProps::default();
    chrome.layout.size.width = Length::Auto;
    chrome.layout.size.height = Length::Auto;
    chrome.padding = Edges {
        left: Px(6.0),
        right: Px(6.0),
        top: Px(2.0),
        bottom: Px(2.0),
    }
    .into();
    chrome.background = background;
    chrome.corner_radii = Corners::all(super::control_chrome::CONTROL_RADIUS);

    cx.container(chrome, move |cx| {
        let mut text = TextProps::new(label.clone());
        text.wrap = TextWrap::None;
        text.overflow = TextOverflow::Clip;
        text.color = Some(foreground);
        vec![cx.text_props(text)]
    })
}
