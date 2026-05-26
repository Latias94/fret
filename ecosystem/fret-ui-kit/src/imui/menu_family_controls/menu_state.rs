use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

use super::ImUiMenubarPolicyState;

pub(super) struct BeginMenuState {
    pub(super) menubar_policy: Option<ImUiMenubarPolicyState>,
    pub(super) popup_open: Model<bool>,
    pub(super) row_open: Model<bool>,
    was_open_model: Model<bool>,
    was_popup_open_model: Model<bool>,
    pub(super) open_before: bool,
    pub(super) popup_open_before: bool,
    was_open_before_render: bool,
    was_popup_open_before_render: bool,
}

pub(super) struct MenuRenderState {
    pub(super) popup_open_after: bool,
}

pub(super) fn capture_begin_menu_state<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
) -> BeginMenuState {
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
    let open_before = read_bool_model(ui, &row_open);
    let popup_open_before = read_bool_model(ui, &popup_open);
    let was_open_before_render = read_bool_model(ui, &was_open_model);
    let was_popup_open_before_render = read_bool_model(ui, &was_popup_open_model);

    BeginMenuState {
        menubar_policy,
        popup_open,
        row_open,
        was_open_model,
        was_popup_open_model,
        open_before,
        popup_open_before,
        was_open_before_render,
        was_popup_open_before_render,
    }
}

impl BeginMenuState {
    pub(super) fn read_row_open<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
        &self,
        ui: &mut W,
    ) -> bool {
        read_bool_model(ui, &self.row_open)
    }

    pub(super) fn read_menubar_open_menu<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
        &self,
        ui: &mut W,
    ) -> Option<Arc<str>> {
        self.menubar_policy
            .as_ref()
            .and_then(|policy| read_open_menu_model(ui, policy))
    }
}

pub(super) fn sync_open_menu_for_active_trigger<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    open_after_trigger: bool,
    trigger_clicked: bool,
    trigger_id: Option<GlobalElementId>,
) {
    let Some(policy) = state.menubar_policy.as_ref() else {
        return;
    };
    if !open_after_trigger || trigger_clicked {
        return;
    }
    let Some(trigger_id) = trigger_id else {
        return;
    };

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

pub(super) fn reconcile_menubar_after_trigger<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    open_after_trigger: bool,
    trigger_id: Option<GlobalElementId>,
) {
    let Some(policy) = state.menubar_policy.as_ref() else {
        return;
    };

    if open_after_trigger && !state.popup_open_before && state.was_popup_open_before_render {
        ui.with_cx_mut(|cx| {
            let _ = cx
                .app
                .models_mut()
                .update(&state.row_open, |value| *value = false);
            let _ = cx.app.models_mut().update(&policy.open_menu, |value| {
                if value.as_ref().is_some_and(|current| current.as_ref() == id) {
                    *value = None;
                }
            });
            if let Some(trigger_id) = trigger_id {
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
    if !state.open_before && state.was_open_before_render {
        if state.popup_open_before {
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
}

pub(super) fn toggle_menu_on_trigger_click<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
) {
    if let Some(policy) = state.menubar_policy.as_ref() {
        ui.with_cx_mut(|cx| {
            let _ = cx.app.models_mut().update(&policy.open_menu, |value| {
                if state.open_before && value.as_ref().is_some_and(|current| current.as_ref() == id)
                {
                    *value = None;
                } else {
                    *value = Some(Arc::from(id));
                }
            });
        });
    } else if state.open_before {
        ui.close_popup(id);
    }
}

pub(super) fn resolve_open_requested<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    open_menu_before: Option<Arc<str>>,
) -> bool {
    let Some(policy) = state.menubar_policy.as_ref() else {
        return read_bool_model(ui, &state.popup_open);
    };

    let open_menu_now = read_open_menu_model(ui, policy);
    let should_close = state.open_before
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
                .update(&state.row_open, |value| *value = false);
        });
        ui.close_popup(id);
    }
    let requested_by_policy = open_menu_now
        .as_ref()
        .is_some_and(|current| current.as_ref() == id);
    requested_by_policy || (state.open_before && !should_close)
}

pub(super) fn activate_menubar_trigger_if_requested<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    open_requested: bool,
    state: &BeginMenuState,
    trigger_id: Option<GlobalElementId>,
) {
    let Some(menubar_policy) = state.menubar_policy.as_ref() else {
        return;
    };
    if !open_requested {
        return;
    }
    let Some(trigger_id) = trigger_id else {
        return;
    };

    ui.with_cx_mut(|cx| {
        let open_for_state = state.row_open.clone();
        let _ = cx
            .app
            .models_mut()
            .update(&menubar_policy.group_active, |value| {
                *value = Some(menubar_trigger_row::MenubarActiveTrigger {
                    trigger: trigger_id,
                    open: open_for_state,
                });
            });
        let _ = cx
            .app
            .models_mut()
            .update(&state.row_open, |value| *value = true);
    });
}

pub(super) fn close_disabled_popup_if_opened<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    enabled: bool,
    popup_opened: bool,
) {
    if enabled || !popup_opened {
        return;
    }

    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&state.row_open, |value| *value = false);
    });
    ui.close_popup(id);
}

pub(super) fn record_render_state<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    state: &BeginMenuState,
) -> MenuRenderState {
    let open_after = read_bool_model(ui, &state.row_open);
    let popup_open_after = read_bool_model(ui, &state.popup_open);
    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&state.was_open_model, |value| *value = open_after);
        let _ = cx
            .app
            .models_mut()
            .update(&state.was_popup_open_model, |value| {
                *value = popup_open_after
            });
    });

    MenuRenderState { popup_open_after }
}

fn read_bool_model<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    model: &Model<bool>,
) -> bool {
    ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, value| *value)
            .unwrap_or(false)
    })
}

fn read_open_menu_model<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    policy: &ImUiMenubarPolicyState,
) -> Option<Arc<str>> {
    ui.with_cx_mut(|cx| {
        cx.read_model(
            &policy.open_menu,
            fret_ui::Invalidation::Paint,
            |_app, value| value.clone(),
        )
        .unwrap_or(None)
    })
}
