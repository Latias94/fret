use std::sync::Arc;

use fret_ui::UiHost;

use super::ImUiMenubarPolicyState;
use crate::imui::{BeginMenuOptions, DisclosureResponse, ImUiFacade, UiWriterImUiFacadeExt};
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

pub(in crate::imui) fn begin_menu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: BeginMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::super::imui_is_disabled(cx));
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
        super::trigger::menu_trigger_with_options(
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

    let popup_opened = super::super::popup_overlay::begin_popup_menu_with_options(
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
