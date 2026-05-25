//! Immediate-mode menu-bar helpers.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::{
    BeginMenuOptions, DisclosureResponse, ImUiFacade, MenuBarOptions, UiWriterImUiFacadeExt,
};
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

mod submenu;
mod submenu_state;
mod trigger;
mod visual;

pub(super) use submenu::begin_submenu_with_options;

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
        trigger::menu_trigger_with_options(
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
        && let Some(trigger_id) = trigger.id()
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
            if let Some(trigger_id) = trigger.id() {
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
        && let Some(trigger_id) = trigger.id()
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
    if open_requested && let Some(anchor) = trigger.rect() {
        ui.open_popup_at(id, anchor);
    }

    let popup_opened = super::popup_overlay::begin_popup_menu_with_options(
        ui,
        id,
        trigger.id(),
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

#[cfg(test)]
mod tests;
